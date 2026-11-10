# EasyDeck — LLM Implementation Specification

> **Purpose:** a hand-off spec for a coding LLM. It states how the app works and how the code
> should be structured and verified. It is safe to paste into any model: it describes an
> independent, open-source desktop app that launches programs, sends key presses and shows
> status on a touch monitor, using public, documented interfaces.
>
> Status: v3, module R0 (define). Product decisions in §1 to §5 are settled. Work packages
> in §7 map to the issues in `docs/ISSUE-BACKLOG.md`.

---

## 0. Ground rules (give these to the coding LLM)

1. **The LLM edits the deck file only, never app code.** Every deck change made by an LLM is a
   diff that the user approves in the app. A change that fails schema validation applies nothing.
2. **Show every action before it runs.** A deck from a file or an LLM can launch programs,
   send key macros and run shell commands. The app lists each new action and asks the user
   before it accepts the deck.
3. **Shell actions use an allow-list only.** No free-form command line from a deck.
4. **Local only.** The MCP server binds to `127.0.0.1`. No telemetry. The only network calls
   are those a widget or source the user configured makes (weather, Spotify, a web embed, the
   classifier if the user turns it on).
5. **No secrets in a deck.** API keys live in the OS credential store or an `.env` file that
   is never exported or committed. A `.easydeck` pack contains the deck JSON and icons only.
6. **Original branding.** Do not bundle third-party logos or art. Game cover art loads at run
   time from the user's own installed store client or a public endpoint.
7. Stack is fixed by §2. Ask before adding a dependency.

## 1. Product

A desktop app that turns a small touch monitor into a dynamic deck. It shows launch tiles,
macros, live stats and media widgets. A user builds the deck in a simple GUI, or tells an
LLM what they want and lets the LLM build the whole deck. Decks export as files that other
people import and change.

Prior art: an earlier PowerShell + browser prototype proved the idea. It had a local server,
one HTML page in a browser app window, a hand-edited JSON config, live stats, weather,
Spotify now-playing, a game page that opened when a game started, and timed key macros. What
carries over: data-driven launch types (`steam`, `exe` with fallback, `uri`, `web`), Steam
cover art by app id, GPU/CPU/RAM stats, game-triggered pages, key loops with hardware scan
codes, and a recent-launch record. What EasyDeck fixes: the browser feel (no press feedback,
text selection, scroll bounce, a visible browser window), the hard-coded game page, the
schema-less config, and the cookie and login limits of embeds in a plain browser.

### 1.1 Layers

| Layer | Job |
|---|---|
| App (Rust) | Window on the chosen display, kiosk mode, touch input, run actions, poll sources, detect games |
| Deck file | Pages, layout, widgets, actions, sources. JSON, checked by `schema/deck.schema.json` |
| GUI editor | Basic edit in the app: add tile, set launch, record or type a macro, move, resize |
| AI layer | Local MCP server + `docs/FOR_LLMS.md`. An LLM reads the deck, proposes a diff, the user approves |
| Classifier | Cheap yes/no gate (Jev or a local model) for suggestions and context switches |

### 1.2 Touch feel

The window is a borderless kiosk on a display the user picks (by monitor id, saved in the app
settings, not the deck). The UI shows press animations on every tile, disables text
selection and context menus, blocks overscroll and pinch zoom, and sizes hit targets for a
finger (at least 48 px).

## 2. Locked technology decisions

- Rust stable (MSVC), Tauri 2, WebView2. Frontend: Svelte 5 + TypeScript (Vite).
- tokio for async; serde / serde_json; `jsonschema` crate for validation; reqwest for HTTP.
- windows-rs for display enumeration, process watch, key injection (`SendInput`, scan codes).
- MCP server: the official Rust MCP SDK (`rmcp`) or a small hand-written JSON-RPC server over
  localhost HTTP/SSE. Decide in issue R6-1.
- Windows 10/11 only for v1. macOS and Linux are out of scope.

## 3. Repository layout (the LLM must create these paths)

```
src-tauri/            Rust crate (Tauri app + core)
  src/main.rs lib.rs error.rs
  src/deck/{model,load,validate,diff}.rs
  src/actions/{mod,launch,macros,shell}.rs
  src/sources/{mod,system_stats,http_poll,game_watch}.rs
  src/display/{mod,kiosk}.rs
  src/mcp/{mod,tools}.rs
  src/classifier/{mod,jev,local}.rs
  src/ipc/{commands,events,state}.rs
  tests/fixtures/
ui/                   Svelte frontend (deck renderer, editor)
schema/deck.schema.json
docs/                 SPEC, STRATEGY, backlog, FOR_LLMS, contributor guides
examples/             Sample decks and .easydeck packs
.github/              CI, issue and PR templates
```

## 4. Deck model

The deck file is JSON. The authoritative shape is `schema/deck.schema.json`. In words:

- **Deck:** `version`, `name`, `pages[]`, `sources{}`, optional `settings`.
- **Page:** `id`, `title`, optional `trigger` (a context rule such as "game process X is
  running"), `grid` (columns, rows), `widgets[]`.
- **Widget:** `id`, `type`, position and size on the grid, plus type-specific fields.
  v1 types: `launch`, `macro`, `toggle`, `gauge`, `status`, `text` (clock or label),
  `media` (Spotify), `embed` (web URL).
- **Action:** `launch` (kinds `steam`, `exe`, `uri`, `url`; `exe` may have a `fallback`),
  `key_macro` (`keys`, `cycleMs`, `keyGapMs`, `holdMs`), `shell` (allow-list id plus args),
  `page` (switch page).
- **Source:** named data feed. Kinds `system_stats`, `http_poll` (url, interval, JSON path),
  `game_watch` (process names to watch).
- **Pack (`.easydeck`):** a zip with `deck.json` and `icons/`.

Recent-first ordering is not stored in the deck. The app keeps a local play-history file and
sorts widgets that carry `"order": "recent"` at render time.

## 5. Service contracts (traits; signatures only)

- `ActionRunner`: `run(&Action) -> Result<()>`; one implementation per action kind.
- `Source`: `subscribe() -> Stream<SourceValue>`; implementations for the three kinds.
- `DeckStore`: `load`, `save`, `validate(&Value) -> Vec<ValidationError>`, `diff(old, new)`.
- `Classifier`: `async fn classify(&self, question: &str, context: &Value) -> Result<Verdict>`
  where `Verdict { yes: bool, probability: f32 }`. Implementations: `JevClassifier`
  (Vercel AI Gateway model `typesafe-ai/jev`, API key from env, never the short-lived OIDC
  token) and `LocalClassifier` (a local model endpoint). The AI SDK call is TypeScript only,
  so capture the HTTP request shape first and call it with reqwest. Check pricing.
  The classifier is off by default.
- MCP tools (local): `deck.get`, `deck.schema`, `deck.validate`, `deck.propose_diff`.
  `propose_diff` queues a diff. Apply needs a tap in the app.

## 6. Data-flow walkthroughs

### 6.1 Launch tile tap
UI sends `run_action(widget_id)` -> core looks up the action -> `ActionRunner` runs it ->
success or a typed error returns to the UI -> a play-history record is written for game tiles.

### 6.2 Game context page
`game_watch` source reports a watched process starting -> core finds the page whose `trigger`
matches -> UI switches to it. When the process stops, the UI returns to the previous page.

### 6.3 LLM edit
LLM calls `deck.schema` and `deck.get` -> proposes a diff via `deck.propose_diff` -> core
validates the result -> the app shows a diff view with every new or changed action ->
the user approves or rejects -> on approve, the deck file is replaced atomically and the
UI reloads. A rejected or invalid diff changes nothing.

### 6.4 Import
User opens a `.easydeck` -> core validates `deck.json` -> the app shows a preview and the
list of actions the deck can run -> the user accepts -> the deck is stored as a new deck.

## 7. Work packages

| Module | Content |
|---|---|
| R0 Define | This spec, deck schema draft, docs, backlog, CI |
| R1 App shell | Tauri scaffold, window on a chosen display, kiosk, touch feel, deck load + validate, launch tiles |
| R2 Live widgets | System stats, HTTP status, clock, Spotify, web embed |
| R3 Macros + game detection | Key macros, process watch, context pages, recent-first order |
| R4 GUI editor | Add, move, edit tiles in the app |
| R5 Export and import | `.easydeck` packs with preview and action review |
| R6 AI layer | MCP server, `FOR_LLMS.md`, approve-diff flow |
| R7 Classifier | `Classifier` trait, Jev client, local client |

Order: R1 first. R2, R3, R4 and R5 then run in parallel. R6 needs R1 and the diff code from
R5-1. R7 needs R6 only for its first consumer, so it can start early.

Out of v1: usage-based auto-suggestions, a community deck gallery, macOS and Linux.

## 8. Testing and QA strategy

- Pure logic (schema validation, diff, grid layout, parsers) has unit tests with fixture
  decks in `src-tauri/tests/fixtures/`.
- Actions run behind traits; tests use fakes. No test launches a real program or sends real
  keys. A manual Windows test checklist sits in each issue marked `needs-windows-test`.
- HTTP code uses `httpmock`. No test calls a real network service.
- Every sample deck in `examples/` must pass the schema in CI.
- UI components get `vitest` tests for state and a screenshot or short clip in the PR.

## 9. Guardrail and review checklist (run before every merge)

- [ ] LLM and import paths cannot apply a deck without user approval
- [ ] every new action kind is listed in the action review screen
- [ ] shell actions go through the allow-list only
- [ ] the MCP server binds to localhost only
- [ ] no secrets in decks, packs, logs or fixtures
- [ ] no telemetry; network calls only for sources the user configured
- [ ] no third-party marks or assets bundled
- [ ] `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt --check` pass
