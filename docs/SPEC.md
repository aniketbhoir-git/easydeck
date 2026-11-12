# EasyDeck — Implementation Specification

> **Purpose:** the hand-off spec for a coding agent. It states how the app works and how the
> code is structured and verified. It describes an independent, open-source desktop app that
> launches programs, sends key presses and shows status on a touch monitor, using public,
> documented interfaces.
>
> Status: **v4** (architecture review, 2026-11-11). v4 replaces v3. The changes and their
> reasons are in `docs/STRATEGY.md` §3.1. Work packages in §7 map to `docs/ISSUE-BACKLOG.md`.

---

## 0. Ground rules

1. **An LLM proposes a new deck file only. It never changes app code.** The user approves
   each proposal in the app. A proposal that fails validation changes nothing.
2. **No risk item runs before the user approves it.** A deck can launch programs, send key
   macros, run shell commands and call the network. Each of these is a *risk item* (§4.3).
   An unapproved risk item is inert. This holds for an import, an LLM proposal and a deck
   file that changed on disk.
3. **Shell actions use an allow-list only.** No free-form command line from a deck.
4. **Local only.** The MCP server binds to `127.0.0.1`, needs a random bearer token that the
   app shows for the user to copy, and rejects a request whose `Origin` or `Host` header is
   not loopback. No telemetry. The only network calls are the ones an approved risk item
   makes (an `http_poll` source, an `embed`, a remote icon).
5. **No secrets in a deck.** A `.easydeck` pack contains the deck JSON and icons only.
6. **Remote content never gets app access.** A page in an `embed` widget has no Tauri IPC and
   no capabilities. An embed is an iframe, or a Tauri child webview when the site blocks
   framing. It shares the app's WebView2 profile, so logins persist and all embeds share
   cookies. Approval of an `embed` URL covers the start address only. It does not limit
   where the page goes next.
7. **Original branding.** Do not bundle third-party logos or art. Game cover art comes from
   the user's local store client cache at run time.
8. **The stack is fixed by §2.** Ask before adding a dependency.
9. **Real input goes to the probe only.** An automated test may launch a program or send a
   key only when the target is the first-party probe (§8.3), and only in the opt-in platform
   job. A plain `cargo test` never does either.

## 1. Product

A desktop app that turns a small touch monitor into a dynamic deck. It shows launch tiles,
macros, live stats and media controls. A user builds the deck in a simple GUI, or tells an
LLM what they want and lets the LLM build the whole deck. Decks export as files that other
people import and change.

Prior art: an earlier PowerShell + browser prototype proved the idea (launch types, Steam
covers, stats, a game page, key loops with scan codes). EasyDeck fixes its limits: the browser
feel, the hard-coded game page and the schema-less config.

### 1.1 Layers

| Layer | Job |
|---|---|
| Core (Rust, no OS code) | Deck model, validation, diff, risk items, approvals, layout, page controller, macro scheduler, packs |
| Platform (Rust, Windows) | Display list, kiosk window, launch, key injection, process watch, media session |
| App (Tauri) | Window, IPC commands, wiring of core and platform |
| UI (Svelte) | Deck renderer, editor, review screens |
| AI layer | Local MCP server + `docs/FOR_LLMS.md`. An LLM reads the deck and proposes a new one |
| CLI | `easydeck-cli validate`, `diff`, `pack`: the same core code, for CI and for contributors |

### 1.2 Touch feel and focus

The window is a borderless kiosk on a display the user picks (by monitor id, saved in the app
settings, not the deck). The UI shows press animations on every tile, disables text selection
and context menus, blocks overscroll and pinch zoom, and sizes hit targets for a finger (at
least 48 px).

**The deck must not take focus from the game.** A tap on a tile must leave the foreground
window as it is. If the deck takes focus, a full-screen game can minimize, and a key macro
goes to the deck instead of the game. This is the largest technical uncertainty in the
product. Issue A-1 (a spike) tests the window design before R1-4 builds it. Edit mode may
take focus, because it needs text input.

### 1.3 Goals and non-goals

Goals for v1: the "Must" list in `docs/STRATEGY.md` §1.2.

Non-goals for v1:

- An app for macOS or Linux.
- Free-form shell commands from a deck.
- The suggestion classifier and usage-based suggestions.
- A community deck gallery.

### 1.4 Risks and limits

| Risk | What the project does |
|---|---|
| The deck takes focus from the game | Spike A-1 tests the no-activate window before R1-4 builds it. |
| A looping macro sends keys to the wrong window | The scheduler checks the foreground process before each cycle (§6.6). |
| A game that runs elevated drops the macro keys | A-1 measures it. R3-1 documents the behavior. |
| A game's anti-cheat rejects injected keys | The docs say so. The user decides to use a macro. |
| A shared deck runs something harmful or tracks the user | Risk items and the approval ledger (§4.3). |
| An unsigned binary triggers SmartScreen | Open question 2 in `docs/STRATEGY.md` §8. |
| An embed breaks on login or cookies | The embed shares the app's WebView2 profile. |

A corrupt `approvals.json`: the app treats the ledger as empty, so every risk item is inert.
It renames the bad file to `approvals.json.corrupt` and shows one message.

Not specified yet: the pack size limits (§6.4) and the behavior when the saved display is
missing.

## 2. Locked technology decisions

- Rust stable (MSVC), Tauri 2, WebView2. Frontend: Svelte 5 + TypeScript (Vite).
- tokio for async; serde / serde_json; `jsonschema` for validation; reqwest for HTTP.
- `schemars` generates the JSON Schema from the Rust deck types. `ts-rs` generates the
  TypeScript types. Neither file is edited by hand (§4.1).
- windows-rs for display enumeration, process watch, key injection (`SendInput`, scan codes)
  and the system media session.
- MCP server: the `rmcp` crate with the Streamable HTTP transport. Check the crate's current
  state in R6-1; a small hand-written JSON-RPC server is the fallback.
- Test tools: `proptest`, `insta` (snapshots), `httpmock`, `vitest`, Playwright,
  `tauri-driver`. Supply chain: `cargo-deny`.
- Windows 10/11 only for v1. macOS and Linux are out of scope for the app. The core, the MCP
  server and the CLI must build and pass their tests on Linux, so that any agent sandbox can
  work on them.

### 2.1 Alternatives considered

Spec v3 used the rejected options below. The full reasons are in `docs/STRATEGY.md` §3.1.

| Decision | Rejected option | Reason |
|---|---|---|
| Code structure | One crate | It needs Windows and WebView2 to build. Most agent sandboxes run Linux. |
| Deck contract | Hand-written schema, Rust types and TypeScript types, with two validators | The three copies drift. |
| Trust | Review at import and at the LLM diff only | A file edit skipped the review. Network endpoints had no review. |
| Media | Spotify OAuth | It needs a client id from each user, a token store and network calls, for one player. |
| Layout | A required `position` | A shared pack meets screens of unknown shape. LLMs write overlap errors. |
| Game detection | A suggestion classifier | It had three issues and no consumer. `game_watch` answers the question. |
| Platform tests | No real launch or key in a test | The platform code had manual tests only. |

Not compared yet:

- MCP transport: stdio or Streamable HTTP. R6-1 records the choice and the reason.
- Focus: the fallback if the no-activate window fails. A-1 reports it.

## 3. Repository layout

A Cargo workspace at the repository root.

```
Cargo.toml                    workspace
crates/
  easydeck-core/              deck/{model,validate,diff,risk,approval,migrate}.rs
                              layout.rs  pages.rs  history.rs  pack/
                              actions/{mod,keys,macros}.rs   traits + pure logic
                              sources/{mod,http_poll}.rs     the only reqwest user in core
  easydeck-platform-windows/  display, kiosk, opener, key sender, process lister,
                              system stats, media session   (windows-rs lives here only)
  easydeck-mcp/               server, tools, proposal queue
  easydeck-cli/               validate, diff, pack, gen
  easydeck-probe/             test target: records the launches and keys it receives
src-tauri/                    easydeck-app: Tauri shell, ipc/{commands,events,state}.rs
xtask/                        `cargo xtask check | gen | ...`
ui/                           Svelte frontend; src/lib/generated/ is generated
schema/deck.schema.json       generated from easydeck-core
tests/fixtures/               valid and invalid decks, shared by all crates
examples/                     sample decks and .easydeck packs
docs/                         SPEC, STRATEGY, backlog, FOR_LLMS, contributor guides
.github/                      CI, CODEOWNERS, issue and PR templates
```

Dependency rules, checked in CI (§9):

- `easydeck-core` depends on no Tauri crate and no `windows` crate.
- Only `easydeck-platform-windows` and `easydeck-probe` call the OS to spawn a process or
  send input.
- Network clients exist only in `easydeck-core::sources::http_poll` and `easydeck-mcp`.

## 4. Deck model

The deck file is JSON. In words:

- **Deck:** `version`, `name`, `pages[]`, `sources{}`, optional `settings`.
- **Page:** `id`, `title`, optional `trigger` (a context rule such as "game process X is
  running"), `grid` (columns, rows), `widgets[]`.
- **Widget:** `id`, `type`, optional `position` on the grid, plus type-specific fields.
  v1 types: `launch`, `macro`, `toggle` (a tile with an `onAction` and an `offAction`), `gauge`, `status`, `text` (clock, date or label),
  `media` (the system media session), `embed` (web URL).
- **Action:** `launch` (kinds `steam`, `exe`, `uri`, `url`; `exe` may have a `fallback`),
  `key_macro` (`keys`, `cycleMs`, `keyGapMs`, `holdMs`, `toggle`; with `toggle` true, one tap
  starts a loop and the next tap stops it), `shell` (allow-list id plus args), `page` (switch
  page).
- **Source:** named data feed. Kinds `system_stats`, `http_poll`, `game_watch`.
- **Pack (`.easydeck`):** a zip with `deck.json` and `icons/`.

Recent-first ordering is not stored in the deck. The app keeps a local play-history file and
sorts widgets that carry `"order": "recent"` at render time.

### 4.1 One contract, generated

The Rust types in `easydeck-core::deck::model` are the single source of the deck contract.
The committed `schema/deck.schema.json` is still the hand-written file of spec v3. Work
package A-3 replaces it.

- `cargo xtask gen` writes `schema/deck.schema.json` (through `schemars`) and
  `ui/src/lib/generated/` (through `ts-rs`). The `gen` code is in `easydeck-cli`, and
  `cargo xtask gen` runs it. Contributors and CI use the `xtask` form only.
- `cargo xtask gen --check` fails when a generated file differs from the committed one. CI
  runs it.
- At run time, core embeds the committed `schema/deck.schema.json` with `include_str!`.
  `jsonschema` stays a run-time dependency.
- Binary names: the app is `easydeck` (package `easydeck-app`). The CLI is `easydeck-cli`.
- Doc comments on the Rust types become the schema descriptions. Write them for an LLM
  reader: they are part of the product.
- One validator. CI checks `examples/` with `easydeck-cli validate`, the same code the app runs.
- Validator messages are the feedback an LLM gets. Each message names the path, the rule and
  the fix. Snapshot tests (`insta`) hold the messages stable.
- Until the first public release, `version` stays `1` and breaking changes are allowed. From
  the first release, each format change adds a migration function and a fixture.

### 4.2 Layout rules

- `position` is optional. A widget without one is placed in the first free cell, in reading
  order, by `easydeck-core::layout`. This makes a deck easy for an LLM to write and lets a
  pack fit a screen with another shape.
- Two widgets must not overlap. A widget must fit inside the grid. Both are validation errors.

### 4.3 Risk items and approval

A *risk item* is a part of a deck that can act outside the deck window:

| Risk item | Example |
|---|---|
| Launch action | start `C:/Games/x.exe`, open `steam://rungameid/440` |
| Key macro | send `1`, `2`, `ctrl+q` every 3 s |
| Shell action | allow-list id `lock-screen` with args |
| Network endpoint | an `http_poll` URL, an `embed` URL, a remote icon URL |

- `easydeck-core::deck::risk` lists the risk items of a deck. Each item has a stable
  SHA-256 hash over a canonical form of the item. Key order and widget position do not change
  the hash. A change to what runs changes it.
- The app keeps an approval ledger (`approvals.json`, in the app data folder, never in a
  deck or a pack). It holds one set of approved hashes for all decks. The ledger has no deck
  key. A renamed or imported deck does not lose approvals and does not get approvals by its
  name. One approval covers an item with the same content in every deck.
- A risk item whose hash is not in the ledger is inert: the tile shows a lock badge and a tap
  opens the review screen. The app does not call an unapproved endpoint.
- The review screen lists items by kind, and approves the items that the user selects. It marks a key macro that uses the Windows key or
  `alt+tab` as a system shortcut. It marks a shell id that is not in the local allow-list as
  unavailable.
- When the user saves an item in the GUI editor, the app adds its hash to the ledger.
- The same mechanism serves the import, the LLM proposal and a deck file that another
  program changed on disk.
- The ledger protects against untrusted deck content: a pack, an LLM proposal, or a deck file
  that a sync tool or an editor changed. It does not protect against a local program that can
  write the app data folder.

## 5. Service contracts (traits; signatures only)

- `ActionRunner`: `run(&Action) -> Result<()>`; one implementation per action type.
- Platform traits, defined in core and implemented in `easydeck-platform-windows`:
  `Opener`, `KeySender`, `ProcessLister`, `StatsReader`, `MediaSession`, `Displays`, `Clock`,
  `Inventory`. `ProcessLister` also reports the foreground process. Core tests use fakes.
- `Source`: `subscribe() -> Stream<SourceValue>`; implementations for the three kinds.
- `DeckStore`: `load`, `save` (atomic), `validate`, `diff(old, new)`.
- `ApprovalLedger`: `is_approved(item)`, `approve(items)`.
- `MediaSession`: current title, artist and thumbnail, plus `play_pause`, `next`, `previous`,
  through `GlobalSystemMediaTransportControlsSessionManager`. It works with any player that
  reports to Windows. It needs no account, no token and no network call.
- MCP tools (local): `deck_get`, `deck_schema`, `deck_validate`, `deck_propose`,
  `system_inventory`. Tool names use `_` because some clients reject a dot.
  The port comes from the app settings.
  - `deck_propose` takes a full new deck, validates it, computes the diff and queues it. The
    proposal records the hash of the deck it was made against. The deck file changes only
    after the user approves the proposal in the app. An MCP call cannot approve.
  - `system_inventory` returns installed Steam games (name, app id) and Start menu programs
    (name, path), so that an LLM can write correct launch actions. It is off until the user
    turns it on, because it sends local data to the LLM client. It reads them through the
    `Inventory` platform trait, so `easydeck-mcp` has no OS code.

## 6. Data-flow walkthroughs

### 6.1 Tile tap

UI sends `run_action(widget_id)` -> the app looks up the action -> the ledger check passes, or
the app returns `needs_approval` -> `ActionRunner` runs it -> success or a typed error returns
to the UI -> a play-history record is written for launch tiles.

### 6.2 Game context page

`game_watch` reports a watched process starting -> `pages::PageController` finds the page whose
`trigger` matches -> UI switches to it. When the process stops, the UI returns to the previous
page, or to `homePage`. A manual page change in a context page wins until the process stops.
When two triggers match, the latest start wins.

### 6.3 LLM edit

LLM calls `deck_schema`, `deck_get` and, when it is on, `system_inventory` -> sends a full new
deck to `deck_propose` -> core validates it and computes the diff -> the app shows the diff,
with new and changed risk items first -> the user approves or rejects -> on approve, the app
replaces the deck file atomically, adds the approved hashes to the ledger and reloads the UI.
A proposal is stale when the deck changed after the proposal was made. The app does not let
the user approve a stale proposal. A rejected, invalid or stale proposal changes nothing.

### 6.4 Import

User opens a `.easydeck` -> core checks the zip (no path escape, size limits, known entries
only) and validates `deck.json` -> the app shows a preview and the risk items -> the user
approves -> the app stores the deck as a new deck and adds the approved hashes to the ledger.

### 6.5 Deck file changed on disk

The app loads a deck file -> core lists its risk items -> the app compares each hash with the
ledger -> an item that the ledger does not hold stays inert until the user reviews it. This
check runs at each load, so a deck file that another program changed gets it on the next
load. The app does not watch the deck file in v1.

### 6.6 Key macro

UI sends `run_action` -> the ledger check passes -> the macro scheduler changes the `keys`
entries into scan codes -> `KeySender` sends them with `SendInput` to the foreground window.
With `toggle`, one tap starts a loop and the next tap stops it. At most one loop runs for
each widget. Before each cycle, the scheduler asks `ProcessLister` for the foreground process.
When that process is not the trigger process of the page, the scheduler pauses the loop and
the tile shows a paused state. A paused loop resumes at the next cycle when the foreground
process matches again. On a page with no `trigger`, the scheduler makes no foreground check.
All loops stop when the app exits and when the trigger process of the page stops.

## 7. Work packages

| Module | Content |
|---|---|
| R0 Define | Spec, schema draft, docs, backlog, CI (done) |
| A Architecture | Focus spike, workspace split, generated contract, `xtask`, guardrail checks, probe, UI harness, approval ledger, layout rules |
| R1 App shell | Scaffold, deck load + validate (done); renderer (R1-3, in review); kiosk window, launch runner |
| R2 Live widgets | System stats, gauge / status / text, system media session, web embed |
| R3 Macros + game detection | Key macros, process watch, context pages, recent-first ordering |
| R4 GUI editor | Add, move, resize, edit, delete and save tiles in the app |
| R5 Export and import | Deck diff, `.easydeck` packs with preview and the review screen |
| R6 AI layer | MCP server, proposal review, `FOR_LLMS.md`, the `system_inventory` tool |

Order: A-1 to A-7 first, then milestone M1 (one thin slice end to end: R1-4, R1-5, R2-1,
R2-2, R3-1, R5-1, A-8, R6-1, R6-2), then milestone M2. A-9 follows A-3. The issue order is in
`docs/ISSUE-BACKLOG.md`. Out of v1: see §1.3.

## 8. Testing strategy

The aim: a pull request needs no manual test, unless its issue carries `verify:human`.

### 8.1 Core, MCP and CLI (Linux and Windows CI)
- Unit tests with fixture decks in `tests/fixtures/`.
- Property tests (`proptest`) for validation, diff, layout and the risk hash.
- Snapshot tests (`insta`) for validator messages and diff summaries.
- State-machine tests for the page controller and the macro scheduler, with a fake clock.
- HTTP code uses `httpmock`. No test calls a real network service.
- MCP: integration tests start the server on a random loopback port and use a scripted client.

### 8.2 UI (Linux CI)
- `vitest` for stores and components.
- Playwright runs the built UI in a browser with a mock IPC layer (`@tauri-apps/api/mocks`)
  and touch emulation. It checks behavior and compares screenshots. CI makes the screenshots
  and uploads them; a contributor does not attach them by hand.

### 8.3 Platform (Windows CI, opt-in job)
- `easydeck-probe` is a small first-party program. It writes the arguments it starts with and
  the key events it receives (scan code, down or up, time in ms) to a file.
- Platform tests start the probe through the real `Opener` and send keys to it through the
  real `KeySender`. They are compiled only with the `probe-tests` feature and run only when
  `EASYDECK_PROBE_TESTS=1`, in the `platform-windows` job. A-1 checks that a hosted runner
  can send input to the probe.

### 8.4 End to end (Windows CI)
- One smoke test through `tauri-driver`: the app starts and loads a deck (A-6). R1-5 adds a
  step: a tap on a launch tile starts the probe.

### 8.5 Manual, once per release
Real touch hardware, a monitor unplug, a game that runs elevated, a full-screen game with the
deck on the second display. The checklist is `docs/RELEASE-CHECKLIST.md` (written in A-6).

## 9. Guardrails (checked by CI)

| Rule | Check |
|---|---|
| Core has no OS or Tauri code | dependency check in `cargo xtask check` |
| Only `easydeck-platform-windows` and `easydeck-probe` spawn a process or send input | the `xtask` source check |
| Network clients only where §3 allows | the `xtask` source check |
| MCP binds to loopback, needs the token, checks the `Origin` and `Host` headers | integration tests in `easydeck-mcp` |
| An unapproved risk item never runs | core tests that an unapproved item never reaches an `ActionRunner` |
| Import and proposal cannot write a deck without approval | core and MCP tests |
| No secrets or user paths in a pack | pack round-trip test |
| Licenses, advisories, dependency sources | `cargo-deny` |
| Generated files are current | `cargo xtask gen --check` |
| Format and lint | `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `npm run check` in `ui/` |

A reviewer checks: no telemetry, no third-party logos or art, and that a new action type or
endpoint kind is a risk item in `deck::risk`.
