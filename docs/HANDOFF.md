# EasyDeck — build handoff

The build orchestrator keeps this file current at each stopping point. The project manager
reads it. Mode: **GTAVI** (owner instruction, 2026-11-25): keep building, self-merge when CI
is green and an independent Opus review finds no blocking issue. Self-merged PRs carry the
label `gtavi:self-merged`.

## State

- Updated: 2026-11-25
- `main`: d79fad1 (#40)
- Running: Opus review of PR #41 (A-2); research on deck software.

## Done (this run)

- Label `gtavi:self-merged` created.

## Next

1. A-2 (#41): review, fix, merge.
2. A-3 (#32) generated contract.
3. A-4 (#33) xtask.
4. A-7 (#36) UI test harness.
5. A-9 (#38) layout rules.
6. Gadget set (issues filed per gadget, from the research doc).

Skipped: A-1 (#30, needs the owner at the touch monitor), A-6 (#35, waits for A-1),
A-5 (#34, runs after milestone M1).

## Decision log

| Date | Decision | Why | Reversible by |
|---|---|---|---|

## Test queue for Aniket

1. **A-1 focus spike (PR #42, draft).** Run the spike in `spikes/focus/` on the PC with the
   touch monitor. Fill each "Observed result" in `docs/spikes/A-1-focus-input.md`. Steps are
   in that report. Then mark PR #42 ready.
