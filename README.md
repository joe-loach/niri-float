# niri-float

Listens to the `niri-ipc` event stream, and requests windows float based on dynamic conditions.
This is useful for applications that set their title dynamically, after their creation, like firefox windows.

# Rules

A file containing rules should be created at `$HOME/.config/niri-float/rules.toml` (or the appropriate config directory, depending on your platform).

The following are a list of currently matchable rules on a window:
* `title`
* `app-id`

An example of the rules config can be found in the repo.

# Installation

To install this locally, ensure you have `cargo` installed, and run:
```bash
cargo install --path=.
```