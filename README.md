# niri-float

Listens to the `niri-ipc` event stream, and requests windows float dynamically.
This is useful for applications that set their title dynamically, after their creation, like firefox windows.

Currently only floats `Bitwarden Extension windows`.
But I plan to add a configuration file to allow any rule to be matched and floated.