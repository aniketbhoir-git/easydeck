# EasyDeck — Strategy

> **Goal:** an open-source desktop app that turns a small touch monitor into a control deck
> that a person can build by hand or by asking their own LLM.
>
> **Scope boundary:** the app launches programs, sends key presses and shows status. It runs
> on the user's machine, for the user. It collects nothing and sends nothing except what the
> user configures.

---

## 1. Triage

### 1.1 Why this exists

Small touch monitors are cheap. The software that makes them useful is either closed, tied to
one vendor's hardware, or a pile of hand-edited config. An earlier PowerShell + browser
prototype proved the idea works: tiles that launch games, live stats, a page that appears when a
game starts. It also showed the limits: a browser window is not a device (no press feedback,
text selection, scroll bounce), config had no schema, and embeds such as Spotify hit cookie
and login limits.

LLMs change what is cheap. Most people do not want to drag tiles for an hour. They want to say
"a page for Diablo with my macros and a CPU gauge" and get it. That works only if the LLM
edits a **data file with a schema** and a person approves the result.

### 1.2 Feature triage (MoSCoW)

**Must (v1)**
- Kiosk window on a chosen display with real touch feel
- Deck file with a JSON Schema; load, validate, render
- Launch tiles (steam, exe, uri, web) with covers and icons
- Key macros; live stats; game-aware context pages; recent-first ordering
- GUI editor for the basics
- Export and import of `.easydeck` packs with action review
- Local MCP server and `docs/FOR_LLMS.md`; approve-diff flow

**Should (v2)**
- Spotify and web-embed widgets polished (login and cookie handling)
- `Classifier` trait with Jev and local implementations
- More sources (weather, service status) as recipes

**Could (v3)**
- Usage-based suggestions ("pin this game first at this hour")
- Community gallery of decks
- macOS and Linux

**Won't**
- Cloud accounts, sync services or telemetry
- Free-form shell commands from a deck (allow-list only)
- LLM edits to app code

### 1.3 Personas

1. **Gamer with a spare monitor** — wants launch tiles, macros and stats next to the game.
2. **Streamer or home-office user** — wants scene, media and status buttons.
3. **Deck author** — builds a deck once and shares the pack.
4. **LLM user** — describes the deck in words; reviews the diff; taps approve.

## 2. Architecture in one picture

```
 user ──GUI editor──┐
                    ├─► deck.json (schema-checked) ──► renderer ──► touch monitor
 LLM ──MCP tools────┘        ▲                              │
   (propose diff,            │ user approves                ▼
    never applies)           └────────────────────── actions / sources
                                                     (launch, keys, stats, game watch)
 Classifier (optional): yes/no gate for suggestions and context switches
```

Rule: an LLM changes the deck file only, never app code. Every LLM change is a diff the user
approves.

## 3. Key decisions

| Decision | Choice | Why |
|---|---|---|
| Shell | Tauri 2 (Rust core, WebView2) | Real window control, small binary, Rust for OS work |
| Frontend | Svelte 5 + TypeScript | Small, fast, good fit for a tile renderer |
| Deck format | One JSON file + JSON Schema | An LLM can edit it; a bad edit fails validation |
| Sharing | `.easydeck` zip (deck JSON + icons) | Simple, no code in the pack |
| LLM access | Local MCP server + `FOR_LLMS.md` | Works with any LLM the user already has |
| Classifier | `Classifier` trait; Vercel AI Gateway `typesafe-ai/jev` or a local model | Cheap yes/no calls gate the expensive LLM; users pick |
| Platform | Windows first | Window, key and process APIs differ per OS; ship one well |

## 4. Safety model

A deck can run commands and key macros, so a deck is untrusted input until the user accepts it.

- Import and LLM diffs show a preview and the full list of new actions first.
- Shell actions run from an allow-list only.
- The MCP server binds to `127.0.0.1` only and applies nothing without a tap in the app.
- Packs hold JSON and icons. No scripts, no secrets.
- No telemetry. Network calls happen only for sources and widgets the user set up.

## 5. Classifier role

The classifier answers yes/no questions with a probability, so the expensive LLM runs less
often. Examples: "Is the user in a game session now?", "Is this game played often enough at
this hour to pin it first?", "Is this prompt a layout change or an action?". It sits behind a
`Classifier` trait. Implementations: Vercel AI Gateway `typesafe-ai/jev`, and a local model.
Off by default. The AI SDK call is TypeScript only, so the Rust client calls the HTTP
endpoint directly (R7-2 captures the request shape first). Use an AI Gateway API key, not the
12-hour OIDC token, and check pricing before enabling.

## 6. Modules and phases

| Phase | Modules | Outcome |
|---|---|---|
| 0 | R0 | Spec, schema draft, docs, backlog, CI |
| 1 | R1 | A window on the touch monitor with launch tiles from a deck file |
| 2 | R2, R3, R4, R5 (parallel) | Live widgets, macros and game pages, GUI editor, packs |
| 3 | R6 | An LLM builds the deck through approved diffs |
| 4 | R7 | Optional classifier |

## 7. Risks and mitigations

| Risk | Mitigation |
|---|---|
| A shared deck runs something harmful | Action review before accept; allow-list for shell; no scripts in packs |
| Macros fail in games that run elevated | Document it; offer an "run as admin" setting; test in R3-1 |
| Embeds (Spotify, YouTube) break on login or cookies | Use the WebView2 profile of the app; keep the embed widget generic |
| Schema churn breaks old decks | `version` field and a migration function from day one |
| Solo maintainer, slow cadence | Small self-contained issues that LLM-assisted contributors can take |

## 8. Open questions

1. Final MCP transport: `rmcp` SDK or a small hand-written JSON-RPC server (decided in R6-1).
2. Spotify auth: PKCE in the app, or a user-provided client id only.
3. Windows only for v1 (current answer: yes).
