# EasyDeck — build handoff

The build orchestrator keeps this file current at each stopping point. The project manager
reads it. Mode: **GTAVI** (owner instruction, 2026-11-25): keep building, self-merge when CI
is green and an independent Opus review finds no blocking issue. Self-merged PRs carry the
label `gtavi:self-merged`.

## State

- Updated: 2026-11-25
- `main`: A-2 merged (#41, squash).
- Running: A-3 (#32) and A-7 (#36) builds, each in its own worktree.

## Done (this run)

- Label `gtavi:self-merged` created.
- A-2 (#41) merged after an Opus review (no blocking issue; 3 doc nits fixed).
- Research: `docs/research/2026-11-25-deck-landscape.md` (PR #43).

## Next

1. A-3 (#32) generated contract.
2. A-7 (#36) UI test harness.
3. A-4 (#33) xtask.
4. A-9 (#38) layout rules.
5. Gadget set (issues filed per gadget, from the research doc).

Skipped: A-1 (#30, needs the owner at the touch monitor), A-6 (#35, waits for A-1),
A-5 (#34, runs after milestone M1).

## Decision log

| Date | Decision | Why | Reversible by |
|---|---|---|---|
| 2026-11-25 | Merge method: squash | Matches earlier merges (#28, #40) | Use another method on later PRs |
| 2026-11-25 | Leave repo auto-merge off; merge by hand after green CI | A settings change is the owner's call | Owner turns on auto-merge |
| 2026-11-25 | Run A-3 and A-7 in parallel from the A-2 head | They touch different files except `ci.yml` | Rebase one PR after the other merges |
| 2026-11-25 | Gadget build order follows the research list (clock, counter, timer, gauge, volume, media, toggle look, long press) | The research ranks these as basic and low risk | Reorder the issues |

## Test queue for Aniket

1. **A-1 focus spike (PR #42, draft).** Run the spike in `spikes/focus/` on the PC with the
   touch monitor. Fill each "Observed result" in `docs/spikes/A-1-focus-input.md`. Steps are
   in that report. Then mark PR #42 ready.
2. **A-2 smoke check.** From `ui/`, run `npm run tauri dev`. Check that the window shows the
   starter deck.
