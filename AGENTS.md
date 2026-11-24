# AGENTS.md

Open-source Rust + Tauri 2 desktop app that turns a small touch monitor into a deck
("EasyDeck"). Windows first. MIT licensed, community-built, slow-moving. Any agent tool or
contributor reads this file first.

## Source of truth (read before coding)
- `docs/SPEC.md` — implementation spec (interfaces, modules, work packages)
- `docs/STRATEGY.md` — rationale and scope
- `docs/ISSUE-BACKLOG.md` — work items and their acceptance criteria
- `schema/deck.schema.json` and `ui/src/lib/generated/` — generated; never edit by hand.
  Run `cargo xtask gen`.

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

## Commands
- `cargo xtask check` — everything the Windows CI job runs: fmt check, clippy with
  `-D warnings`, workspace tests, `gen --check`, example validation, then in `ui/`
  `npm ci`, `npm run check`, `npm test`, `npm run build`. It stops at the first failing step
  and names it. Run it before you push.
- `cargo xtask check --core` — the Linux subset (core and CLI crates only, no UI). CI job
  `core-linux` runs it.
- `cargo xtask gen` — regenerate the schema and TypeScript types (runs `easydeck-cli gen`;
  `xtask` holds no generator code). `cargo xtask gen --check` fails on a stale file.
- `ui/`: `npm run test` (vitest), `npm run e2e` (Playwright).
- CI jobs: `windows` runs `cargo xtask check`; `core-linux` runs `cargo xtask check --core`.
  `ui-linux` runs the UI checks and Playwright directly, because the screenshot baselines
  need the Playwright container image (see `ui/e2e/README.md`). Keep the job ids.

## Labels
`verify:auto` — CI proves the work. `verify:human` — a person with hardware must check.
`review:owner` — touches a CODEOWNERS path; the maintainer reviews it line by line.

## Git workflow
- `main` is protected: the `windows` CI check must pass; no force pushes. Keep that job name.
- Work on `impl/<issue-id>--short-name` (for example `impl/A-2--workspace`), open a PR
  with the template. Do not merge; the maintainer reviews and merges. Commit messages:
  conventional prefix (`feat(core):`, `fix(deck):`, `ci:`, `docs:`).

## Layout (see SPEC §3)
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
  Crates not yet created are planned (SPEC §3).
