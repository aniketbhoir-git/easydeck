# CLAUDE.md

Open-source Rust + Tauri 2 desktop app that turns a small touch monitor into a deck
("EasyDeck"). Windows first. MIT licensed, community-built, slow-moving.

## Source of truth (read before coding)
- `docs/SPEC.md` — implementation spec (interfaces, modules, work packages)
- `docs/STRATEGY.md` — rationale and scope
- `docs/ISSUE-BACKLOG.md` — work items and their acceptance criteria
- `schema/deck.schema.json` — the deck file contract

## Non-negotiable rules
- An LLM or an imported file changes the deck file only, never app code. Every such change is
  a diff or a pack that the user approves in the app first.
- The app shows every new action (launch, key macro, shell) before it accepts a deck.
- Shell actions run from an allow-list only.
- The MCP server binds to `127.0.0.1` only.
- No telemetry. Network calls only for sources and widgets the user configured.
- No secrets in decks, packs, logs or fixtures.
- Original branding only; do not bundle third-party marks or assets.

## Stack (do not add heavy dependencies beyond SPEC.md §2)
Rust stable · Tauri 2 · Svelte/TypeScript · tokio · windows-rs · jsonschema

## Work loop
- Work one issue at a time, in backlog order unless told otherwise.
- Implement incrementally: `cargo build` → `cargo test` → `cargo clippy` → `cargo fmt`
  after each logical chunk; commit frequently with clear messages.
- Write tests for validation, diff, layout and parsers. Actions run behind traits; tests use
  fakes and never launch a program or send a key.
- Stop at the issue's definition of done, then report before continuing.

## Tier note
`tier:max` issues need a top-tier model (Windows platform code, MCP flow). `tier:any` issues
are for community contributors on cheaper models.

## Status
- Module R0 (define) done: spec, schema draft, docs, backlog, CI.
- Next: R1-1 (Tauri scaffold). There is no app code yet; CI is dormant until R1 lands.

## Git workflow
- `main` is protected: the `windows` CI check must pass; no force pushes.
- Work on `impl/<issue-id>--short-name` (for example `impl/R1-2--kiosk-window`), open a PR
  with the template, tick the guardrail checklist. Commit messages: conventional prefix
  (`feat(core):`, `fix(deck):`, `ci:`, `docs:`).

## Environment notes (Windows 11)
- Cargo is at `%USERPROFILE%\.cargo\bin` and may be missing from a tool's PATH. Bash:
  `export PATH="$HOME/.cargo/bin:$PATH"`; PowerShell:
  `$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"`.
- Toolchain: Rust stable MSVC, VS Build Tools 2022 (C++ workload), Node 22+, WebView2.
- Build the UI once (`npm install && npm run build` in `ui/`) before the first `cargo build`;
  `tauri::generate_context!` embeds `ui/dist`.
- Antivirus software may lock a freshly built executable on first launch. If the linker
  reports LNK1104, `cargo test --lib` still works; a reboot usually clears it.

## Layout (planned; see SPEC §3)
- `src-tauri/src/deck/` — model, load, validate, diff.
- `src-tauri/src/actions/` — `ActionRunner` per kind: launch, key macros, shell allow-list.
- `src-tauri/src/sources/` — system stats, HTTP poll, game watch.
- `src-tauri/src/display/` — display list, kiosk window.
- `src-tauri/src/mcp/` — local MCP server and its four tools.
- `src-tauri/src/classifier/` — `Classifier` trait, Jev and local clients.
- `ui/` — Svelte deck renderer and editor.
