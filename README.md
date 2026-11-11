# EasyDeck

An open-source desktop app that turns a small touch monitor into a dynamic control deck for
Windows. Launch games and apps, fire key macros, watch live stats, control your media player and
embed web pages, from tiles you can build in a simple GUI — or describe to
your own LLM and let it build the whole deck for you.

> Not affiliated with or endorsed by Valve, Spotify, or any other third party. Product names
> and logos belong to their owners.

## Status

Early and in progress. The app shell loads a deck file and draws launch tiles; the
architecture work of spec v4 comes next. This is deliberately a **slow-moving,
community-built project** — no deadlines, contribution is asynchronous, and every piece of
work is a self-contained issue that a coding agent can pick up and CI can prove.
See `docs/ISSUE-BACKLOG.md`.

## How it works

- **Touch-first window.** A kiosk window on the display you choose, with press animations,
  no text selection and no scroll bounce, so it feels like a device and not a web page. A
  tap does not take focus from your game.
- **Tiles and widgets.** Launch tiles (Steam, program, URI, web), key macros, toggles, live
  stats (CPU, GPU, RAM), status tiles, clock, media controls for any player and a web-embed
  widget.
- **Game-aware pages.** When a game starts, its page opens. When the game stops, the deck
  returns. Launch tiles sort recent-first.
- **GUI editor.** Add, move, resize and edit tiles in the app.
- **Deck files.** A deck is one JSON file, checked by a JSON Schema
  ([`schema/deck.schema.json`](schema/deck.schema.json)). Export it as a `.easydeck` pack;
  others import it.
- **Nothing runs unseen.** Every program, key macro, shell command and network address in a
  deck stays locked until you approve it, for an import, an LLM change or a file edit.
- **LLM-built decks.** A local MCP server lets your LLM read the deck and propose a new one.
  You approve each proposal in the app. The LLM edits the deck file, never the app code. See
  [`docs/FOR_LLMS.md`](docs/FOR_LLMS.md).

## Stack

Rust · Tauri 2 (WebView2) · Svelte/TypeScript · tokio · windows-rs · JSON Schema

Windows first. macOS and Linux are out of scope for v1.

## Development

Prerequisites: Rust stable (MSVC toolchain), Node 22+, the Visual Studio Build Tools "Desktop
development with C++" workload, and the WebView2 runtime. Run `npm install` in
`ui/`, then `npm run tauri dev` from `ui/` for a debug run, or `npm run tauri build` for a
standalone exe. A plain `cargo build` does not embed the UI: the window then shows
"can't reach this page". From `src-tauri/`, check your work
with `cargo test`, `cargo clippy --all-targets -- -D warnings` and `cargo fmt --all -- --check`.

## Contributing

Contributors can donate their **LLM tokens** instead of (or in addition to) hours: each
issue comes with its tests, so a coding agent can do the work and CI can prove it. Start here:

- [`CONTRIBUTING.md`](CONTRIBUTING.md)
- [`docs/CONTRIBUTING-WITH-LLM.md`](docs/CONTRIBUTING-WITH-LLM.md)
- [`docs/SPEC.md`](docs/SPEC.md) — the implementation spec
- [`docs/SKILLS.md`](docs/SKILLS.md) — agent skills that help on this codebase
- [`docs/ISSUE-BACKLOG.md`](docs/ISSUE-BACKLOG.md) — the issue list; pick one with status `ready`

## License

MIT — see [`LICENSE`](LICENSE).
