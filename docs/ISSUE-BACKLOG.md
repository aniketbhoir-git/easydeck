# Issue backlog

Tracked on GitHub (once the repo is public) with the same ids as below.

Each issue below is a **self-contained, paste-ready prompt**. Copy the body into your model
together with `docs/SPEC.md` §0 (ground rules).

## Labels

- `tier:max` — needs a top-tier model (Claude Max/Ultra, Sonnet 4.5+, Opus, GPT-5 class). Maintainer-owned by default.
- `tier:any` — any decent model (Haiku 4.5, DeepSeek V3.2, Gemini 2.5 Flash, Qwen3). Community token-friendly.
- `needs-windows-test` — must be validated on a real Windows box before merge. Touch issues
  also want a touch screen.
- Size: `S` < 1 day of focused work · `M` 1–3 days · `L` up to a week.

Ids use the module prefix: `R1-2` is the second issue of module R1.

## Dependency map

```
R1-1 scaffold ─► R1-2 deck model+validate ─► R1-3 renderer+launch tiles ─► R1-5 launch action
      │
      └─► R1-4 kiosk window on a chosen display   (independent of R1-2, R1-3)

after R1:
  R2-1 stats source ─► R2-2 gauge/status/text widgets      R2-3 spotify     R2-4 web embed
  R3-1 key macros     R3-2 game watch ─► R3-3 context pages     R3-4 recent-first order
  R4-1 editor shell ─► R4-2 launch/macro forms ─► R4-3 drag, resize, save
  R5-1 deck diff ─► R5-2 export pack ─► R5-3 import + action review
  R5-1 ─► R6-1 MCP server ─► R6-2 approve-diff UI ─► R6-3 FOR_LLMS.md
  R7-1 Classifier trait ─► R7-2 Jev client, R7-3 local client
```

Suggested order: **R1-1 → (R1-2 ∥ R1-4) → R1-3 → R1-5** — everything after R1 is parallelizable.

---

## R1 — App shell

### Issue R1-1 — Tauri scaffold + CI wiring (SPEC §2, §3)
- **Labels:** `tier:any`, `size:S`, `area:infra`
- **Depends on:** none
- **Goal:** a compilable Rust + Tauri 2 repo that builds and tests on Windows CI.
- **Files:** create `src-tauri/Cargo.toml`, `tauri.conf.json`, `src/main.rs`, `src/lib.rs`,
  `src/error.rs`, `ui/` Svelte + Vite + TypeScript skeleton with a `package-lock.json`.
- **Requirements:** error enum with JSON `code`/`message`/`detail`/`retryable`; a window opens
  and shows "EasyDeck"; `npm run build` in `ui/` works; the existing `windows` CI job runs
  green (see the note at the top of `.github/workflows/ci.yml`).
- **Acceptance:** `cargo build` + `cargo test` + clippy + fmt green on Windows CI; the window
  opens locally; one test proves the error type serializes to the JSON shape.
- **Tests:** unit test for error serialization. No UI tests yet.
- **LLM note:** `tier:any`; pure scaffolding, cheap model is fine.

### Issue R1-2 — Deck model, load and validate (SPEC §4, §5)
- **Labels:** `tier:any`, `size:M`, `area:deck`
- **Depends on:** R1-1
- **Goal:** read a deck file, check it against `schema/deck.schema.json`, give typed Rust data.
- **Files:** `src-tauri/src/deck/{mod,model,load,validate}.rs`, fixtures in
  `src-tauri/tests/fixtures/` (valid and invalid decks), `examples/` stays as is.
- **Requirements:** serde types that mirror the schema; `DeckStore::load(path)` returns the
  typed deck or a list of `ValidationError { path, message }`; embed the schema in the binary
  with `include_str!`; extra semantic checks the schema cannot express: widget ids unique
  per page, widgets fit inside the grid, `trigger.source` names a `game_watch` source, gauge
  and status `source` names an existing source, `homePage` exists.
- **Acceptance:** `examples/starter.deck.json` loads; each invalid fixture yields the expected
  error path; an unknown `version` gives a clear error.
- **Tests:** table-driven unit tests, one fixture per rule.
- **LLM note:** `tier:any`.

### Issue R1-3 — Deck renderer and launch tiles UI (SPEC §1.2, §4)
- **Labels:** `tier:any`, `size:M`, `area:ui`
- **Depends on:** R1-2
- **Goal:** the UI draws the loaded deck: page grid, page switcher and launch tiles.
- **Files:** `ui/src/lib/api.ts` (typed IPC client), `ui/src/components/{Deck,Page,Tile}.svelte`,
  `src-tauri/src/ipc/{commands,state}.rs` with `load_deck`, `run_action` (stub that returns ok).
- **Requirements:** CSS grid from `page.grid`; tile shows label and icon; press animation on
  `pointerdown` (scale + highlight, under 100 ms); `user-select: none`,
  `overscroll-behavior: none`, no context menu, `touch-action: manipulation`; hit target at
  least 48 px; tapping a tile calls `run_action(widget_id)`.
- **Acceptance:** the starter deck renders; tiles animate on press; text cannot be selected;
  the page does not bounce when dragged.
- **Tests:** `vitest` component tests for grid placement and for the `run_action` call.
  Attach a screenshot or short clip.
- **LLM note:** `tier:any` (Svelte UI).

### Issue R1-4 — Kiosk window on a chosen display (SPEC §1.2)
- **Labels:** `tier:max`, `size:M`, `area:display`, `needs-windows-test`
- **Depends on:** R1-1
- **Goal:** the app window fills the display the user picks, without borders.
- **Files:** `src-tauri/src/display/{mod,kiosk}.rs`, settings file (`app-settings.json`,
  separate from the deck) with `displayId`.
- **Requirements:** list displays (id, name, size, position) through Tauri's monitor API or
  windows-rs; command `list_displays`; at start, place a borderless, always-on-top (optional
  setting) window on `displayId`, or on the primary display if that id is gone; a second
  command `set_display(id)` moves the window and saves the choice; the window never takes
  focus away from a running game on tap if the platform allows (document what works).
- **Acceptance:** with two or more monitors, the window opens on the chosen one after a
  restart; unplugging the monitor falls back to the primary display without a crash.
- **Tests:** unit-test the display-choice function with fake display lists. Manual Windows
  checklist in the PR: two-monitor test, unplug test.
- **LLM note:** `tier:max` (Windows display and focus behavior).

### Issue R1-5 — Launch action runner (SPEC §5, §6.1)
- **Labels:** `tier:max`, `size:M`, `area:actions`, `needs-windows-test`
- **Depends on:** R1-2, R1-3
- **Goal:** tapping a launch tile starts the game, program or page.
- **Files:** `src-tauri/src/actions/{mod,launch}.rs`, wire into `ipc/commands.rs`.
- **Requirements:** `ActionRunner` trait; `LaunchRunner` handles kinds `steam`
  (`steam://rungameid/<appId>`), `exe` (spawn without a shell; try `fallback` when `path`
  does not exist), `uri` and `url` (open with the default handler). The OS call sits behind a
  small `Opener` trait so tests use a fake. Return typed errors (`NotFound`, `SpawnFailed`).
- **Acceptance:** each kind opens the right target on Windows; a missing `exe` path uses the
  fallback; an error shows a short toast in the UI.
- **Tests:** fake `Opener` unit tests for each kind and for the fallback. Manual test of
  each kind on Windows. No automated test launches a real program.
- **LLM note:** `tier:max` (process spawn and URI handling).

---

## R2 — Live widgets

### Issue R2-1 — System stats source (SPEC §5)
- **Labels:** `tier:any`, `size:M`, `area:sources`, `needs-windows-test`
- **Depends on:** R1-2
- **Goal:** a source that streams CPU load, RAM use and GPU load/temperature.
- **Files:** `src-tauri/src/sources/{mod,system_stats}.rs`.
- **Requirements:** `Source` trait with `subscribe()` returning a stream of
  `SourceValue` (JSON object); CPU and RAM through a crate such as `sysinfo`; GPU through
  `nvidia-smi --query-gpu=...` when present, else the metric is `null`; interval from the
  source config; the command runner sits behind a trait so tests can fake `nvidia-smi` output.
  Push values to the UI as a Tauri event `source:<id>`.
- **Acceptance:** the UI console shows updating values every interval; missing GPU does not
  error.
- **Tests:** parse tests for `nvidia-smi` CSV output (with a missing-GPU case); interval test
  with a fake clock.
- **LLM note:** `tier:any`.

### Issue R2-2 — Gauge, status and text widgets (SPEC §4)
- **Labels:** `tier:any`, `size:M`, `area:ui`
- **Depends on:** R1-3, R2-1
- **Goal:** render the `gauge`, `status` and `text` (label, clock, date) widgets.
- **Files:** `ui/src/components/{Gauge,Status,TextWidget}.svelte`, a `sourceStore` that
  subscribes to `source:<id>` events.
- **Requirements:** gauge draws an arc or bar between `min` and `max` with `unit`; status
  shows OK or problem from `okWhen`; clock updates each second from the system time and
  honors `format`. Widgets show a neutral "no data" state when the source is silent.
- **Acceptance:** the starter deck shows a live CPU gauge and a ticking clock.
- **Tests:** `vitest` tests for value-to-angle mapping and for the "no data" state.
- **LLM note:** `tier:any`.

### Issue R2-3 — Spotify widget (SPEC §4, STRATEGY §7)
- **Labels:** `tier:max`, `size:L`, `area:media`, `needs-windows-test`
- **Depends on:** R1-3
- **Goal:** now-playing display with play/pause, next and previous.
- **Files:** `src-tauri/src/media/spotify.rs`, `ui/src/components/Media.svelte`.
- **Requirements:** OAuth with PKCE (no client secret) using a user-provided client id in
  app settings; open the sign-in page in the system browser and receive the code on a
  `127.0.0.1` loopback port; store the refresh token in the Windows credential store, never
  in the deck; poll the player endpoint; show title, artist, cover; buttons call the player
  endpoints. Only the controls listed in `controls` render.
- **Acceptance:** after sign-in the widget shows the current track and the buttons work;
  signing out clears the stored token.
- **Tests:** `httpmock` tests for the token exchange and the player calls. Manual test with a
  real account in the PR, with a screenshot.
- **LLM note:** `tier:max` (OAuth flow and credential store).

### Issue R2-4 — Web embed widget (SPEC §4)
- **Labels:** `tier:any`, `size:M`, `area:ui`, `needs-windows-test`
- **Depends on:** R1-3
- **Goal:** show any `http(s)` page (YouTube, a dashboard) inside a tile.
- **Files:** `ui/src/components/Embed.svelte`, `src-tauri/src/ipc` embed helpers if a child
  webview is needed.
- **Requirements:** use an iframe first; if a site blocks framing, fall back to a Tauri child
  webview placed over the tile rectangle, sharing the app's WebView2 profile so logins persist;
  `refreshSeconds` reloads; only `http` and `https` URLs load; the page keeps touch scroll
  inside the tile only.
- **Acceptance:** a YouTube video and a plain dashboard page both show and respond to touch;
  logins persist across restarts.
- **Tests:** `vitest` test that rejects non-http URLs. Manual test of both fallbacks.
- **LLM note:** `tier:any`.

---

## R3 — Macros and game detection

### Issue R3-1 — Key macro runner (SPEC §5, §6.1)
- **Labels:** `tier:max`, `size:L`, `area:actions`, `needs-windows-test`
- **Depends on:** R1-5
- **Goal:** run key macros with hardware scan codes, once or in a loop.
- **Files:** `src-tauri/src/actions/macros.rs`, a `KeySender` trait with a `SendInput`
  implementation in `src-tauri/src/actions/keys_windows.rs`.
- **Requirements:** parse `keys` entries (`"1"`, `"f5"`, `"ctrl+q"`) into scan codes; send with
  `SendInput` using `KEYEVENTF_SCANCODE`; honor `holdMs`, `keyGapMs`, `cycleMs`; with `toggle`,
  one tap starts a loop task and the next tap stops it; at most one loop per widget; a global
  stop on app exit. Document that games running as administrator need the app elevated.
- **Acceptance:** a macro tile types into Notepad in the right order and timing; a toggle
  macro starts and stops; closing the app stops all loops.
- **Tests:** unit tests for key-name parsing and for timing with a fake `KeySender` and fake
  clock. Manual test in Notepad. No automated test sends real keys.
- **LLM note:** `tier:max` (input injection and task cancellation).

### Issue R3-2 — Game watch source (SPEC §5, §6.2)
- **Labels:** `tier:max`, `size:M`, `area:sources`, `needs-windows-test`
- **Depends on:** R1-2
- **Goal:** report when a watched process starts or stops.
- **Files:** `src-tauri/src/sources/game_watch.rs`.
- **Requirements:** poll the process list at `intervalMs` (windows-rs toolhelp or `sysinfo`);
  compare against `processes` case-insensitively; emit `started`/`stopped` events with the
  process name; a `ProcessLister` trait so tests use fake lists; debounce a flapping
  process (ignore a stop shorter than one interval).
- **Acceptance:** starting and closing Notepad (watched as `notepad.exe`) produces one
  `started` and one `stopped` event.
- **Tests:** unit tests over scripted process lists, including the debounce case.
- **LLM note:** `tier:max`.

### Issue R3-3 — Context pages (SPEC §6.2)
- **Labels:** `tier:any`, `size:M`, `area:core`
- **Depends on:** R3-2, R1-3
- **Goal:** a page with a `trigger` opens when its process starts; the deck returns when it stops.
- **Files:** `src-tauri/src/ipc/state.rs` (page controller), `ui/src/lib/pageStore.ts`.
- **Requirements:** a `PageController` holds the current page and a "previous" page; on
  `started` for a trigger's source and process, switch; on `stopped`, return to the previous
  page, or `homePage`; a manual page change while in a context page wins until the process
  stops; two triggers at once: the latest start wins.
- **Acceptance:** the sequence start A, start B, stop B, stop A ends on the home page after
  returning through A; manual navigation is not overridden.
- **Tests:** state-machine unit tests for each sequence above.
- **LLM note:** `tier:any` (pure state logic).

### Issue R3-4 — Recent-first ordering (SPEC §4)
- **Labels:** `tier:any`, `size:S`, `area:core`
- **Depends on:** R1-5
- **Goal:** launch tiles marked `"order": "recent"` sort by play history.
- **Files:** `src-tauri/src/deck/history.rs`, a `history.json` in the app data folder.
- **Requirements:** record `{widgetId, timestamp}` on each successful launch; keep the last
  200 entries; the renderer sorts `recent` widgets by last launch time, then by launch count
  in the current hour-of-day bucket as a tie-breaker; sorting changes the order of the
  `recent` tiles among their own slots only, never the grid positions of other widgets.
- **Acceptance:** after launching tile B then tile A, the recent group shows A first.
- **Tests:** unit tests for sorting and the tie-breaker; history trims to 200.
- **LLM note:** `tier:any`.

---

## R4 — GUI editor

### Issue R4-1 — Editor shell and edit mode (SPEC §1.1)
- **Labels:** `tier:any`, `size:M`, `area:ui`
- **Depends on:** R1-3
- **Goal:** a toggle that switches the deck between run mode and edit mode.
- **Files:** `ui/src/editor/{EditorShell,Palette,Inspector}.svelte`, editor store.
- **Requirements:** edit mode shows a palette of widget types, a selected-tile inspector and
  an unsaved-changes bar; the working copy is a draft in memory; every change re-validates
  through `validate_deck` (IPC command wrapping R1-2) and shows errors inline; leaving edit
  mode with unsaved changes asks to save or discard.
- **Acceptance:** entering edit mode shows the palette; adding a widget places it in the first
  free grid cell; an invalid draft cannot be saved.
- **Tests:** `vitest` tests for the draft store (add, remove, validate error shown).
- **LLM note:** `tier:any`.

### Issue R4-2 — Launch and macro forms (SPEC §4)
- **Labels:** `tier:any`, `size:M`, `area:ui`
- **Depends on:** R4-1
- **Goal:** forms that set a tile's label, icon and action without typing JSON.
- **Files:** `ui/src/editor/forms/{LaunchForm,MacroForm}.svelte`.
- **Requirements:** launch form: kind picker (steam, exe, uri, url) with only the fields for
  that kind; exe file picker through the Tauri dialog plugin; macro form: type the key
  list, or "record" by capturing the next key presses; numeric inputs for the timings with the
  schema's limits; any new action shows a small "this tile can run a program / send keys" note.
- **Acceptance:** a user creates a Steam tile and a macro tile with no JSON typing, and the
  draft validates.
- **Tests:** `vitest` tests that each form emits JSON that passes the schema.
- **LLM note:** `tier:any`.

### Issue R4-3 — Move, resize and save (SPEC §4)
- **Labels:** `tier:any`, `size:M`, `area:ui`
- **Depends on:** R4-1
- **Goal:** drag tiles on the grid, resize them, and save the deck.
- **Files:** `ui/src/editor/GridEditor.svelte`, `src-tauri/src/deck/save.rs`.
- **Requirements:** pointer-based drag with snap to grid cells and no overlap (a drop on an
  occupied cell is refused); resize handle sets `w`/`h`; delete with an undo; undo and redo
  stacks; `save_deck` writes atomically (temp file, then rename) and keeps one `.bak` copy.
- **Acceptance:** drag, resize, delete, undo and save work with touch and mouse; a crash
  during save never leaves a half-written deck.
- **Tests:** `vitest` tests for snap and overlap rules; Rust test that `save_deck` is atomic
  (the old file stays intact when the write fails).
- **LLM note:** `tier:any`.

---

## R5 — Export and import

### Issue R5-1 — Deck diff (SPEC §5, §6.3)
- **Labels:** `tier:any`, `size:M`, `area:deck`
- **Depends on:** R1-2
- **Goal:** compute a readable diff between two decks, and flag changed actions.
- **Files:** `src-tauri/src/deck/diff.rs`.
- **Requirements:** `diff(old, new) -> DeckDiff` listing added, removed and changed pages,
  widgets and sources; each entry has a path and a short human summary; a separate
  `actions_changed()` list holds every new or changed action with its widget id and kind;
  a function `apply(old, diff)` is not needed, since the approved deck replaces the old one.
- **Acceptance:** a fixture pair with one added launch tile, one moved tile and one changed
  macro yields exactly three entries, and two of them in `actions_changed()` (the new launch
  and the changed macro).
- **Tests:** table-driven tests on fixture pairs, including identical decks (empty diff).
- **LLM note:** `tier:any`.

### Issue R5-2 — Export a `.easydeck` pack (SPEC §4)
- **Labels:** `tier:any`, `size:M`, `area:pack`
- **Depends on:** R1-2
- **Goal:** save the deck and its icons as one zip file.
- **Files:** `src-tauri/src/pack/{mod,export}.rs`, export button in the app menu.
- **Requirements:** zip with `deck.json` and `icons/`; rewrite icon references to pack-relative
  paths; copy only icons the deck uses; strip any absolute user path from `exe` actions into
  a placeholder (for example `%PROGRAMFILES%` where possible, otherwise keep the file name
  and warn); never include settings, history or credentials.
- **Acceptance:** exporting the starter deck yields a pack that unzips to valid JSON and the
  used icons only.
- **Tests:** round-trip test (export, unzip, validate); test that a user home path never
  appears in the pack.
- **LLM note:** `tier:any`.

### Issue R5-3 — Import with preview and action review (SPEC §0, §6.4)
- **Labels:** `tier:max`, `size:L`, `area:pack`
- **Depends on:** R5-1, R5-2, R1-3
- **Goal:** import a pack safely; the user sees what it can run before accepting.
- **Files:** `src-tauri/src/pack/import.rs`, `ui/src/components/ImportReview.svelte`.
- **Requirements:** open by file picker and by double-clicking a `.easydeck` file (file
  association); reject zip-slip paths, huge files and unknown entries; validate `deck.json`;
  the review screen shows a read-only preview of each page and a list of every action
  (launch targets, key macros, shell commands) grouped by kind, with shell actions marked
  as unavailable when their id is not in the local allow-list; Accept stores the deck as a new
  deck (never overwrites); Cancel changes nothing.
- **Acceptance:** a pack with a launch, a macro and a shell action shows all three in the
  review; a pack with `../` paths is rejected; Cancel leaves no files behind.
- **Tests:** zip-slip and size-limit tests; a fixture pack per case.
- **LLM note:** `tier:max` (untrusted-input handling).

---

## R6 — AI layer

### Issue R6-1 — Local MCP server (SPEC §5)
- **Labels:** `tier:max`, `size:L`, `area:mcp`
- **Depends on:** R1-2, R5-1
- **Goal:** let an LLM client read the deck and propose a change.
- **Files:** `src-tauri/src/mcp/{mod,tools}.rs`.
- **Requirements:** choose between the `rmcp` crate and a small JSON-RPC server (write the
  choice and reason in the PR); bind `127.0.0.1` only, on a port from app settings; require a
  random bearer token that the app shows once for the user to copy into the LLM client;
  tools: `deck.get` (current deck), `deck.schema` (the JSON Schema), `deck.validate`
  (given a deck, return errors), `deck.propose_diff` (given a full new deck, validate it, compute
  the diff with R5-1, queue it); the server never applies anything.
- **Acceptance:** an MCP client connects with the token, lists the four tools and calls each;
  a call without a token or from another interface fails; an invalid proposal returns the
  validator's errors and queues nothing.
- **Tests:** integration tests that start the server on a random port; a test that the
  listener address is loopback; token tests.
- **LLM note:** `tier:max` (protocol and security).

### Issue R6-2 — Approve-diff UI (SPEC §0, §6.3)
- **Labels:** `tier:max`, `size:M`, `area:ui`
- **Depends on:** R6-1
- **Goal:** the user sees each queued LLM proposal and approves or rejects it.
- **Files:** `ui/src/components/DiffReview.svelte`, `src-tauri/src/mcp/queue.rs`.
- **Requirements:** a badge shows queued proposals; the review screen shows the diff from
  R5-1 and highlights `actions_changed()` first; Approve replaces the deck atomically (R4-3's
  save) and reloads; Reject drops the proposal; a proposal made against an older deck version
  than the current one is marked stale and cannot be approved; approval needs a tap on the
  touch screen or a click, never an MCP call.
- **Acceptance:** an LLM proposal appears, shows its new actions, and applies only after
  Approve; a stale proposal is blocked.
- **Tests:** unit tests for the queue (stale detection, reject, approve); `vitest` test that
  the new-actions list renders before the Approve button is enabled.
- **LLM note:** `tier:max` (approval gate).

### Issue R6-3 — Write `docs/FOR_LLMS.md` (SPEC §0)
- **Labels:** `tier:any`, `size:M`, `area:docs`
- **Depends on:** R6-1
- **Goal:** turn the stub into a page that any LLM can follow to build a deck.
- **Files:** `docs/FOR_LLMS.md`, 4 or more tested examples in `examples/`.
- **Requirements:** keep to the outline in the stub; give the exact MCP tool inputs and
  outputs; one worked example per widget type; one example per action kind; a "common mistakes"
  list that quotes real validator messages; every example deck in the page also exists in
  `examples/` and passes the schema in CI.
- **Acceptance:** a fresh LLM session given only this page and the MCP server builds a
  two-page deck that validates; the PR includes the transcript summary.
- **Tests:** CI validates each file in `examples/`.
- **LLM note:** `tier:any`.

---

## R7 — Classifier

### Issue R7-1 — `Classifier` trait and a rule-based default (SPEC §5)
- **Labels:** `tier:any`, `size:S`, `area:classifier`
- **Depends on:** R1-1
- **Goal:** the trait other code calls, and a default that needs no model.
- **Files:** `src-tauri/src/classifier/{mod,rules}.rs`.
- **Requirements:** `Classifier::classify(question, context) -> Verdict { yes, probability }`;
  a `ClassifierConfig` (`off`, `jev`, `local`) in app settings, default `off`; a
  `RuleClassifier` that answers the "game session now?" question from `game_watch` state, so
  the app works with no model; a `Question` enum for the known questions instead of free text.
- **Acceptance:** with config `off` the app never calls a model; `RuleClassifier` answers
  correctly for running and stopped games.
- **Tests:** unit tests for each `Question` and for the config switch.
- **LLM note:** `tier:any`.

### Issue R7-2 — Jev client on the AI Gateway (SPEC §5)
- **Labels:** `tier:any`, `size:M`, `area:classifier`
- **Depends on:** R7-1
- **Goal:** a `Classifier` that calls Vercel AI Gateway model `typesafe-ai/jev` over HTTP.
- **Files:** `src-tauri/src/classifier/jev.rs`, `docs/classifier-http-shape.md`.
- **Requirements:** first capture the HTTP request and response shape of the Jev call (the AI
  SDK is TypeScript only; run the SDK once with request logging, and record the shape in
  `docs/classifier-http-shape.md` without any key); then implement it with reqwest; the API key
  comes from an environment variable or the credential store, never the 12-hour OIDC token;
  timeout 5 s; on error return `Err`, never a guessed answer; add a short note on price per
  1,000 calls, checked on the date you wrote it.
- **Acceptance:** the client returns a `Verdict` against a mocked server; with no key set it
  returns a clear "not configured" error and makes no network call.
- **Tests:** `httpmock` tests for success, timeout, 401 and malformed body.
- **LLM note:** `tier:any` (HTTP client).

### Issue R7-3 — Local model client (SPEC §5)
- **Labels:** `tier:any`, `size:M`, `area:classifier`
- **Depends on:** R7-1
- **Goal:** a `Classifier` that calls a local model server the user already runs.
- **Files:** `src-tauri/src/classifier/local.rs`.
- **Requirements:** call an OpenAI-compatible `/v1/chat/completions` endpoint on a URL from
  app settings (default `http://127.0.0.1:11434/v1` style); ask for a one-word yes or no and
  parse it; probability from logprobs when the server returns them, else 1.0 or 0.0;
  refuse non-loopback URLs unless the user sets an explicit "allow LAN" option.
- **Acceptance:** against a mocked server the client returns the right `Verdict`; a
  non-loopback URL is refused by default.
- **Tests:** `httpmock` tests for yes, no, ambiguous reply (error) and the URL rule.
- **LLM note:** `tier:any`.

---

## Totals

25 issues: R1 x5, R2 x4, R3 x4, R4 x3, R5 x3, R6 x3, R7 x3.

| Module | Issues | `tier:max` | `tier:any` |
|---|---|---|---|
| R1 | 5 | R1-4, R1-5 | R1-1, R1-2, R1-3 |
| R2 | 4 | R2-3 | R2-1, R2-2, R2-4 |
| R3 | 4 | R3-1, R3-2 | R3-3, R3-4 |
| R4 | 3 | — | R4-1, R4-2, R4-3 |
| R5 | 3 | R5-3 | R5-1, R5-2 |
| R6 | 3 | R6-1, R6-2 | R6-3 |
| R7 | 3 | — | R7-1, R7-2, R7-3 |
