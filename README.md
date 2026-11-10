# EasyDeck

An open-source desktop app that turns a small touch monitor into a dynamic control deck for
Windows. Launch games and apps, fire key macros, watch live stats, and run media widgets
(Spotify, YouTube, any web page) from tiles you can build in a simple GUI — or describe to
your own LLM and let it build the whole deck for you.

> Not affiliated with or endorsed by Valve, Spotify, or any other third party. Product names
> and logos belong to their owners.

## Status

Early and in progress (module R0: spec, docs, backlog, CI). This is deliberately a
**slow-moving, community-built project** — no deadlines, contribution is asynchronous, and
every piece of work is a self-contained issue that anyone (on any model tier) can pick up.
See `docs/ISSUE-BACKLOG.md`. There is no app code yet; issue R1 creates the Tauri shell.

## How it works

- **Touch-first window.** A kiosk window on the display you choose, with press animations,
  no text selection and no scroll bounce, so it feels like a device and not a web page.
- **Tiles and widgets.** Launch tiles (Steam, program, URI, web), key macros, toggles, live
  stats (CPU, GPU, RAM), status tiles, clock, a Spotify widget and a web-embed widget.
- **Game-aware pages.** When a game starts, its page opens. When the game stops, the deck
  returns. Launch tiles sort recent-first.
- **GUI editor.** Add, move, resize and edit tiles in the app.
- **Deck files.** A deck is one JSON file, checked by a JSON Schema
  ([`schema/deck.schema.json`](schema/deck.schema.json)). Export it as a `.easydeck` pack;
  others import it. Import shows every action the deck can run before you accept it.
- **LLM-built decks.** A local MCP server lets your LLM read the deck and propose a diff.
  You approve each diff in the app. The LLM edits the deck file, never the app code. See
  [`docs/FOR_LLMS.md`](docs/FOR_LLMS.md).
- **Optional classifier.** A small yes/no model (Vercel AI Gateway `typesafe-ai/jev`, or a
  local model) makes cheap calls such as "is the user in a game now?".

## Stack

Rust · Tauri 2 (WebView2) · Svelte/TypeScript · tokio · windows-rs · JSON Schema

Windows first. macOS and Linux are out of scope for v1.

## Development

Prerequisites: Rust stable (MSVC toolchain), Node 22+, the Visual Studio Build Tools "Desktop
development with C++" workload, and the WebView2 runtime. Once issue R1 lands: run
`npm install` in `ui/`, then `npm run tauri dev` from `ui/`. From `src-tauri/`, check your work
with `cargo test`, `cargo clippy --all-targets -- -D warnings` and `cargo fmt --all -- --check`.

## Contributing

Contributors can donate their **LLM tokens** instead of (or in addition to) hours: each
implementation issue is a paste-ready prompt. Start here:

- [`CONTRIBUTING.md`](CONTRIBUTING.md)
- [`docs/CONTRIBUTING-WITH-LLM.md`](docs/CONTRIBUTING-WITH-LLM.md)
- [`docs/SPEC.md`](docs/SPEC.md) — the LLM hand-off spec (safe to paste into any model)
- [`docs/SKILLS.md`](docs/SKILLS.md) — agent skills that help on this codebase
- [`docs/ISSUE-BACKLOG.md`](docs/ISSUE-BACKLOG.md) — the issue list; pick one labelled `tier:any` or `tier:max`

## License

MIT — see [`LICENSE`](LICENSE).
