# CLAUDE.md

Open-source Rust + Tauri 2 desktop app that turns a small touch monitor into a deck
("EasyDeck"). Windows first. MIT licensed, community-built, slow-moving.

## Source of truth (read before coding)
- `docs/SPEC.md` — implementation spec (interfaces, modules, work packages)
- `docs/STRATEGY.md` — rationale and scope
- `docs/ISSUE-BACKLOG.md` — work items and their acceptance criteria
- `schema/deck.schema.json` — the deck file contract (generated from the Rust types after A-3;
  do not edit it by hand then)

## Non-negotiable rules (SPEC §0)
- An LLM proposes a new deck file only. It never changes app code. An imported pack changes
  the deck file only. The user approves each proposal or pack in the app first. A proposal
  that fails validation changes nothing.
- Nothing risky runs before the user approves it. Launch, key macro, shell and network
  endpoints are risk items (SPEC §4.3); an unapproved one is inert.
- Shell actions run from an allow-list only.
- The MCP server binds to `127.0.0.1` only.
- Remote content in an `embed` gets no Tauri IPC and no capability.
- No telemetry. Network calls only for approved risk items.
- No secrets in decks, packs, logs or fixtures.
- Original branding only; do not bundle third-party marks or assets.
- A test sends real input or starts a real program only against `easydeck-probe`, and only
  with `EASYDECK_PROBE_TESTS=1`. A plain `cargo test` never does.

## Stack (do not add dependencies beyond SPEC.md §2 without asking)
Rust stable · Tauri 2 · Svelte/TypeScript · tokio · windows-rs · jsonschema · schemars · ts-rs

## Work loop
- Work one issue at a time, in the order of `docs/ISSUE-BACKLOG.md` unless told otherwise.
- Implement incrementally: `cargo build` → `cargo test` → `cargo clippy` → `cargo fmt`
  after each logical chunk; commit frequently with clear messages. After A-4, use
  `cargo xtask check`.
- Pure logic goes in `easydeck-core` behind traits, with fakes in tests. OS calls go in
  `easydeck-platform-windows` only.
- Stop at the issue's definition of done, then report before continuing.
- Ask before starting the app or a spike on the maintainer's screen.

## Labels
`verify:auto` — CI proves the work. `verify:human` — a person with hardware must check.
`review:owner` — touches a CODEOWNERS path; the maintainer reviews it line by line.

## Architect consult
The architecture is SPEC v4 (2026-11-11); the reasons are in STRATEGY §3.1. Return to the
maintainer, who consults the architect session, when: a spike result contradicts the spec, a
crate boundary in SPEC §3 does not hold, a dependency outside SPEC §2 is needed, or a ground
rule blocks an issue. Send the question, the options and the minimum facts.

## Status
- Done: R0 (define), R1-1 (PR #26), R1-2 (PR #27). R1-3 is in PR #28.
- Architecture review v4 is docs only (PR #29). No code follows v4 yet: the code is still one
  crate in `src-tauri/`, and the schema is still hand-written.
- Next, in this order: **A-2** (workspace split, after PR #28 merges), **A-3**, **A-4**,
  **A-5**, **A-7**. **A-1** (focus spike) runs in parallel and needs the maintainer at the
  touch monitor. **A-6** waits for the A-1 answer.
- GitHub issues: A-1 … A-9 are #30–#38, R6-4 is #39. #23–#25 (R7) carry the `deferred` label.
- `position` stays required until A-9.

## Git workflow
- `main` is protected: the `windows` CI check must pass; no force pushes. Keep that job name.
- Work on `impl/<issue-id>--short-name` (for example `impl/A-2--workspace`), open a PR
  with the template. Do not merge; the maintainer reviews and merges. Commit messages:
  conventional prefix (`feat(core):`, `fix(deck):`, `ci:`, `docs:`).

## Environment notes (Windows 11)
- Cargo is at `%USERPROFILE%\.cargo\bin` and may be missing from a tool's PATH. Bash:
  `export PATH="$HOME/.cargo/bin:$PATH"`; PowerShell:
  `$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"`.
- Toolchain: Rust stable MSVC, VS Build Tools 2022 (C++ workload), Node 22+, WebView2.
- Build the UI once (`npm install && npm run build` in `ui/`) before the first `cargo build`;
  `tauri::generate_context!` embeds `ui/dist`.
- A plain `cargo build` (debug or release) makes an exe that loads the dev server at
  `http://localhost:1420` and shows "can't reach this page". Use `npm run tauri dev` from `ui/`
  for a debug run. For a standalone exe use `npm run tauri build` from `ui/`, or
  `cargo build --release --features custom-protocol`.
- Antivirus software may lock a freshly built executable on first launch. If the linker
  reports LNK1104, `cargo test --lib` still works; a reboot usually clears it.

## Layout (target; see SPEC §3. Today everything is still in `src-tauri/`)
- `crates/easydeck-core/` — deck model, validate, diff, risk, approval, layout, pages,
  action and source traits, packs. No Tauri, no `windows` crate.
- `crates/easydeck-platform-windows/` — display, kiosk, opener, keys, processes, stats, media.
- `crates/easydeck-mcp/` — local MCP server, tools, proposal queue.
- `crates/easydeck-cli/` — binary `easydeck-cli`: `validate`, `diff`, `pack`, `gen`. The app
  exe stays `easydeck`. Contributors and CI run `gen` through `cargo xtask gen`.
- `crates/easydeck-probe/` — test target for real launches and keys.
- `src-tauri/` — the Tauri app: window and IPC.
- `xtask/` — `cargo xtask check | gen`.
- `ui/` — Svelte deck renderer and editor; `ui/src/lib/generated/` is generated.
