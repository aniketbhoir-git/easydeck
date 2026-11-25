# EasyDeck — build handoff

The build orchestrator keeps this file current at each stopping point. The project manager
reads it. Mode: **GTAVI** (owner instruction, 2026-11-25): keep building, self-merge when CI
is green and an independent Opus review finds no blocking issue. Self-merged PRs carry the
label `gtavi:self-merged`. While GTAVI mode is on, this owner instruction overrides the
CLAUDE.md git rule "Do not merge; the maintainer reviews and merges".

## State

- Updated: 2026-11-25
- `main`: A-2 (#41), A-7 (#51), A-3 (#50), A-4 (#52), G-1 (#53), A-9 (#55), G-6 (#57),
  R2-1 (#58), G-2 (#59), R2-2 (#61), G-3 (#62) merged.
- Running: G-4 (#47) volume and G-5 (#48) toggle look, each in its own worktree.
- Check command: `cargo xtask check` (Windows), `cargo xtask check --core` (Linux).

## Done (this run)

- Label `gtavi:self-merged` created.
- A-2 (#41) merged after an Opus review (no blocking issue; 3 doc nits fixed).
- Research: `docs/research/2026-11-25-deck-landscape.md` (PR #43).
- A-7 (#51) merged: Playwright harness, `ui-linux` CI job, Linux screenshot baselines.
- A-3 (#50) merged: schema and TS types generated from Rust; `easydeck-cli`. The review
  caught `null` on optional fields; fixed with a `strip_null` post-process.
- A-4 (#52) merged: `xtask`, `AGENTS.md`; CI calls `cargo xtask check`. Review caught clippy
  without `--locked`; fixed.
- G-1 (#53) merged: clock, date and label tile. `format` field replaced by `hour12`,
  `seconds`, `showDate`.
- A-9 (#55) merged: `position` is optional; `layout::resolve` fills it for the UI only.
  Follow-up #56: stronger layout tests.
- G-6 (#57) merged: long press runs `holdAction`. The review caught hold tests that waited
  on real time; they now run on a paused fake clock (`ui/e2e/hold.spec.ts`).
- R2-1 (#58) merged: `system_stats` source, event `source:<id>`. The review caught a unit
  test that started the real `nvidia-smi`; fixed.
- G-2 (#59) merged: counter tile. Follow-up #60 (safe-integer range, deck-reload key).
- R2-2 (#61) merged: gauge and status widgets, shared source store. The review caught a
  frozen value on a silent source; the store now shows "no data" after 3× the interval.
- G-3 (#62) merged: countdown and stopwatch tile on a monotonic clock.
- Gadget issues filed: G-1 #44, G-2 #45, G-3 #46, G-4 #47, G-5 #48, G-6 #49 (label `gadget`).

## Next

1. G-4 volume (#47) and G-5 toggle look (#48), running.
2. R2-3 media (#8).
3. Follow-ups: #56 layout tests, #60 counter nits.

Skipped: A-1 (#30, needs the owner at the touch monitor), A-6 (#35, waits for A-1),
A-5 (#34, runs after milestone M1).

## Decision log

| Date | Decision | Why | Reversible by |
|---|---|---|---|
| 2026-11-25 | Merge method: squash | Matches earlier merges (#28, #40) | Use another method on later PRs |
| 2026-11-25 | Leave repo auto-merge off; merge by hand after green CI | A settings change is the owner's call | Owner turns on auto-merge |
| 2026-11-25 | Run A-3 and A-7 in parallel from the A-2 head | They touch different files except `ci.yml` | Rebase one PR after the other merges |
| 2026-11-25 | New widget types `counter`, `timer`, `volume`; `holdAction` on tiles; toggle looks | The owner asked for basic gadgets; the research found these gaps | Remove the types before the first release (format breaks are allowed until then) |
| 2026-11-25 | Volume moves from v2 "Should" to now (G-4) | The owner asked for a volume gadget | Close #47 |
| 2026-11-25 | Clock split from R2-2 into G-1 (#44) | The clock needs no stats source | Fold back into R2-2 |
| 2026-11-25 | **Rule breach:** one force push (`--force-with-lease`) of `impl/A-3--generated-contract` after a rebase onto the squashed A-2. No PR used the branch yet and no work was lost. | Mistake by the orchestrator | From now on: merge `main` into a branch; never rebase a pushed branch |
| 2026-11-25 | `invalid_bad_widget_id` now fails at `/pages/0/widgets/0/id` (A-3) | The generated schema has no per-widget `oneOf`; the path is more precise | Accept as is |
| 2026-11-25 | Screenshot compare is strict (`maxDiffPixels: 0`) | CI pins the Playwright container | Loosen to `maxDiffPixels: 20` if it flakes |
| 2026-11-25 | Text widget: `format` removed; `hour12`, `seconds`, `showDate` added (G-1) | An unused field misleads LLM deck writers | Add `format` back with a parser |
| 2026-11-25 | Gadget build order follows the research list (clock, counter, timer, gauge, volume, media, toggle look, long press) | The research ranks these as basic and low risk | Reorder the issues |

| 2026-11-25 | `run_action` takes `{ widgetId, hold: true }` for a long press; a hold on a tile without `holdAction` is a tap (G-6) | A slow touch must not be lost | Change `press.ts` |
| 2026-11-25 | The approval step must check the action that runs (`holdAction` when `hold` is true) | Review note on G-6 | — (rule for the approval issue) |
| 2026-11-25 | R2-1 reads CPU and RAM with windows-rs (`GetSystemTimes`, `GlobalMemoryStatusEx`), not `sysinfo` | `sysinfo` is not in SPEC §2; a new dependency needs the owner | Owner approves `sysinfo` |
| 2026-11-25 | GPU load from `nvidia-smi` (hidden window, 2 s timeout, off after one failure); `cpu.temp` is always `null` | No driver-free source for CPU temperature | Add a sensor source later |
| 2026-11-25 | Counter value kept in WebView `localStorage`, key `easydeck:widget:<deck>:<widget>` (G-2) | Never in the deck file; simplest store | Move to an app-side state file |

| 2026-11-25 | Status `okWhen`: a number compares as a number, other values as text; no `okWhen` means non-zero is OK. `path` is a metric name (R2-2) | SPEC §4 does not define it | Change the model doc and `gauge.ts` |
| 2026-11-25 | A tap on an ended countdown clears the pulse and resets; timer state is not kept across app runs (G-3) | Simplest; issue allows it | Change `timer.ts` |
| 2026-11-25 | Volume: poll the default device every 500 ms instead of a COM callback (G-4) | Simpler; easy to swap | Implement `IAudioEndpointVolumeCallback` |

## Test queue for the owner

1. **A-1 focus spike (PR #42, draft).** Run the spike in `spikes/focus/` on the PC with the
   touch monitor. Fill each "Observed result" in `docs/spikes/A-1-focus-input.md`. Steps are
   in that report. Then mark PR #42 ready.
2. **A-2 smoke check.** From `ui/`, run `npm run tauri dev`. Check that the window shows the
   starter deck.
3. **A-7 screenshots.** Open `docs/screenshots/A-7-starter-1920x480.png` and
   `docs/screenshots/A-7-starter-1024x600.png`. Say if the tile look is acceptable.
4. **G-1 clock.** Open `docs/screenshots/G-1-clock-1920x480.png` and
   `docs/screenshots/G-1-clock-1024x600.png`. Then run `npm run tauri dev` from `ui/` and
   check that the clock ticks and the date line is right for your locale.
5. **R2-1 stats.** Run `npm run tauri dev` from `ui/`. Check that the CPU gauge moves. If the
   PC has an NVIDIA GPU, check that the GPU value shows. No console window may flash.
6. **R2-2 gauge.** Open `docs/screenshots/R2-2-gauge-1920x480.png` and
   `docs/screenshots/R2-2-gauge-1024x600.png`. Say if the gauge look is acceptable.
7. **G-2 counter and G-3 timer.** Run `npm run dev` from `ui/` and open
   `http://localhost:1420/?deck=counter` and `?deck=timer` in a browser. Tap, long press,
   and check the end pulse of the 5 s countdown.
