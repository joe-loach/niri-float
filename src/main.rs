mod config;
mod rules;

use anyhow::Context;
use niri_ipc::{
    Action, Event, PositionChange, Request, Response, SizeChange, Window,
    socket::Socket,
    state::{EventStreamState, EventStreamStatePart},
};

use rules::*;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let Some(rules) = read_rules_from_config() else {
        log::error!("Cannot find rules in config file, exiting");
        log::error!(
            "No rules at: {}",
            config::rules_path().expect("no path to rules").display()
        );
        return Ok(());
    };

    let rules = rules
        .into_iter()
        .map(Rule::try_compile)
        .collect::<Result<Vec<_>, _>>()
        .context("One or more rules failed to compile")?;

    let (reply, mut event_stream) = Socket::connect()?.send(Request::EventStream)?;

    assert!(
        matches!(reply, Ok(Response::Handled)),
        "event stream request denied"
    );

    let mut state = EventStreamState::default();

    while let Ok(event) = event_stream() {
        let should_fix = should_fix(&event, &state, &rules);

        state.apply(event);

        if let Some(id) = should_fix {
            if let Err(e) = float_window(id) {
                log::error!("Failed to float window: {}", e);
            }
        }
    }

    Ok(())
}

fn should_fix(event: &Event, state: &EventStreamState, rules: &[CompiledRule]) -> Option<u64> {
    if let Event::WindowOpenedOrChanged { window } = &event {
        if let Some(old) = state.windows.windows.get(&window.id) {
            let title_changed = old.title != window.title;
            let app_id_changed = old.app_id != window.app_id;

            // only fix if the title or app_id changed, nothing else
            if !(title_changed || app_id_changed) {
                return None;
            }
        }

        matches_any_rule(rules, window).then_some(window.id)
    } else {
        None
    }
}

fn matches_any_rule(rules: &[CompiledRule], window: &Window) -> bool {
    for rule in rules {
        if rule.matches(window) {
            return true;
        }
    }

    false
}

fn float_window(id: u64) -> anyhow::Result<()> {
    log::debug!("Floating window: {}", id);

    let (reply, _) = Socket::connect()?.send(Request::FocusedOutput)?;

    let Ok(Response::FocusedOutput(Some(output))) = reply else {
        anyhow::bail!("Request for focused output denied");
    };

    // float the window
    let _ = Socket::connect()?.send(Request::Action(Action::MoveWindowToFloating {
        id: Some(id),
    }))?;

    const PROPORTION_WIDTH: f64 = 0.2;
    const PROPORTION_HEIGHT: f64 = 0.5;

    // let resize the window
    let _ = Socket::connect()?.send(Request::Action(Action::SetWindowWidth {
        id: Some(id),
        change: SizeChange::SetProportion(PROPORTION_WIDTH * 100.0),
    }));
    let _ = Socket::connect()?.send(Request::Action(Action::SetWindowHeight {
        id: Some(id),
        change: SizeChange::SetProportion(PROPORTION_HEIGHT * 100.0),
    }));

    if let Some(dimensions) = output.logical {
        // the anchor point for moving is top left,
        // we first need to calculate how large the window is
        // and subtract from the center to make that the anchor
        let window_center_x = PROPORTION_WIDTH * (dimensions.width as f64) / 2.0;
        let window_center_y = PROPORTION_HEIGHT * (dimensions.height as f64) / 2.0;
        let center_x = (dimensions.width as f64 / 2.0) - window_center_x;
        let center_y = (dimensions.height as f64 / 2.0) - window_center_y;
        // move window to the center
        let _ = Socket::connect()?.send(Request::Action(Action::MoveFloatingWindow {
            id: Some(id),
            x: PositionChange::SetFixed(center_x),
            y: PositionChange::SetFixed(center_y),
        }))?;
    } else {
        log::warn!("no logical dimensions for output, cannot center");
    }

    // focus the window
    let _ = Socket::connect()?.send(Request::Action(Action::FocusWindow { id }))?;

    Ok(())
}
