# Issue backlog

Backlog v4 (2026-11-11). It follows `docs/SPEC.md` v4. Each issue is self-contained: an agent
that reads `AGENTS.md` (or `CLAUDE.md`), SPEC §0 and the issue can do the work.

GitHub issues #1–#25 map to R1-1 … R7-3 in order. Module A issues and R6-4 are new in v4 and
get GitHub issues after this backlog merges.

## Labels

- `verify:auto` — CI proves the acceptance criteria. Any model may take it.
- `verify:human` — a person with the hardware must check (touch monitor, second display,
  a real game). The issue lists the manual steps.
- `review:owner` — the change touches a CODEOWNERS path (platform crate, MCP crate, risk and
  approval code, workflows). The maintainer reviews it line by line.
- Size: `S` under 1 day of focused work · `M` 1–3 days · `L` up to a week.
- `deferred` — not in v1.

## How an issue opens (test-first)

An issue is open for contributors when its **contract** is on `main`: the public types, the
trait signatures and the acceptance tests, with the tests marked
`#[ignore = "<issue id>"]` (Rust) or `test.skip` (TypeScript). The contributor removes the
ignore marks and makes the tests pass without changing what they assert. The maintainer writes
the contract. Until module A is done, the maintainer also does the issues.

Status values below: `done`, `ready` (contract merged), `next` (maintainer works on it),
`blocked` (waits for a dependency), `deferred`.

## Order

```
A-1 focus spike ───────────────────────────────┐ (parallel with A-2 … A-7)
A-2 workspace ─► A-3 generated contract ─► A-9 layout rules
       ├─► A-4 xtask + AGENTS.md + Linux CI ─► A-5 guardrail checks
       ├─► A-6 probe + platform job  (needs the A-1 answer on CI runners)
       └─► A-7 UI harness

Milestone M1 — one thin slice, end to end:
  R1-4 kiosk (needs A-1) · R1-5 launch (needs A-6) · R2-1 stats ─► R2-2 gauge
  R3-1 macros (needs A-6) · R5-1 diff ─► A-8 risk + ledger ─► R6-1 MCP ─► R6-2 approve UI

Milestone M2 — breadth, open to the community:
  R2-3 media · R2-4 embed · R3-2 game watch ─► R3-3 context pages · R3-4 recent order
  R4-1 ─► R4-2, R4-3 editor · R5-2 export ─► R5-3 import · R6-3 FOR_LLMS · R6-4 inventory
```

---

## A — Architecture

### Issue A-1 — Focus and input spike (SPEC §1.2, §8.3)
- **Labels:** `verify:human`, `review:owner`, `size:M` · **Status:** next
- **Depends on:** none
- **Goal:** prove how the deck window takes a tap without taking focus, and how real input
  behaves. The result fixes the design of R1-4, R3-1 and A-6.
- **Files:** throwaway code in `spikes/focus/` (not a workspace member); the report
  `docs/spikes/A-1-focus-input.md`.
- **Questions to answer, each with the method and the observed result:**
  1. Does a Tauri 2 window with `WS_EX_NOACTIVATE` get touch taps and mouse clicks in
     WebView2, while another window stays the foreground window?
  2. Does a tap on the deck minimize or disturb a full-screen program on the other display
     (exclusive full screen and borderless)? Does the mouse cursor or mouse capture change?
  3. Do scan-code keys from `SendInput` reach the foreground window when the deck is tapped?
  4. What happens when the target runs elevated? List the options (run the app elevated,
     a `uiAccess` manifest) with their cost.
  5. How does edit mode get text input? (Expected: switch the no-activate style off.)
  6. On a hosted `windows-latest` CI runner: does `SendInput` reach a window of our own
     process, and does a WebView2 window start under `tauri-driver`?
- **Acceptance:** the report answers the six questions and ends with a decision: the window
  style for run mode and edit mode, and "probe tests on CI: yes or no". If the answer to 1 is
  no, stop and return to the architect with the options.
- **Manual steps:** the maintainer runs the spike on the machine with the touch monitor. Ask
  before starting anything on the maintainer's screen.

### Issue A-2 — Cargo workspace split (SPEC §3)
- **Labels:** `verify:auto`, `review:owner`, `size:M` · **Status:** next
- **Depends on:** PR #28 (R1-3) merged
- **Goal:** move to the workspace layout of SPEC §3 with no behavior change.
- **Files:**
  - **Create:** root `Cargo.toml` (workspace, shared lints and dependency versions);
    `crates/easydeck-core/` (from `src-tauri/src/deck/*`);
    `crates/easydeck-core/tests/deck_load.rs` (from `src-tauri/tests/deck_load.rs`);
    `tests/fixtures/` (from `src-tauri/tests/fixtures/`); empty crates
    `easydeck-platform-windows`, `easydeck-mcp`, `easydeck-cli`.
  - **Modify:** `src-tauri/Cargo.toml` (package `easydeck-app`, depends on `easydeck-core`;
    the exe name stays `easydeck`); `src-tauri/src/lib.rs`; `.github/workflows/ci.yml`.
  - **Delete:** `src-tauri/src/deck/`, `src-tauri/tests/`, `src-tauri/Cargo.lock`.
  - **Stays:** `src-tauri/src/error.rs` and, from PR #28, `src-tauri/src/ipc/`.
- **Requirements:** `easydeck-core` has no dependency on `tauri` or `windows`; `AppError`
  (the IPC error shape) stays in `src-tauri/src/error.rs` and stays JSON-compatible; core
  keeps its own error types and the app maps them; one `Cargo.lock` at the root; the
  `windows` CI job keeps its name (branch protection uses it) and runs the whole workspace; a
  new `core-linux` job runs `cargo test -p easydeck-core -p easydeck-mcp -p easydeck-cli` on
  `ubuntu-latest`. The binary of `easydeck-cli` has the name `easydeck-cli`.
  Fix the paths that the move breaks. In `validate.rs`, the `include_str!` path becomes
  `../../../../schema/deck.schema.json`. In `deck_load.rs`, the fixtures are at
  `CARGO_MANIFEST_DIR/../../tests/fixtures`, the starter deck is at
  `CARGO_MANIFEST_DIR/../../examples/starter.deck.json`, and the import is
  `easydeck_core::deck`.
  In the `windows` job, remove `working-directory: src-tauri` from the Format, Clippy, Build
  and Test steps. The steps run from the repository root: `cargo fmt --all -- --check`,
  `cargo clippy --workspace --all-targets --all-features -- -D warnings`,
  `cargo build --workspace --locked`, `cargo test --workspace --locked`. Set the rust-cache
  `workspaces` to `.`. Delete the `detect` job and the `needs` and `if` lines that use it.
  The job id `windows` does not change.
- **Acceptance:** `cargo test --workspace` runs no fewer tests than `cargo test` in
  `src-tauri` ran before the move; the PR states both counts; `npm run tauri dev` still shows
  the starter deck; both CI jobs are green.
- **Tests:** the existing ones, moved.

### Issue A-3 — Generated contract (SPEC §4.1)
- **Labels:** `verify:auto`, `review:owner`, `size:M` · **Status:** blocked
- **Depends on:** A-2
- **Goal:** the Rust deck types generate the JSON Schema and the TypeScript types.
- **Files:**
  - **Create:** `ui/src/lib/generated/` (generated); the `gen` and `validate` code in
    `crates/easydeck-cli`.
  - **Modify:** `crates/easydeck-core/src/deck/model.rs` (derive `JsonSchema` and `TS`, doc
    comments, constraints as attributes); `schema/deck.schema.json` (generated);
    `ui/src/lib/api.ts` (use the generated types; keep the `invoke` wrappers); root
    `Cargo.toml` and `crates/easydeck-core/Cargo.toml` (`schemars`, `ts-rs`, and `insta` as a
    dev dependency; SPEC §2 approves all three); `.github/workflows/ci.yml`.
- **Requirements:** `easydeck-cli gen` writes both outputs; `easydeck-cli gen --check` exits
  non-zero on drift; `cargo xtask gen` (A-4) runs this code, and contributors and CI use the
  `xtask` form after A-4; `easydeck-cli validate <files>` prints one line per error (path,
  rule, fix); the generated schema keeps the v3 limits (id pattern, string lengths, numeric
  ranges, no unknown fields); the model changes of SPEC v4 land here: `media` without a
  provider and without `volume`. `position` stays required until A-9. Use the current stable
  `schemars` and `ts-rs`, pin the exact versions in the workspace manifest, and state them
  in the PR. If `schemars` cannot express a rule, the rule moves to the semantic validator
  with a test. Two rules are known to need this: the required fields for each launch kind
  (`appId`, `path`, `target`; an `if/then` in the v3 schema), and unknown fields on a widget
  (`unevaluatedProperties` in the v3 schema; the widget type is a flattened enum). Each gets
  a fixture. Core embeds the committed `schema/deck.schema.json` with `include_str!`.
  `jsonschema` stays a run-time dependency.
- **Acceptance:** every fixture in `tests/fixtures/` gives the same verdict as before (valid
  stays valid, each invalid one fails on the same path); `examples/` validates; the CI
  `schema` job (ajv) is replaced by `easydeck-cli validate examples/*.deck.json` in
  `core-linux`.
- **Tests:** the fixture table; `insta` snapshots of the validator messages for each invalid
  fixture.

### Issue A-4 — `xtask`, `AGENTS.md`, one check command (STRATEGY §5)
- **Labels:** `verify:auto`, `size:S` · **Status:** blocked
- **Depends on:** A-2 (and A-3 for the `gen --check` step)
- **Goal:** one command runs what CI runs, and any agent tool finds the rules.
- **Files:**
  - **Create:** `xtask/`, `.cargo/config.toml` (alias), `AGENTS.md`, one first test in
    `ui/src/`.
  - **Modify:** `CLAUDE.md`, `.github/workflows/ci.yml`, `ui/package.json` (add `vitest` as
    a dev dependency and a `test` script; SPEC §2 approves it).
- **Requirements:** `cargo xtask check` runs fmt check, clippy with `-D warnings`, the
  workspace tests, `gen --check`, example validation, and in `ui/`: `npm ci`,
  `npm run check`, `npm test`, `npm run build`; `cargo xtask gen` runs the `gen` code of
  `easydeck-cli`, and `xtask` holds no generator code; `cargo xtask check --core` runs the
  Linux subset; CI calls
  `xtask` so that the two cannot drift; `AGENTS.md` holds the rules, the commands and the
  layout; `CLAUDE.md` imports `AGENTS.md` and adds only what is specific to Claude Code.
- **Acceptance:** a clean clone on Linux passes `cargo xtask check --core`; on Windows
  `cargo xtask check` passes; CI is green through `xtask`.
- **Tests:** CI itself.

### Issue A-5 — Guardrail checks in CI (SPEC §9)
- **Labels:** `verify:auto`, `review:owner`, `size:M` · **Status:** blocked
- **Depends on:** A-4
- **Goal:** the SPEC §9 table is enforced by CI.
- **Files:**
  - **Create:** `deny.toml`, `.github/CODEOWNERS`, an `xtask` source check.
  - **Modify:** `.github/workflows/ci.yml`, `.github/PULL_REQUEST_TEMPLATE.md`.
- **Requirements:** `cargo-deny` (licences, advisories, bans, sources); CI installs
  `cargo-deny` with a pinned version; `deny.toml` allows these licences: MIT, Apache-2.0,
  BSD-2-Clause, BSD-3-Clause, ISC, Zlib, Unicode-3.0, MPL-2.0; the implementer reports each
  crate that needs another licence, and the maintainer decides; a check that fails
  when `easydeck-core` depends on `tauri` or `windows`; a source check that fails on a
  process spawn (`std::process::Command`, `tokio::process`, `CreateProcessW`,
  `ShellExecuteW`) or `SendInput` outside `easydeck-platform-windows` and `easydeck-probe`,
  and on a network client (`reqwest`, `hyper`, `std::net`, `tokio::net`) outside
  `easydeck-core::sources::http_poll` and `easydeck-mcp`;
  CODEOWNERS for `crates/easydeck-platform-windows/`, `crates/easydeck-mcp/`,
  `crates/easydeck-core/src/deck/{risk,approval}.rs`, `.github/` and `deny.toml`; workflow
  actions pinned by commit SHA; the workflow uses no secrets on `pull_request`.
- **Acceptance:** a test branch that adds `Command::new` to core fails CI; a dependency with
  a non-permissive licence fails CI.
- **Tests:** the `xtask` check has unit tests with good and bad sample sources.

### Issue A-6 — Probe and the platform test job (SPEC §0 rule 9, §8.3, §8.4)
- **Labels:** `verify:auto`, `review:owner`, `size:M` · **Status:** blocked
- **Depends on:** A-2, A-1 (question 6)
- **Goal:** platform code gets automatic tests with real input, safely.
- **Files:** `crates/easydeck-probe/`, test support in `crates/easydeck-platform-windows`,
  a `platform-windows` CI job, `docs/RELEASE-CHECKLIST.md`.
- **Requirements:** the probe opens a window, takes focus, and writes to a JSON file: its
  arguments, each key event (scan code, down or up, time in ms) and a ready marker; it exits
  on a named event or after a timeout; a test helper starts the probe, waits for ready, and
  reads the file; probe tests are compiled only with the feature `probe-tests` and run only
  when `EASYDECK_PROBE_TESTS=1`; they refuse to send input when the foreground window is
  not the probe; one `tauri-driver` smoke test (app starts, deck loads); the release
  checklist holds the manual steps of SPEC §8.5. If A-1 says a hosted runner cannot do this,
  the job becomes a local command and the issue returns to the architect.
- **Acceptance:** the `platform-windows` job is green; a plain `cargo test` on a developer
  machine opens no window and sends no key.
- **Tests:** a probe self-test: start it with arguments, read them back.

### Issue A-7 — UI test harness (SPEC §8.2)
- **Labels:** `verify:auto`, `size:M` · **Status:** blocked
- **Depends on:** PR #28 (R1-3) merged
- **Goal:** UI work is proven in CI with behavior tests and screenshots.
- **Files:**
  - **Create:** `ui/playwright.config.ts`, `ui/e2e/`, `ui/src/lib/mock-ipc.ts`.
  - **Modify:** `ui/package.json`, `.github/workflows/ci.yml` (a `ui-linux` job).
- **Requirements:** Playwright runs the built UI with `mockIPC` and a fixture deck; touch
  emulation at two viewport sizes (1920x480 and 1024x600); first tests: the starter deck
  renders, a tap calls `run_action` once, text cannot be selected, the press animation class
  appears; screenshot baselines are made in the CI container only and are stored in the
  repo; CI uploads the screenshots and the diff images as artifacts. Playwright is a dev
  dependency only.
- **Acceptance:** the `ui-linux` job is green; a deliberate CSS change makes the screenshot
  test fail and the artifact shows the difference.
- **Tests:** the suite itself.

### Issue A-8 — Risk items and the approval ledger (SPEC §4.3, §6.1, §6.5)
- **Labels:** `verify:auto`, `review:owner`, `size:M` · **Status:** blocked
- **Depends on:** A-3, R5-1
- **Goal:** nothing risky runs before the user approves it.
- **Files:** `crates/easydeck-core/src/deck/{risk,approval}.rs`, `src-tauri/src/ipc`,
  `ui/src/components/RiskReview.svelte`.
- **Requirements:** `risk_items(&Deck) -> Vec<RiskItem>` for launch, key macro, shell and
  network endpoints (`http_poll` URL, `embed` URL, remote icon URL); a stable SHA-256 hash
  over a canonical form (key order and widget position do not change it; any change to what
  runs does); a flag for system-shortcut macros; `ApprovalLedger` is one set of approved
  hashes for all decks, with no deck key, stored as `approvals.json` in the app data folder
  in the form `{ "version": 1, "approved": ["<hex sha256>"] }` and written atomically; when
  `approvals.json` is corrupt, the app treats the ledger as empty, renames the bad file to
  `approvals.json.corrupt` and shows one message; `run_action` returns the typed error `needs_approval` for an
  unapproved item; sources and embeds with an unapproved endpoint do not start; the review
  screen lists items by kind and approves the selected ones; the bundled starter deck is
  approved at first start only for items that are in the build.
- **Acceptance:** a deck edited on disk to change one `exe` path shows that tile locked and
  the others working; approval unlocks it; a moved tile needs no new approval.
- **Tests:** property tests (hash is stable under reorder, changes with content); ledger
  tests (the same item in two decks needs one approval; a renamed deck keeps its approvals;
  a corrupt file gives an empty ledger and a `.corrupt` copy); a core test that an unapproved item never reaches an `ActionRunner`; Playwright test
  of the review screen.

### Issue A-9 — Layout rules (SPEC §4.2)
- **Labels:** `verify:auto`, `size:S` · **Status:** blocked
- **Depends on:** A-3
- **Goal:** auto-placement and the overlap check.
- **Files:** `crates/easydeck-core/src/layout.rs`, `deck/model.rs`, `deck/validate.rs`, the
  UI `Page` component.
- **Requirements:** `position` becomes optional in the model, and the generated files are
  made again; `place(page) -> Vec<Placed>` keeps explicit positions and puts each
  widget without one in the first free cell in reading order; a widget that does not fit
  anywhere is a validation error; overlap of two explicit positions is a validation error
  that names both widget ids; the UI draws from `Placed`, not from raw positions.
- **Acceptance:** a deck with no positions renders in reading order; the overlap fixture fails.
- **Tests:** unit and property tests (no two placed widgets overlap; all fit in the grid).

---

## R1 — App shell

### Issue R1-1 — Tauri scaffold + CI wiring · **Status:** done (PR #26)
### Issue R1-2 — Deck model, load and validate · **Status:** done (PR #27)
### Issue R1-3 — Deck renderer and launch tiles UI · **Status:** in review (PR #28)

### Issue R1-4 — Kiosk window on a chosen display (SPEC §1.2)
- **Labels:** `verify:human`, `review:owner`, `size:M` · **Status:** blocked
- **Depends on:** A-1, A-2
- **Goal:** the app window fills the display the user picks, without borders, and does not
  take focus in run mode.
- **Files:** `crates/easydeck-platform-windows/src/{display,kiosk}.rs`, a `Displays` trait
  and the display-choice function in core, app settings (`app-settings.json`, separate from
  the deck) with `displayId`.
- **Requirements:** list displays (id, name, size, position); command `list_displays`; at
  start, place a borderless window on `displayId`, or on the primary display if that id is
  gone; always-on-top is a setting; `set_display(id)` moves the window and saves the choice;
  the window style for run mode and edit mode follows the A-1 report.
- **Acceptance:** with two or more monitors, the window opens on the chosen one after a
  restart; unplugging the monitor falls back to the primary display without a crash; a tap
  leaves the foreground window unchanged.
- **Tests:** unit tests of the display-choice function with fake display lists.
- **Manual steps:** two-monitor test, unplug test, tap during a full-screen program.

### Issue R1-5 — Launch action runner (SPEC §5, §6.1)
- **Labels:** `verify:auto`, `review:owner`, `size:M` · **Status:** blocked
- **Depends on:** A-2, A-6
- **Goal:** tapping a launch tile starts the game, program or page.
- **Files:** `crates/easydeck-core/src/actions/{mod,launch}.rs` (`ActionRunner`, `Opener`
  trait, launch planning), `crates/easydeck-platform-windows/src/opener.rs`, `ipc/commands.rs`.
- **Requirements:** core turns a launch action into an `OpenRequest` (kinds `steam` →
  `steam://rungameid/<appId>`, `exe` with `fallback` when `path` does not exist, `uri`,
  `url`); the Windows `Opener` spawns an `exe` without a shell and opens URIs with the default
  handler; typed errors (`NotFound`, `SpawnFailed`); an error shows a short toast in the UI.
- **Acceptance:** each kind produces the right `OpenRequest`; a missing `exe` path uses the
  fallback; the probe test starts the probe with arguments and reads them back.
- **Tests:** fake `Opener` unit tests for each kind and for the fallback; one probe test; the
  `tauri-driver` smoke test gets a step: a tap on a launch tile starts the probe.

---

## R2 — Live widgets

### Issue R2-1 — System stats source (SPEC §5)
- **Labels:** `verify:auto`, `size:M` · **Status:** blocked
- **Depends on:** A-2
- **Goal:** a source that streams CPU load, RAM use and GPU load and temperature.
- **Files:** `crates/easydeck-core/src/sources/{mod,system_stats}.rs` (`Source`,
  `StatsReader` trait, scheduler), `crates/easydeck-platform-windows/src/stats.rs`.
- **Requirements:** `subscribe()` returns a stream of `SourceValue`; CPU and RAM through a
  crate such as `sysinfo`; GPU through `nvidia-smi --query-gpu=...` when present, else the
  metric is `null`; interval from the source config; the app pushes values to the UI as the
  Tauri event `source:<id>`.
- **Acceptance:** values arrive every interval; a missing GPU does not error; a Windows CI
  smoke test reads CPU and RAM once and gets numbers in range.
- **Tests:** parse tests for `nvidia-smi` CSV output (with a missing-GPU case); interval test
  with a fake clock.

### Issue R2-2 — Gauge, status and text widgets (SPEC §4)
- **Labels:** `verify:auto`, `size:M` · **Status:** blocked
- **Depends on:** A-7, R2-1
- **Goal:** render the `gauge`, `status` and `text` (label, clock, date) widgets.
- **Files:** `ui/src/components/{Gauge,Status,TextWidget}.svelte`, a `sourceStore` that
  subscribes to `source:<id>` events.
- **Requirements:** gauge draws an arc or bar between `min` and `max` with `unit`; status
  shows OK or problem from `okWhen`; clock updates each second and honors `format`; a
  neutral "no data" state when the source is silent.
- **Acceptance:** the starter deck shows a live CPU gauge and a ticking clock.
- **Tests:** `vitest` for value-to-angle mapping and the "no data" state; Playwright
  screenshots with a fixed clock and fixed source values.

### Issue R2-3 — Media widget on the system media session (SPEC §5)
- **Labels:** `verify:human`, `review:owner`, `size:M` · **Status:** blocked
- **Depends on:** A-2, A-7
- **Goal:** now-playing display with play/pause, next and previous, for any player.
- **Files:** `MediaSession` trait in core, `crates/easydeck-platform-windows/src/media.rs`,
  `ui/src/components/Media.svelte`.
- **Requirements:** read title, artist and thumbnail from
  `GlobalSystemMediaTransportControlsSessionManager`; follow the current session; send the
  three commands; emit changes as the event `media`; only the controls in `controls` render;
  a neutral state when no player runs. No account, no token, no network call.
- **Acceptance:** with a player running, the widget shows the track and the buttons work;
  with none, it shows the neutral state.
- **Tests:** UI tests with a fake session through mock IPC.
- **Manual steps:** play a track in two different players; check each control.

### Issue R2-4 — Web embed widget (SPEC §0 rule 6, §4)
- **Labels:** `verify:human`, `review:owner`, `size:M` · **Status:** blocked
- **Depends on:** A-8
- **Goal:** show any `http(s)` page inside a tile, with no app access for the page.
- **Files:** `ui/src/components/Embed.svelte`, embed helpers in `src-tauri/src/ipc`.
- **Requirements:** an iframe first; if a site blocks framing, a Tauri child webview over the
  tile rectangle that shares the app's WebView2 profile; the embedded page has no Tauri IPC
  and no capability; `refreshSeconds` reloads; only `http` and `https` load; the URL is a
  risk item and does not load before approval; touch scroll stays inside the tile.
- **Acceptance:** a video page and a plain dashboard page both show and respond to touch;
  logins persist across restarts; `window.__TAURI__` is absent in the embedded page.
- **Tests:** `vitest` test that rejects non-http URLs and unapproved URLs.
- **Manual steps:** both paths (iframe and child webview); the IPC-absent check.

---

## R3 — Macros and game detection

### Issue R3-1 — Key macro runner (SPEC §5, §6.1)
- **Labels:** `verify:auto`, `review:owner`, `size:L` · **Status:** blocked
- **Depends on:** R1-5, A-6, A-1, R3-2
- **Goal:** run key macros with hardware scan codes, once or in a loop.
- **Files:** `crates/easydeck-core/src/actions/{keys,macros}.rs` (parser, scheduler,
  `KeySender` trait), `crates/easydeck-platform-windows/src/keys.rs`.
- **Requirements:** parse `keys` entries (`"1"`, `"f5"`, `"ctrl+q"`) into scan codes; send
  with `SendInput` using `KEYEVENTF_SCANCODE`; honor `holdMs`, `keyGapMs`, `cycleMs`; with
  `toggle`, one tap starts a loop task and the next tap stops it; at most one loop per widget;
  all loops stop on app exit and when the page's trigger process stops; the elevated-target
  behavior follows the A-1 report and is documented. Before each cycle of a loop, the
  scheduler asks `ProcessLister::foreground_process()`. When the result is not the trigger
  process of the page, the loop pauses and the app emits `macro:<widgetId>` with `paused`.
  The same event carries `running` and `stopped`. A paused loop resumes at the next cycle
  when the foreground process matches again. On a page with no `trigger`, the scheduler
  makes no foreground check.
- **Acceptance:** the probe receives the keys in the right order, with timing inside a
  tolerance; a toggle macro starts and stops; closing the app stops all loops.
- **Tests:** unit tests for key-name parsing; scheduler tests with a fake `KeySender` and a
  fake clock; scheduler tests with a fake `ProcessLister`: pause, resume, stop when the
  trigger process exits, and no check on a page with no trigger; probe tests for order and
  timing.

### Issue R3-2 — Game watch source (SPEC §5, §6.2)
- **Labels:** `verify:auto`, `review:owner`, `size:M` · **Status:** blocked
- **Depends on:** A-2, A-6
- **Goal:** report when a watched process starts or stops.
- **Files:** `crates/easydeck-core/src/sources/game_watch.rs` (`ProcessLister` trait, the
  diff and debounce logic), `crates/easydeck-platform-windows/src/processes.rs`.
- **Requirements:** poll the process list at `intervalMs`; compare against `processes`
  case-insensitively; emit `started` / `stopped` with the process name; debounce a flapping
  process (ignore a stop shorter than one interval); `ProcessLister` also has
  `foreground_process() -> Option<String>` (the process name of the foreground window).
- **Acceptance:** starting and closing the probe (watched by its file name) produces one
  `started` and one `stopped` event.
- **Tests:** unit tests over scripted process lists, including the debounce case; one probe test.

### Issue R3-3 — Context pages (SPEC §6.2)
- **Labels:** `verify:auto`, `size:M` · **Status:** blocked
- **Depends on:** R3-2
- **Goal:** a page with a `trigger` opens when its process starts; the deck returns when it stops.
- **Files:** `crates/easydeck-core/src/pages.rs`, `ui/src/lib/pageStore.ts`.
- **Requirements:** a `PageController` holds the current page and a stack of previous pages;
  on `started`, switch; on `stopped`, return to the previous page, or `homePage`; a manual
  page change while in a context page wins until the process stops; two triggers at once:
  the latest start wins.
- **Acceptance:** the sequence start A, start B, stop B, stop A ends on the home page after
  returning through A; manual navigation is not overridden.
- **Tests:** state-machine unit tests for each sequence; a property test over random event
  orders (the controller always ends on a page that exists).

### Issue R3-4 — Recent-first ordering (SPEC §4)
- **Labels:** `verify:auto`, `size:S` · **Status:** blocked
- **Depends on:** R1-5, A-9
- **Goal:** launch tiles marked `"order": "recent"` sort by play history.
- **Files:** `crates/easydeck-core/src/history.rs`, a `history.json` in the app data folder.
- **Requirements:** record `{widgetId, timestamp}` on each successful launch; keep the last
  200 entries; sort `recent` widgets by last launch time, then by launch count in the current
  hour-of-day bucket; sorting changes the order of the `recent` tiles among their own slots
  only, never the grid positions of other widgets.
- **Acceptance:** after launching tile B then tile A, the recent group shows A first.
- **Tests:** unit tests for sorting and the tie-breaker; history trims to 200.

---

## R4 — GUI editor

### Issue R4-1 — Editor shell and edit mode (SPEC §1.1)
- **Labels:** `verify:auto`, `size:M` · **Status:** blocked
- **Depends on:** A-7, A-9
- **Goal:** a toggle that switches the deck between run mode and edit mode.
- **Files:** `ui/src/editor/{EditorShell,Palette,Inspector}.svelte`, editor store.
- **Requirements:** edit mode shows a palette of widget types, a selected-tile inspector and
  an unsaved-changes bar; the working copy is a draft in memory; every change re-validates
  through the `validate_deck` command and shows errors inline; leaving edit mode with unsaved
  changes asks to save or discard; edit mode asks the app for the edit window style (R1-4).
- **Acceptance:** entering edit mode shows the palette; adding a widget places it in the first
  free grid cell; an invalid draft cannot be saved.
- **Tests:** `vitest` for the draft store; Playwright for the mode switch and add-widget flow.

### Issue R4-2 — Launch and macro forms (SPEC §4)
- **Labels:** `verify:auto`, `size:M` · **Status:** blocked
- **Depends on:** R4-1
- **Goal:** forms that set a tile's label, icon and action without typing JSON.
- **Files:** `ui/src/editor/forms/{LaunchForm,MacroForm}.svelte`.
- **Requirements:** launch form: kind picker (steam, exe, uri, url) with only the fields for
  that kind; exe file picker through the Tauri dialog plugin; macro form: type the key list,
  or "record" by capturing the next key presses; numeric inputs with the schema's limits,
  read from the generated schema; any new action shows a "this tile can run a program / send
  keys" note.
- **Acceptance:** a user creates a Steam tile and a macro tile with no JSON typing, and the
  draft validates.
- **Tests:** `vitest` tests that each form emits JSON that passes the schema; Playwright for
  both forms.

### Issue R4-3 — Move, resize and save (SPEC §4)
- **Labels:** `verify:auto`, `size:M` · **Status:** blocked
- **Depends on:** R4-1, A-8
- **Goal:** drag tiles on the grid, resize them, and save the deck.
- **Files:** `ui/src/editor/GridEditor.svelte`, `crates/easydeck-core/src/deck/save.rs`.
- **Requirements:** pointer-based drag with snap to grid cells and no overlap (a drop on an
  occupied cell is refused); resize handle sets `w` / `h`; delete with an undo; undo and redo
  stacks; `save_deck` writes atomically (temp file, then rename) and keeps one `.bak` copy; a
  save from the editor approves the risk items the user made in that session.
- **Acceptance:** drag, resize, delete, undo and save work with touch and mouse; a failure
  during save never leaves a half-written deck.
- **Tests:** `vitest` for snap and overlap rules; Playwright drag test with touch; Rust test
  that `save_deck` is atomic.

---

## R5 — Export and import

### Issue R5-1 — Deck diff (SPEC §5, §6.3)
- **Labels:** `verify:auto`, `size:M` · **Status:** blocked
- **Depends on:** A-3
- **Goal:** compute a readable diff between two decks.
- **Files:** `crates/easydeck-core/src/deck/diff.rs`; `easydeck diff a.json b.json` in the CLI.
- **Requirements:** `diff(old, new) -> DeckDiff` listing added, removed and changed pages,
  widgets and sources; each entry has a path and a short human summary; no `apply`, since the
  approved deck replaces the old one.
- **Acceptance:** a fixture pair with one added launch tile, one moved tile and one changed
  macro yields exactly three entries.
- **Tests:** table-driven tests on fixture pairs, including identical decks (empty diff);
  `insta` snapshots of the summaries; a property test (`diff(a, a)` is empty).

### Issue R5-2 — Export a `.easydeck` pack (SPEC §4)
- **Labels:** `verify:auto`, `size:M` · **Status:** blocked
- **Depends on:** A-3
- **Goal:** save the deck and its icons as one zip file.
- **Files:** `crates/easydeck-core/src/pack/{mod,export}.rs`; `easydeck pack` in the CLI; an
  export button in the app menu.
- **Requirements:** zip with `deck.json` and `icons/`; rewrite icon references to
  pack-relative paths; copy only icons the deck uses; replace an absolute user path in an
  `exe` action by a placeholder where possible (for example `%PROGRAMFILES%`), otherwise keep
  the file name and warn; never include settings, history, the ledger or credentials.
- **Acceptance:** exporting the starter deck yields a pack that unzips to valid JSON and the
  used icons only.
- **Tests:** round-trip test (export, unzip, validate); test that a user home path never
  appears in the pack.

### Issue R5-3 — Import with preview and risk review (SPEC §0, §6.4)
- **Labels:** `verify:auto`, `review:owner`, `size:L` · **Status:** blocked
- **Depends on:** A-8, R5-2
- **Goal:** import a pack safely; the user sees what it can do before accepting.
- **Files:** `crates/easydeck-core/src/pack/import.rs`, `ui/src/components/ImportReview.svelte`.
- **Requirements:** open by file picker and by double-clicking a `.easydeck` file; reject
  path-escape entries, huge files and unknown entries; validate `deck.json`; the review
  screen shows a read-only preview of each page and the risk review of A-8; Accept stores the
  deck as a new deck (never overwrites) and writes the approved hashes; Cancel changes nothing.
- **Acceptance:** a pack with a launch, a macro, a shell action and an `http_poll` source
  shows all four in the review; a pack with `../` paths is rejected; Cancel leaves no files.
- **Tests:** path-escape and size-limit tests with a fixture pack per case; a fuzz-style
  property test on zip entry names; Playwright test of the review screen.

---

## R6 — AI layer

### Issue R6-1 — Local MCP server (SPEC §5)
- **Labels:** `verify:auto`, `review:owner`, `size:L` · **Status:** blocked
- **Depends on:** R5-1, A-8
- **Goal:** let an LLM client read the deck and propose a new one.
- **Files:** `crates/easydeck-mcp/src/{lib,server,tools,queue}.rs`.
- **Requirements:** check the current state of `rmcp` and its Streamable HTTP server; write
  the choice and the reason in the PR; bind `127.0.0.1` only, on a port from app settings;
  require a random bearer token that the app shows for the user to copy; reject a request
  whose `Origin` or `Host` header is not loopback; tools `deck_get`, `deck_schema`,
  `deck_validate`, `deck_propose` (a full new deck: validate, diff, queue); a proposal records
  the hash of the deck it was made against; the server never applies anything; validation
  errors return in the same text the CLI prints.
- **Acceptance:** a scripted MCP client connects with the token, lists the tools and calls
  each; a call without a token fails; an invalid proposal returns the errors and queues
  nothing. All of it runs on Linux CI.
- **Tests:** integration tests on a random port; listener-address test; token and `Origin`
  tests; queue tests.

### Issue R6-2 — Approve-proposal UI (SPEC §0, §6.3)
- **Labels:** `verify:auto`, `review:owner`, `size:M` · **Status:** blocked
- **Depends on:** R6-1, A-7
- **Goal:** the user sees each queued proposal and approves or rejects it.
- **Files:** `ui/src/components/ProposalReview.svelte`, `src-tauri/src/ipc`.
- **Requirements:** a badge shows queued proposals; the review screen shows the diff and puts
  new and changed risk items first; Approve replaces the deck atomically, writes the approved
  hashes and reloads; Reject drops the proposal; a proposal made against an older deck is
  stale and cannot be approved; approval comes from a tap or a click in the app, never from
  an MCP call.
- **Acceptance:** a proposal appears, shows its risk items, and applies only after Approve; a
  stale proposal is blocked.
- **Tests:** queue unit tests (stale, reject, approve); Playwright test that the Approve
  button is disabled until the risk list is on screen.

### Issue R6-3 — Write `docs/FOR_LLMS.md` (SPEC §0)
- **Labels:** `verify:human`, `size:M` · **Status:** blocked
- **Depends on:** R6-1
- **Goal:** a page that any LLM can follow to build a deck.
- **Files:** `docs/FOR_LLMS.md`, 4 or more examples in `examples/`.
- **Requirements:** keep to the outline in the stub; give the exact tool inputs and outputs;
  one worked example per widget type and per action kind; a "common mistakes" list that
  quotes real validator messages; every example also exists in `examples/` and validates in CI.
- **Acceptance:** a fresh LLM session with only this page and the MCP server builds a
  two-page deck that validates; the PR includes a summary of the transcript.
- **Tests:** CI validates each file in `examples/`.

### Issue R6-4 — Inventory tool (SPEC §5)
- **Labels:** `verify:auto`, `review:owner`, `size:M` · **Status:** blocked
- **Depends on:** R6-1
- **Goal:** an LLM can find the user's games and programs, with the user's consent.
- **Files:** `crates/easydeck-core/src/inventory.rs` (parsers),
  `crates/easydeck-platform-windows/src/inventory.rs` (the `Inventory` implementation),
  `crates/easydeck-mcp/src/tools.rs`.
- **Requirements:** parse Steam `libraryfolders.vdf` and `appmanifest_*.acf` into (name, app
  id); list Start menu shortcuts as (name, target path); the tool `system_inventory` exists
  only when the setting is on (default off); the settings screen says what the tool sends;
  the result holds no user name and no document path; the `Inventory` trait is in core and
  has a fake; the app passes the implementation to the `easydeck-mcp` server when it starts
  it; `easydeck-mcp` has no OS code; the implementer states the settings key name in the PR.
- **Acceptance:** with the setting off, the tool is not in the tool list; with it on, the
  fixture Steam library yields the expected games.
- **Tests:** parser tests on fixture files; the off-by-default test.

---

## R7 — Classifier · **Status:** deferred

R7-1, R7-2 and R7-3 (GitHub #23–#25) are not in v1. See STRATEGY §3.1 item 6. Their v3 text
is in the git history of this file.

---

## Totals

32 issues in v1: A x9, R1 x5 (3 done), R2 x4, R3 x4, R4 x3, R5 x3, R6 x4. Deferred: R7 x3.

| Proof | Issues |
|---|---|
| `verify:human` | A-1, R1-4, R2-3, R2-4, R6-3 |
| `verify:auto` | all others |
| `review:owner` | A-1, A-2, A-3, A-5, A-6, A-8, R1-4, R1-5, R2-3, R2-4, R3-1, R3-2, R5-3, R6-1, R6-2, R6-4 |
