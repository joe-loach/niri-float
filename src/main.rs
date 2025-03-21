use std::sync::LazyLock;

use niri_ipc::{
    Action, Event, PositionChange, Request, Response, SizeChange,
    socket::Socket,
    state::{EventStreamState, EventStreamStatePart},
};
use regex_lite::Regex;

static BITWARDEN_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\(Bitwarden Password Manager\) - Bitwarden").unwrap());

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let (reply, mut event_stream) = Socket::connect()?.send(Request::EventStream)?;

    assert!(
        matches!(reply, Ok(Response::Handled)),
        "event stream request denied"
    );

    let mut state = EventStreamState::default();

    log::debug!("listening");

    while let Ok(event) = event_stream() {
        let should_fix = should_fix(&event, &state);

        state.apply(event);

        if let Some(id) = should_fix {
            log::debug!("Fixing bitwarden window");

            fix_bitwarden(id)?;
        }
    }

    Ok(())
}

fn should_fix(event: &Event, state: &EventStreamState) -> Option<u64> {
    if let Event::WindowOpenedOrChanged { window } = &event {
        if let Some(old) = state.windows.windows.get(&window.id) {
            let title_changed = old.title != window.title;
            let app_id_changed = old.app_id != window.app_id;

            // only fix if the title or app_id changed, nothing else
            if !(title_changed || app_id_changed) {
                return None;
            }
        }

        let is_bitwarden = window
            .title
            .as_ref()
            .is_some_and(|title| BITWARDEN_REGEX.is_match(title));

        let is_firefox = window.app_id.as_ref().is_some_and(|id| id == "firefox");

        log::debug!("Bitwarden? {} Firefox? {}", is_bitwarden, is_bitwarden);
        (is_bitwarden && is_firefox).then_some(window.id)
    } else {
        None
    }
}

fn fix_bitwarden(id: u64) -> anyhow::Result<()> {
    let (reply, _) = Socket::connect()?.send(Request::FocusedOutput)?;

    let Ok(Response::FocusedOutput(Some(output))) = reply else {
        panic!("request for focused output denied");
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
