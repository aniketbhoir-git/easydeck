# Contributing

Thanks for helping. This is a slow, async project: **there are no deadlines and no one blocks on
anyone else.** Every implementation issue is a self-contained vertical slice, so you can work
fully independently.

## Ways to help

1. **Implement an issue** from [`docs/ISSUE-BACKLOG.md`](docs/ISSUE-BACKLOG.md) — code you
   generate with your own LLM subscription is welcome (see below).
2. **Test on real hardware** — issues labeled `verify:human` need a person with a touch
   monitor, a second display or a real game.
3. **Share decks** — once import and export land, sample `.easydeck` packs go in `examples/`.
4. **Improve docs** — `docs/SPEC.md`, `docs/FOR_LLMS.md`, the issue template, the backlog.
5. **Review PRs** against `docs/SPEC.md` §0 and §9.

## Process

1. Pick an unassigned issue with status `ready` in the GitHub issue tracker; comment "I'm on
   this". A claim with no pull request after 7 days is free again. Issue ids (A-3, R2-3 and
   so on) match `docs/ISSUE-BACKLOG.md`.
2. Branch from `main`: `impl/<issue-id>--short-name`.
3. Follow `docs/SPEC.md`. If you're generating code with a model, read
   [`docs/CONTRIBUTING-WITH-LLM.md`](docs/CONTRIBUTING-WITH-LLM.md) first.
4. Make the issue's tests pass. Run `cargo xtask check` (until issue A-4 lands: `cargo test`,
   `cargo clippy`, `cargo fmt`). Do the manual steps when the issue is `verify:human`.
5. Open a PR with the PR template. Keep it to one issue.

## Labels

Labels say what proves the work, not which model to use:

| Label | Meaning |
|---|---|
| `verify:auto` | CI proves the acceptance criteria. Use any model; the tests decide. **Community tokens matter most here.** |
| `verify:human` | A person with the hardware must check. The issue lists the manual steps. |
| `review:owner` | The change touches a protected path (platform, MCP, risk and approval code, workflows). The maintainer reviews it line by line. |

## Code standards

- An LLM or an import changes the deck file only, and only after the user approves (SPEC §0).
- Nothing risky runs before the user approves it (risk items, SPEC §4.3).
- Shell actions use the allow-list only. The MCP server binds to localhost only.
- Pure logic goes in `easydeck-core`; OS calls go in `easydeck-platform-windows` only.
- No telemetry; no secrets in decks, packs, logs or fixtures.
- Rust: clippy clean, `cargo fmt` clean, tests for validation, diff, layout and parsers.

## Maintainer

Currently a solo maintainer moving at a slow cadence. PRs are reviewed asynchronously; if yours
sits for a while, it is not personal — feel free to ping.
