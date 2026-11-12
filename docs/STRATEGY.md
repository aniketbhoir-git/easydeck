# EasyDeck — Strategy

> **Goal:** an open-source desktop app that turns a small touch monitor into a control deck
> that a person can build by hand or by asking their own LLM.
>
> **Scope boundary:** the app launches programs, sends key presses and shows status. It runs
> on the user's machine, for the user. It collects nothing and sends nothing except what the
> user approves.

---

## 1. Triage

### 1.1 Why this exists

Small touch monitors are cheap. The software that makes them useful is either closed, tied to
one vendor's hardware, or a pile of hand-edited config. An earlier PowerShell + browser
prototype proved the idea works: tiles that launch games, live stats, a page that appears when a
game starts. It also showed the limits: a browser window is not a device (no press feedback,
text selection, scroll bounce) and the config had no schema.

LLMs change what is cheap. Most people do not want to drag tiles for an hour. They want to say
"a page for Diablo with my macros and a CPU gauge" and get it. That works only if the LLM
edits a **data file with a schema** and a person approves the result.

### 1.2 Feature triage (MoSCoW)

**Must (v1)**
- Kiosk window on a chosen display with real touch feel, which does not take focus from a game
- Deck file with a generated JSON Schema; load, validate, render
- Launch tiles (steam, exe, uri, web) with covers and icons
- Key macros; live stats; game-aware context pages; recent-first ordering
- Media controls through the Windows system media session
- GUI editor for the basics
- Export and import of `.easydeck` packs with risk review
- Local MCP server and `docs/FOR_LLMS.md`; propose-and-approve flow

**Should (v2)**
- Web-embed widget polished (login and cookie handling)
- More sources (weather, service status) as recipes
- Volume control; per-app audio

**Could (v3)**
- A yes/no classifier for suggestions ("pin this game first at this hour")
- Community gallery of decks
- macOS and Linux

**Won't**
- Cloud accounts, sync services or telemetry
- Free-form shell commands from a deck (allow-list only)
- LLM edits to app code
- A plugin system that loads third-party code

### 1.3 Personas

1. **Gamer with a spare monitor** — wants launch tiles, macros and stats next to the game.
2. **Streamer or home-office user** — wants scene, media and status buttons.
3. **Deck author** — builds a deck once and shares the pack.
4. **LLM user** — describes the deck in words; reviews the proposal; taps approve.
5. **Token contributor** — points a coding agent at an issue and lets CI prove the result.

## 2. Architecture in one picture

```
 user ──GUI editor──┐
                    ├─► deck.json ──► validate ──► risk items ──► approval ledger
 LLM ──MCP tools────┘   (generated      (core)        (core)      (user taps approve)
   (proposes,            schema)                                        │
    never applies)                                                      ▼
                                              renderer ──► touch monitor (no focus theft)
                                                  │
                                                  ▼
                              core traits ──► platform-windows (launch, keys, stats,
                                                  process watch, media session)
```

Rule: an LLM changes the deck file only, never app code. Nothing risky runs before the user
approves it.

## 3. Key decisions

| Decision | Choice | Why |
|---|---|---|
| Shell | Tauri 2 (Rust core, WebView2) | Real window control, small binary, Rust for OS work |
| Frontend | Svelte 5 + TypeScript | Small, fast, good fit for a tile renderer |
| Code structure | Cargo workspace: core, platform-windows, mcp, cli, probe, app | Core builds and tests on Linux; OS code sits in one crate |
| Deck format | One JSON file; Rust types are the source, schema and TS types are generated | One contract, no drift; an LLM can edit it; a bad edit fails validation |
| Trust | Risk items + approval ledger | One mechanism for import, LLM proposal and a changed file |
| Sharing | `.easydeck` zip (deck JSON + icons) | Simple, no code in the pack |
| LLM access | Local MCP server (Streamable HTTP) + `FOR_LLMS.md` | Works with any LLM the user already has |
| Media | Windows system media session | Any player, no account, no token, no network |
| Platform | Windows first | Window, key and process APIs differ per OS; ship one well |
| Contribution | Test-first issues; CI is the proof | The maintainer's review time is the limit, not tokens |

### 3.1 What changed in v4, and why (2026-11-11)

1. **Focus is the first risk.** v3 gave it half a sentence. A deck that takes focus breaks
   full-screen games and sends macros to itself. A spike (A-1) now comes before the window work.
2. **Workspace split.** v3 had one crate that needs Windows and WebView2 to build. Most
   coding-agent sandboxes run Linux. Now the core, the MCP server and the CLI build on Linux.
3. **Generated contract.** v3 kept three hand-written copies of the deck shape (schema,
   Rust, TypeScript) and two validators. Now the Rust types generate the other two, and CI
   uses the app's own validator.
4. **Risk items and a ledger.** v3 reviewed actions at import and at LLM diff only. A file
   edit skipped the review, and network endpoints (`embed`, `http_poll`, remote icons) had no
   review. A shared pack could track a user through them.
5. **System media session in place of Spotify OAuth.** The OAuth design needed a client id
   from each user, a token store and network calls, for one player only.
6. **Classifier out of v1.** It had three issues and no consumer. The "is a game running"
   question is answered by `game_watch`.
7. **MCP moved earlier and got an inventory tool.** The LLM flow is what sets EasyDeck apart,
   and an LLM cannot guess a Steam app id or a program path.
8. **Layout.** `position` is optional (auto-placement) and overlap is an error. Shared packs
   meet screens of unknown shape, and LLMs write overlap errors.
9. **Real-input tests against a probe.** v3 forbade every real launch and key in tests, so
   the platform code had manual tests only. A first-party probe target makes them automatic.
10. **Labels by proof, not by model.** `tier:max` / `tier:any` named models that age in
    months. `verify:auto` / `verify:human` say what proves the work.

## 4. Safety model

A deck can run commands, send keys and call the network, so a deck is untrusted input until the
user approves it.

- Every launch, key macro, shell action and network endpoint is a risk item with a hash.
- An unapproved risk item is inert. The review screen lists items by kind.
- Shell actions run from an allow-list only.
- The MCP server binds to `127.0.0.1` only, needs a token, checks the `Origin` header and
  applies nothing without a tap in the app.
- Remote pages in an `embed` get no app access.
- Packs hold JSON and icons. No scripts, no secrets.
- No telemetry.

Contributors are also untrusted input. The app injects keys and starts programs, so:
CODEOWNERS on the platform crate, the MCP crate, the risk code and the workflows; CI for
forks runs without secrets; actions are pinned; `cargo-deny` checks each dependency change.

## 5. Contribution model

People contribute tokens: a contributor points a coding agent at an issue. The limit is the
maintainer's review time, so the design moves proof into CI.

- **Test-first issues.** Before an issue opens, the maintainer merges a contract: the types
  and the tests, marked ignored. The contributor's agent makes the tests pass.
- **One command.** `cargo xtask check` runs what CI runs. `AGENTS.md` tells any agent tool
  the rules.
- **Proof labels.** `verify:auto`: CI proves the acceptance criteria. `verify:human`: a person
  with hardware must check. `review:owner`: the change touches a CODEOWNERS path.
- **Evidence from CI.** Screenshots and test reports are CI artifacts.

Details: `docs/CONTRIBUTING-WITH-LLM.md`.

## 6. Modules and phases

| Phase | Work | Outcome |
|---|---|---|
| 0 | R0, R1-1 to R1-3 (done) | Spec, schema, scaffold, deck load, renderer |
| 1 | A-1 | Focus and input spike: the window design is proven or changed |
| 2 | A-2 to A-7 | Workspace, generated contract, `xtask`, guardrails, probe, UI harness |
| 3 | Milestone M1 | One thin slice: a launch, a macro, a gauge, an LLM proposal with approval |
| 4 | Milestone M2 | Remaining widgets, editor, packs; open to the community |

## 7. Risks and mitigations

| Risk | Mitigation |
|---|---|
| The deck takes focus from the game | Spike A-1 first; no-activate window in run mode |
| Macros fail in games that run elevated | A-1 measures it; document; offer a "run as admin" setting |
| A game's anti-cheat rejects injected keys | Document it; the user owns the choice to use a macro |
| A shared deck runs something harmful or tracks the user | Risk items, approval ledger, allow-list, no scripts in packs |
| A contributed change hides harmful code | CODEOWNERS, guardrail checks, `cargo-deny`, small pull requests |
| Unsigned binaries trigger SmartScreen | Open question 2 |
| Embeds break on login or cookies | Use the app's WebView2 profile; keep the embed widget generic |
| Schema churn breaks old decks | Free changes before the first release; migrations after it |
| Solo maintainer, slow cadence | Test-first issues that CI can prove |

## 8. Open questions

1. Does a hosted Windows CI runner accept `SendInput` and a WebView2 window? (A-1 answers.)
2. Code signing for releases: cost and method.
3. `rmcp` or a hand-written server, after a check of the crate's current state (R6-1).
