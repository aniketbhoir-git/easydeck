# CLAUDE.md

@AGENTS.md

Claude Code specifics only. The rules, commands and layout are in `AGENTS.md`.

## Work loop
- Work one issue at a time, in the order of `docs/ISSUE-BACKLOG.md` unless told otherwise.
- Implement incrementally; run `cargo xtask check` after each logical chunk (use
  `--core` when only core or CLI code changed); commit frequently with clear messages.
- Pure logic goes in `easydeck-core` behind traits, with fakes in tests. OS calls go in
  `easydeck-platform-windows` only.
- Stop at the issue's definition of done, then report before continuing.
- Ask before starting the app or a spike on the maintainer's screen.

## Architect consult
The architecture is SPEC v4 (2026-11-11); the reasons are in STRATEGY §3.1. Return to the
maintainer, who consults the architect session, when: a spike result contradicts the spec, a
crate boundary in SPEC §3 does not hold, a dependency outside SPEC §2 is needed, or a ground
rule blocks an issue. Send the question, the options and the minimum facts.

## Status
- Done: R0, R1-1 (#26), R1-2 (#27), R1-3 (#28), A-2 (#41), A-3 (#50), A-7 (#51), A-4.
- GTAVI mode (owner instruction, 2026-11-25): the build orchestrator keeps building and
  self-merges; state, decisions and the owner's test queue are in `docs/HANDOFF.md`.
- Next: A-9 (layout), then the gadget issues G-1 … G-6 (#44–#49) with R2-1, R2-2, R2-3.
  **A-5** runs after milestone M1. **A-1** (focus spike) needs the maintainer at the touch
  monitor. **A-6** waits for the A-1 answer.
- GitHub issues: A-1 … A-9 are #30–#38, R6-4 is #39. #23–#25 (R7) carry the `deferred` label.
- `position` stays required until A-9.

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
