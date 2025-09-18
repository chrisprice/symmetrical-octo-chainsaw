# symmetrical-octo-chainsaw

To develop in VSCode, in .vscode/settings.json, comment out either the firmware OR the std linked project.

To run the app on target (must cd to pick up cargo config) -

`cd firmware && cargo run`

To run the utilities on the host e.g. the HTTP server -

`cd std && cargo run --bin http`
