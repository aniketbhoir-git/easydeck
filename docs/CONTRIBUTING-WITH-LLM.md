# Contributing with your own LLM tokens

This repository is designed so that community members can contribute **compute** as well as
hours. You point your own coding agent (Claude Code, Codex, Cursor, any tool) at an issue.
The issue already has its tests. The agent makes them pass. CI proves the result.

Tokens are not the scarce part of this project. The maintainer's review time is. So the rule
of the whole design is: **CI is the first reviewer.** A pull request that CI proves is cheap
to merge. A pull request that needs a person to work out whether it is right is expensive, no
matter how few tokens it cost.

## The flow

1. Pick an issue with status `ready` in `docs/ISSUE-BACKLOG.md` and claim it on GitHub.
   `ready` means its contract is on `main`: the types, the trait signatures and the
   acceptance tests, marked `#[ignore = "<issue id>"]` or `test.skip`.
2. Start your agent in the repository root. It reads `AGENTS.md` (or `CLAUDE.md`) on its own.
   Tell it: "Implement issue <ID> from docs/ISSUE-BACKLOG.md. Read docs/SPEC.md §0 first."
3. The agent removes the ignore marks and makes the tests pass. It does not change what a
   test asserts. If a test looks wrong, stop and say so in the issue.
4. Run `cargo xtask check`. It runs what CI runs. (Until issue A-4 lands: `cargo test`,
   `cargo clippy --all-targets -- -D warnings`, `cargo fmt --all -- --check` in `src-tauri/`,
   and `npx vitest run` in `ui/`.)
5. Read the diff yourself. Open one pull request for the one issue, with the template.

## What you can work on, by machine

| You have | You can take |
|---|---|
| Any OS, or a cloud agent sandbox (Linux) | Issues in `easydeck-core`, `easydeck-mcp`, `easydeck-cli` and `ui/`. `cargo xtask check --core` proves them. |
| Windows 10/11 | Everything above, plus the platform crate and the app. |
| Windows + a touch monitor or a second display | `verify:human` issues. These are worth the most. |

Until issue A-2 (workspace split) merges, every build needs Windows.

## Labels

| Label | Meaning |
|---|---|
| `verify:auto` | CI proves the acceptance criteria. Any model may try; the tests decide. |
| `verify:human` | A person with the hardware must check. The issue lists the manual steps. |
| `review:owner` | The change touches a protected path. The maintainer reviews it line by line, so keep it small. |

The labels do not name models. If a cheap model makes the tests pass and the checks are
green, the work is good. If it cannot, use a stronger model or leave the issue.

## Rules for the agent

- Stay inside the issue. No extra refactor, no extra feature, no new dependency outside
  `docs/SPEC.md` §2.
- Pure logic goes in `easydeck-core`, behind a trait, with a fake in the tests. OS calls go in
  `easydeck-platform-windows` only. CI rejects the opposite.
- Never weaken a guardrail to make a test pass (`docs/SPEC.md` §0 and §9).
- No real launch and no real key in a plain `cargo test`. Real input goes to
  `easydeck-probe` only, in the opt-in platform job.
- If the model proposes anything outside the scope (telemetry, a cloud account, free-form
  shell commands from a deck), drop that part.

## What counts as a finished pull request

- One issue; the issue is linked.
- CI is green. The contract tests pass, and the PR says "only un-ignored" or lists each
  assertion it changed, with the reason.
- For `verify:human`: the manual steps, what you ran and on what hardware.
- You read the code. Never send code you did not read.

## If you are the maintainer

- Write the contract before you open an issue: types, traits, ignored tests. A good contract
  is most of the design work, and it is the part that needs a top model and your judgment.
- Keep `review:owner` paths short and stable, so that most pull requests do not touch them.
- When a pull request is hard to review, the fix is a better test in the contract, not a
  longer review.
