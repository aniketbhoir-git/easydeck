# Skills for contributors

Agent "skills" are small instruction packs (a `SKILL.md` plus optional references) that a
coding model loads on demand. This page lists the ones that help on this codebase. All are
optional; the spec and the issue text are always enough on their own.

## Project skills

None yet. Candidates once the code exists: `easydeck-work-resume` (load context and resume one
issue) and a deck-schema skill (how to edit `schema/deck.schema.json` and keep the examples
valid). Other agents (Cursor, Codex, pi) can read any `SKILL.md`; copy it into the folder your
tool expects.

## Ecosystem skills (install globally in your own tool)

Install with the Skills CLI (`npx skills add <owner/repo@skill> -g -y`) or the Claude Code
plugin marketplace. Checked 2026-11-10; licences are the repos' own.

| Skill | Why it fits here |
|---|---|
| `rust-analyzer-lsp@claude-plugins-official` (plugin) | Compiler-grade diagnostics inside the session. |
| `apollographql/skills@rust-best-practices` | Idiomatic Rust: ownership, `Result` handling, API shape. |
| `wshobson/agents@rust-async-patterns` | tokio tasks, channels, streams, cancellation (sources, process watch). |
| `affaan-m/ecc@rust-testing` | rstest, proptest, mockall, cargo-llvm-cov; matches SPEC §8. |
| `trailofbits/skills@rust-review` | Security-flavoured Rust review before a PR, useful for the key-injection and shell allow-list code (CC-BY-SA licence; use, do not copy text into this repo). |
| `nodnarbnitram/claude-code-extensions@tauri-v2` | tauri.conf.json, commands, capabilities, IPC errors. Check against the current Tauri docs. |
| `full-stack-skills/tauri-skills@tauri-ipc`, `@tauri-security` | Typed IPC contracts and the Tauri security model (window, capability and CSP choices). |
| `sveltejs/ai-tools@svelte-code-writer`, `@svelte-core-bestpractices` | Official Svelte 5 guidance for the `ui/` frontend. |

Reference material that is not a skill: Tauri's `https://v2.tauri.app/llms-full.txt`, the
Model Context Protocol docs (for R6) and Microsoft's
[Pragmatic Rust Guidelines](https://github.com/microsoft/rust-guidelines).

## Not recommended for this project

- Windows VM image builders (Packer): this is a desktop app, not an image.
- Any skill that asks the app to send data to a third party. The ground rules in
  `docs/SPEC.md` §0 allow only the sources the user configures.

## Reviewing changes

Built-in Claude Code commands `/simplify` and `/code-review`, and the `code-simplifier`
agent, cover simplification and general review. Run them before opening a PR.
