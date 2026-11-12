# Session prompts

Paste-ready prompts for a Claude Code (or other agent) session in the repo root.
Fenced as plain text so they can be copied directly.

## Status (2026-11-11)

- Done: R0, R1-1 (PR #26), R1-2 (PR #27). R1-3 is in PR #28.
- Architecture review v4 is merged as docs: `docs/SPEC.md` v4, `docs/STRATEGY.md` §3.1,
  `docs/ISSUE-BACKLOG.md` v4. The code does not follow v4 yet.
- Next: **A-2** (after PR #28 merges) → **A-3** → **A-4** → **A-5**, **A-7**. **A-1** runs in
  parallel and needs the maintainer at the touch monitor. **A-6** waits for A-1.

## Resume prompt — one issue per session

Replace `<ID>` with the issue id (for example `A-2`). Run from the repo root.

```text
You are continuing "EasyDeck", an open-source Rust + Tauri 2 desktop app that turns a small
touch monitor into a deck (launch tiles, key macros, live stats, media controls), Windows first.

YOUR SCOPE (hard stop): implement Issue <ID> exactly as written in docs/ISSUE-BACKLOG.md.
Do not start any other issue. Stop and report when its acceptance criteria are met.

BEFORE WRITING CODE:
1. Read CLAUDE.md, then docs/SPEC.md §0 and the sections the issue cites, then the issue.
   Read docs/STRATEGY.md §3.1 once for the reasons behind the v4 design.
2. Read the existing code you build on.
3. Run the tests once to confirm the baseline is green. Cargo lives at
   %USERPROFILE%\.cargo\bin (see CLAUDE.md "Environment notes").

WORKFLOW:
- Branch from main: impl/<ID>--short-name. Commit logical chunks with clear messages.
- After each chunk: cargo build -> cargo test -> cargo clippy --all-targets -- -D warnings
  -> cargo fmt. Zero warnings. Once `cargo xtask check` exists, use it.
- Pure logic goes in easydeck-core behind traits, with fakes in tests. OS calls go in
  easydeck-platform-windows only.
- A plain `cargo test` never starts a program, sends a key or calls the network.
- Ask before you start the app or a spike on the maintainer's screen.
- When done: push the branch, open a PR with the repository template, make sure the
  `windows` CI check passes. Do not merge.

NON-NEGOTIABLE RULES (docs/SPEC.md §0): an LLM or an import changes the deck file only,
never app code, and only after the user approves; nothing risky runs before approval (risk
items); shell actions use the allow-list only; the MCP server binds to 127.0.0.1 only; remote
content gets no app access; no telemetry; no secrets in decks, packs, logs or fixtures;
original branding; MIT.

STACK: docs/SPEC.md §2. Check the current version of a crate before you use it. Ask before
any dependency that §2 does not list.

ESCALATION: stop and report the question, the options and the minimum facts when: a decision
changes scope; a spike result contradicts the spec; a crate boundary in SPEC §3 does not
hold; a dependency outside SPEC §2 is needed; a ground rule blocks the issue. The maintainer
decides, and consults the architect session when the question is about design.

END WITH A SHORT REPORT: files touched, test/clippy output tail, PR link, deviations from
the spec, open questions.
```

## Notes

- Run prompts from the repo root so `docs/...` paths resolve.
- `CLAUDE.md` at the repo root auto-loads for every Claude Code session, so the guardrails
  stay in force even if a prompt omits them.
- A-1 is a spike: its output is the report `docs/spikes/A-1-focus-input.md` and a decision,
  not product code.
- After this backlog merges, create the GitHub issues for A-1 … A-9 and R6-4, and put the
  `deferred` label on #23–#25.
