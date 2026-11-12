Closes: <!-- issue id, for example R1-2 -->

## What changed
<!-- Brief description. -->

## Acceptance criteria
<!-- Copy the checkboxes from the issue and tick each. -->

## Evidence
- CI: <!-- green? link the run -->
- Tests changed: <!-- "only un-ignored", or list each assertion you changed and why -->
- Manual steps (`verify:human` issues only): <!-- what you ran, on what hardware -->

## Reviewer checklist (what CI cannot prove yet; `docs/SPEC.md` §9)
- [ ] an LLM or import path cannot apply a deck without user approval
- [ ] a new action kind or endpoint kind is a risk item in the review screen
- [ ] shell actions go through the allow-list only
- [ ] no telemetry; no network call outside an approved risk item
- [ ] no secrets in decks, packs, logs or fixtures
- [ ] no third-party marks or assets bundled
- [ ] no new dependency outside `docs/SPEC.md` §2, or the PR says why

## Model / tokens (optional)
<!-- model used + rough tokens, to help track community token spend -->
