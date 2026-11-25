# Spike A-1: focus and input

Issue A-1 (#30), SPEC §1.2 and §8.3. The code is in `spikes/focus/` (throwaway, not a
workspace member). This file is the report. Every result is **pending** until the maintainer
runs the steps on the machine with the touch monitor.

## Setup (once)

1. Open a terminal in `spikes/focus/`. Build with `cargo build --release` (Bash: first
   `export PATH="$HOME/.cargo/bin:$PATH"`). Or double-click `run-spike.cmd`.
2. Start `target\release\easydeck-a1-focus-spike.exe`. Add `--monitor <index>` to put the
   window on the touch monitor. The window header lists the monitors and their indexes.
3. The header shows `NOACTIVATE true` in run mode. Each tap adds a block to the on-screen log
   and a JSON line to `%TEMP%\easydeck-a1-spike.jsonl`.

Reading a log block: `fg_before` and `fg_after` are the foreground window right at the tap and
about 200 ms later (`process`, `pid`, `title`, `elevated`, `minimized`, `fills_monitor`,
`is_ours`). `pointer_type` is touch, mouse or pen. `cursor_*.flags` is 1 when the cursor is
showing and 2 when it is suppressed. `send_input` lists one entry per key event: `sent` is the
`SendInput` return value (1 = accepted) and `last_error` is `GetLastError`.

Use a finger for every "tap" below; repeat once with the mouse. Control run: start with
`--no-noactivate` and repeat Q1 to confirm the window does take focus without the style.

## Q1. Taps and clicks reach WebView2 while another window stays foreground

Method:
1. Open Notepad on the main monitor and click in it so it is the foreground window.
2. Tap "Tap test" on the touch monitor three times with a finger, then three times with the mouse.
3. Read the counter and the log blocks.

Log to read: `fg_before.process` and `fg_after.process` must both be `notepad.exe`, with
`is_ours` false. `pointer_type` should be `touch` for finger taps. The counter must count all taps.

**Observed result:** pending

## Q2. A tap does not minimize or disturb a full-screen program on the other display

Method:
1. Exclusive full screen: start a program in exclusive full screen on the main monitor (a game
   that offers "Fullscreen", not "Borderless"). Note the program name.
2. With the game running, tap "Tap test" on the touch monitor three times. Watch the game: does
   it minimize, flicker, lose its mode, or change the resolution?
3. Move the mouse in the game. Does the cursor still behave, or is mouse capture lost?
4. Borderless: repeat steps 1 to 3 with a program in borderless windowed full screen (a
   browser in F11 mode also works, or the same game set to "Borderless").
5. Repeat once with `--topmost` (or tap "Toggle topmost"), because a topmost deck can overlap a
   full-screen window.

Log to read: `fg_before` and `fg_after` must show the game, not the spike. `fg_after.minimized`
must be false. Compare `cursor_before` and `cursor_after` (`flags`, position, and
`own_thread_capture`). `fg_after.gui_thread.capture` is the capture window of the foreground thread.

**Observed result:** pending

## Q3. Scan-code keys from `SendInput` reach the foreground window

Method:
1. Open Notepad, click in it, leave it empty.
2. Tap "Send keys 1 2 3" on the touch monitor.
3. Read Notepad (expected text: `123`) and the log line.
4. Tap "Send keys after 3 s", click back into Notepad within 3 seconds, and read Notepad again.
5. Repeat with the game from Q2 in the foreground, if the game accepts keys: check the key in
   the game (a key that has a visible effect works best; the spike sends the keys 1, 2, 3).

Log to read: every `send_input.events[].sent` should be 1 and `last_error` 0; then
`fg_after.process` is the target program. Note: `SendInput` does not report a block by UIPI
(see Q4), so only the target tells whether the keys arrived.

**Observed result:** pending

## Q4. The target runs elevated

Method:
1. Start Notepad as administrator (right-click, Run as administrator) and click in it.
2. Tap "Send keys 1 2 3" on the touch monitor (the spike runs un-elevated).
3. Read Notepad and the log line. Expected, to be confirmed: Notepad stays empty, and `sent`
   is still 1.
4. Check that `fg_before.elevated` reads `true` or `null`. `null` means the spike could not
   read the token of the elevated process, which is expected from a normal process.
5. Start the spike from an elevated terminal and repeat step 2.

Log to read: `fg_before.elevated`, `send_input.events[]`, and the text in Notepad.

Options if keys must reach an elevated target (written from general knowledge; every point
marked unverified has not been tested here):

| Option | Cost |
|---|---|
| Run the whole app elevated | A UAC prompt at every start, or a scheduled task set to "run with highest privileges" for a silent start (unverified for this app). Needs a manifest `requireAdministrator`. Elevated apps also change drag and drop from non-elevated windows. The WebView2 behaviour under elevation is unverified. The app then runs with more rights than it needs; that goes against the least-privilege intent of SPEC §0. |
| `uiAccess="true"` in the app manifest | The exe must be signed with a certificate that chains to a trusted root, and installed in a secure location (for example `C:\Program Files`). A code-signing certificate costs money for an open-source project. It lets the app send input above its integrity level without full elevation. Unverified together with Tauri/WebView2: whether the WebView2 process works with a `uiAccess` host. |
| Do nothing; document the limit | No cost to build. Keys sent to an elevated game do not arrive; the tile shows no error because `SendInput` does not report a UIPI block. Needs a note in the docs and, if possible, a warning when the foreground process is elevated (readable only for some processes, see `elevated: null`). |

**Observed result:** pending

## Q5. Edit mode gets text input

Method:
1. In run mode, confirm that the text box is hidden.
2. Tap "Toggle edit mode". The header must show `NOACTIVATE false` and a text box appears.
3. Tap in the text box and type with the physical keyboard. Then, if you have one, open the
   Windows touch keyboard and type with it.
4. Tap "Toggle edit mode" again and check that the style returns (the text box hides, and a tap
   on "Tap test" no longer changes the foreground window).

Log to read: the `toggle_edit` block, `fg_before` and `fg_after` (after the toggle, tapping the
box is expected to make the spike the foreground window: `is_ours` true).

**Observed result:** pending

## Q6. Hosted `windows-latest` runner

Method: the workflow `.github/workflows/spike-a1.yml` runs on every push to the branch
`impl/A-1--focus-spike`. It builds the spike, runs `--selftest` and then a best-effort
`tauri-driver` session check. The result files are the artifact `spike-a1-results`.

- `--selftest`: the spike opens a normal window, takes focus, sends scan codes 1, 2, 3 with
  `SendInput` to itself, and counts the key events in the page (`keys_reached_webview`) and in a
  low-level keyboard hook (`ll_hook_keys`, `injected` true).
- `tauri-driver`: starts `tauri-driver` with a matching `msedgedriver` and asks for one
  WebDriver session on the spike exe.

Log to read: `a1-selftest.json` (`keys_reached_webview`, `ll_hook_keys`, `details`) and
`a1-tauri-driver.json` (`session_created`).

**Observed result** (workflow run 43236018438, runner `windows-latest`, WebView2 runtime
156.0.4314.58; the same selftest result came in the two earlier runs):

- `SendInput` to our own window: **yes**. The spike was the foreground window (`SetForegroundWindow`
  returned true, `is_ours` true, process elevated as `runneradmin`). All six scan-code events
  returned `sent` 1 with `last_error` 0, and the page received `keydown` and `keyup` for
  `Digit1`, `Digit2` and `Digit3` (`keys_reached_webview` true, exit code 0, about 5 s).
  `ll_hook_keys` was empty in every run: the low-level hook, installed from `setup`, never saw an
  event. The cause is not examined; the WebView result above does not depend on it. This shows
  that a hosted runner has an interactive desktop that accepts injected input to a window of the
  same process. It does not show that input reaches a second program (the probe, SPEC §8.3);
  that needs a test with `easydeck-probe`.
- `tauri-driver`: **no session** in the best-effort step (failed in all three runs, which also
  tried a visible start window). `msedgedriver` 156 (downloaded for the installed runtime) answered
  `session not created: DevToolsActivePort file doesn't exist`. A direct check showed that the
  spike with `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9222` did not open that
  port in 8 s. Cause unconfirmed. An untested guess: the environment variable is not honoured by this
  Tauri/wry window setup, or the elevated runner session blocks it. Not pursued further in
  this spike; A-6 must solve it before the smoke test relies on `tauri-driver`.

## Decision

To fill in after Q1 to Q6. If the answer to Q1 is no, stop and return to the architect with
the options (SPEC §1.2 and the issue text).

- **Window style for run mode:** pending
- **Window style for edit mode:** pending
- **Probe tests on CI: yes or no:** pending
