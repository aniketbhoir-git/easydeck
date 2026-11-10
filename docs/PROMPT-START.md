# Session prompts

Paste-ready prompts for a Claude Code (or other model) session in the repo root.
Fenced as plain text so they can be copied directly.

## Status (2026-11-10)

- Module R0 (spec, deck schema draft, docs, backlog, CI) is committed. There is no app code.
- Next: **R1-1 (Tauri scaffold)**, then R1-2 to R1-5. After R1, modules R2 to R5 run in
  parallel; R6 follows R5-1.

## Resume prompt — one issue per session

Replace `<ID>` with the issue id (for example `R1-1`). Run from the repo root.

```text
You are continuing "EasyDeck", an open-source Rust + Tauri 2 desktop app that turns a small
touch monitor into a deck (launch tiles, key macros, live stats, media widgets), Windows first.

YOUR SCOPE (hard stop): implement Issue <ID> exactly as written in docs/ISSUE-BACKLOG.md and
the matching GitHub issue. Do not start any other issue. Stop and report when its acceptance
criteria are met.

BEFORE WRITING CODE:
1. Read CLAUDE.md, then docs/SPEC.md §0 and the sections the issue cites, then the issue.
2. Read schema/deck.schema.json and the existing code you build on.
3. Run `cargo test` in src-tauri once to confirm the baseline is green (skip if R1-1 is not
   merged yet). Cargo lives at %USERPROFILE%\.cargo\bin (see CLAUDE.md "Environment notes").

WORKFLOW:
- Branch from main: impl/<ID>--short-name. Commit logical chunks with clear messages.
- After each chunk: cargo build -> cargo test -> cargo clippy --all-targets -- -D warnings
  -> cargo fmt. Zero warnings.
- Tests for validation, diff, layout and parsers. Run actions behind traits and use fakes.
  Never launch a real program, send real keys or call the network in unit tests.
- When done: push the branch, open a PR with the repository template, make sure the
  `windows` CI check passes, and tick the guardrail checklist in the PR body.

NON-NEGOTIABLE RULES (docs/SPEC.md §0): an LLM or an import changes the deck file only,
never app code, and only after the user approves; every new action is shown before it runs;
shell actions use the allow-list only; the MCP server binds to 127.0.0.1 only; no telemetry;
no secrets in decks, packs, logs or fixtures; original branding; MIT.

STACK: Rust stable, Tauri 2, Svelte/TypeScript, tokio, windows-rs, jsonschema. Ask before any
other dependency.

ESCALATION: if a decision changes scope, needs an unlisted dependency, or conflicts with
SPEC.md, stop and report the question with the options. The maintainer decides.

END WITH A SHORT REPORT: files touched, test/clippy output tail, PR link, deviations from
the spec, open questions.
```

## Kickoff prompt — the first session (R1-1)

```text
You are bootstrapping "EasyDeck", an open-source Rust + Tauri 2 desktop app for a touch-monitor
deck. This repo currently holds docs, a JSON Schema and CI only. Your job this session is to
turn it into a compiling, tested skeleton.

SCOPE FOR THIS SESSION (hard stop): implement Issue R1-1 (Tauri scaffold) from
docs/ISSUE-BACKLOG.md. Then stop and report. Do NOT begin R1-2 or beyond without approval.

BEFORE WRITING CODE:
1. Read docs/STRATEGY.md (context), then docs/SPEC.md in full, then docs/ISSUE-BACKLOG.md.
2. Verify the toolchain and report what is missing instead of improvising installs:
   cargo --version, rustc --version, node --version, MSVC Build Tools, WebView2.

Follow the engineering standards and rules in CLAUDE.md. End with a short report: what you
built, test/clippy output, toolchain gaps, deviations from the spec.
```

## Notes

- Run prompts from the repo root so `docs/...` paths resolve.
- `CLAUDE.md` at the repo root auto-loads for every Claude Code session, so the guardrails
  stay in force even if a prompt omits them.
- Recommended order: R1-1 → R1-2 ∥ R1-3 → R1-4 → R1-5; everything after R1 is parallel.
