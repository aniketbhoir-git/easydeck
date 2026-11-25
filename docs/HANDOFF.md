# EasyDeck — build handoff

The build orchestrator keeps this file current at each stopping point. The project manager
reads it. Mode: **GTAVI** (owner instruction, 2026-11-25): keep building, self-merge when CI
is green and an independent Opus review finds no blocking issue. Self-merged PRs carry the
label `gtavi:self-merged`. While GTAVI mode is on, this owner instruction overrides the
CLAUDE.md git rule "Do not merge; the maintainer reviews and merges".

## State

- Updated: 2026-11-25
- `main`: A-2 (#41), A-7 (#51), A-3 (#50), A-4 (#52), G-1 (#53), A-9 (#55), G-6 (#57),
  R2-1 (#58), G-2 (#59), R2-2 (#61), G-3 (#62), G-4 (#63), G-5 (#64), #67 (follow-up #56),
  #68 (follow-up #60), R5-1 (#70), R4-1 (#71), R4-2 (#74) merged.
- Open for the owner (`review:owner`, not self-merged): PR #65 (R2-3 media), PR #76 (A-8
  approval ledger and risk gate, head 5ea2529).
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
- G-4 (#63) merged: volume tile. The review caught a probe test that did not restore the
  volume on failure; it now restores in a `Drop` guard.
- G-5 (#64) merged: toggle look per state; `run_action { widgetId, toggle: 'on' | 'off' }`.
  The review made a plain tap or a hold on a toggle `Invalid`.
- #67 merged: stronger layout property tests and resolve/mock parity (follow-up #56).
- #68 merged: counter safe-integer range and deck-reload key (follow-up #60).
- R5-1 (#70) merged: deck diff with risk marks. The review caught remote icon forms that were
  not flagged; `icon_is_inert` is now an allow-list.
- R4-1 (#71) merged: editor mode (hold 1.5 s), inspector, `validate_deck`.
- R4-2 (#74) merged: launch and macro forms. The review caught a key listener that stayed on;
  recording now stops on Esc, blur and selection change. Follow-up #75.
- A-8 (#76) built and reviewed; open for the owner. The review caught a re-seed after a
  corrupt ledger and an overwrite of an unreadable ledger; both fixed.
- Gadget issues filed: G-1 #44, G-2 #45, G-3 #46, G-4 #47, G-5 #48, G-6 #49 (label `gadget`).

## Next

1. R4-3. It depends on the A-8 merge (PR #76).
2. R2-3 media on the system media session (#8; the old title said Spotify): PR #65 waits for
   the owner's review.
3. Follow-ups: #69 vitest flake; #72 after R1-4; #73 exe picker (owner decision, see
   "Owner decisions waiting"); #75 key-name contract, before the Windows key sender (R3-1).

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
| 2026-11-25 | The approval step must check the action that runs (`holdAction` when `hold` is true; `onAction` or `offAction` for a toggle). It hooks `dispatch_impl`, the one entry of `run_action`. Text, gauge and status tiles still reach `run_action` on a tap; the action step must decide per widget kind | Review notes on G-6 and G-5 | — (rule for the approval issue) |
| 2026-11-25 | R2-1 reads CPU and RAM with windows-rs (`GetSystemTimes`, `GlobalMemoryStatusEx`), not `sysinfo` | `sysinfo` is not in SPEC §2; a new dependency needs the owner | Owner approves `sysinfo` |
| 2026-11-25 | GPU load from `nvidia-smi` (hidden window, 2 s timeout, off after one failure); `cpu.temp` is always `null` | No driver-free source for CPU temperature | Add a sensor source later |
| 2026-11-25 | Counter value kept in WebView `localStorage`, key `easydeck:widget:<deck>:<widget>` (G-2) | Never in the deck file; simplest store | Move to an app-side state file |
| 2026-11-25 | Status `okWhen`: a number compares as a number, other values as text; no `okWhen` means non-zero is OK. `path` is a metric name (R2-2) | SPEC §4 does not define it | Change the model doc and `gauge.ts` |
| 2026-11-25 | A tap on an ended countdown clears the pulse and resets; timer state is not kept across app runs (G-3) | Simplest; issue allows it | Change `timer.ts` |
| 2026-11-25 | Volume: poll the default device every 500 ms instead of a COM callback (G-4) | Simpler; easy to swap | Implement `IAudioEndpointVolumeCallback` |
| 2026-11-25 | Toggle colour: validator rule `toggle-color` only, no schema pattern (G-5) | A schema pattern gives a vague `oneOf` error; the rule gives a fix hint | Add the pattern |
| 2026-11-25 | R2-3 media PR is not self-merged; it waits for the owner | The backlog marks R2-3 `review:owner` (no CODEOWNERS file exists) | Owner merges or says to self-merge |
| 2026-11-25 | R4-2 key names come from an allow-list of `e.code` values. Modifiers are `ctrl+alt+shift+win+`. A comma is rejected. Names follow US physical positions, so the sender must map by scan code (#75) | A name must not depend on the keyboard layout | Change the allow-list and the key sender together |
| 2026-11-25 | The validator reports a missing launch field at `<widget>/<slot>/<field>` | The editor can mark the exact field | Report at the action path |
| 2026-11-25 | A-8 fingerprint = SHA-256 of `easydeck.risk.v1\0` + canonical JSON of what runs | A change to what runs must change the fingerprint | Change the prefix to version the format |
| 2026-11-25 | A-8 ledger `approvals.json` in app data, atomic write. A corrupt file becomes a `.corrupt` copy plus a fresh empty file, with no re-seed. An unreadable file means memory only for the session. The starter deck is seeded only when no file exists | A damaged ledger must not approve anything by itself | Change `approval` store |
| 2026-11-25 | A-8 PR #76 is not self-merged (`review:owner`) | The backlog marks A-8 `review:owner` | Owner merges or says to self-merge |
| 2026-11-25 | New `icon-form` rule rejects text icons with `. : / % @`. It breaks old decks such as `v1.2` | Keeps icon text from looking like a path or a URL | Remove the rule or narrow the set |

## Owner decisions waiting

1. A-8: hand-written SHA-256, or approve the `sha2` crate (outside SPEC §2)?
2. R5-2 pack format: `zip` crate (recommended), hand-written zip, or one JSON file?
3. #73: Tauri dialog plugin (`dialog:allow-open` only), or keep the text field?
4. Review and merge PR #65 and PR #76.

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
7. **G-2 counter and G-3 timer.** Run `npx vite --mode e2e` from `ui/` (mock deck, no app) and open
   `http://localhost:1420/?deck=counter` and `?deck=timer` in a browser. Tap, long press,
   and check the end pulse of the 5 s countdown.
8. **G-4 volume (PR #63).** Add a `volume` widget to your deck file (for example
   `{ "id": "vol", "type": "volume", "label": "Volume" }`). From `ui/`, run `npm run tauri dev`.
   1. Drag the slider. The Windows mixer must follow, and the tile must show the same percent.
   2. Tap the mute button. Windows must mute, and the tile must show the red "muted" state.
   3. Change the volume with the keyboard keys or the mixer. The tile must update within
      about 1 s.
   4. Mute from Windows. The tile must show "muted".
   5. Switch the default output device. The tile must follow it.
   6. Turn off all output devices. The tile must show "no data", with mute greyed out.
   7. Check a wide tile and a tall tile.
9. **G-5 toggle.** Run `npx vite --mode e2e` from `ui/` and open `?deck=toggle`. Tap the
   toggle twice and check both looks.
10. **R2-3 media (PR #65, review it line by line).** Add a `media` widget to a deck. From
    `ui/`, run `npm run tauri dev`.
    1. With no player running, the tile shows "Nothing playing" and all buttons are off.
    2. Play a track in player A (for example a browser tab). Title, artist and cover show
       within about 1 s.
    3. Press previous, play/pause and next. Each acts on the player; the tile follows.
    4. Do steps 2 and 3 again with player B (for example VLC). Start B last.
    5. Close both players. The tile goes back to "Nothing playing".
    6. With `controls: ["play_pause"]`, only that button shows.
11. **A-8 approval ledger (PR #76, review it line by line).** From `ui/`, run
    `npx vite --mode e2e` or `npm run tauri dev`.
    1. A tile with a launch action shows a lock badge.
    2. Tap the tile. The risk review opens.
    3. Approve it. Only that tile unlocks.
    4. Delete `approvals.json` in the app data folder and start again. Only the starter deck
       is seeded as approved.
