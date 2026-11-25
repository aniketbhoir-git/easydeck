# A-1 focus spike (throwaway)

Answers EasyDeck issue A-1 (#30): can a deck window take a tap without taking focus, and how
does `SendInput` behave. Not product code. Not a workspace member. The steps and the report
are in `docs/spikes/A-1-focus-input.md`.

Build: `cargo build --release` in this folder. The `custom-protocol` feature is on by default,
so the exe loads the embedded page `dist/index.html` (no dev server). Do not use `tauri dev`.
Cargo not on PATH in Bash: `export PATH="$HOME/.cargo/bin:$PATH"`.

Run: double-click `run-spike.cmd` (builds, then starts in run mode). Or start
`target\release\easydeck-a1-focus-spike.exe` with flags:

- `--monitor <index>`: place the window on that display (the indexes show in the window header).
- `--topmost`: also set always-on-top (or tap "Toggle topmost").
- `--no-noactivate`: start without `WS_EX_NOACTIVATE`, as a control run.
- `--selftest [--out <file>]`: CI only (question 6). Opens a normal window, takes focus, sends
  keys to itself. Do not run it on a real desktop.

Log: shown in the window and appended as JSON lines to `%TEMP%\easydeck-a1-spike.jsonl`.
Each tap records the pointer type, the foreground window before and about 200 ms after,
the cursor state and the `SendInput` result.
