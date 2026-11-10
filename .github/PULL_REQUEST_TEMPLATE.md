Closes: <!-- issue id, for example R1-2 -->

## What changed
<!-- Brief description. -->

## Acceptance criteria
<!-- Copy the checkboxes from the issue and tick each. -->

## Evidence
- `cargo test`: <!-- summary / paste tail -->
- clippy/fmt: <!-- clean? -->
- Windows test: <!-- screenshot, clip or log, or "CI only" -->

## Guardrail checklist (`docs/SPEC.md` §9)
- [ ] LLM and import paths cannot apply a deck without user approval
- [ ] every new action kind is listed in the action review screen
- [ ] shell actions go through the allow-list only
- [ ] the MCP server binds to localhost only
- [ ] no secrets in decks, packs, logs or fixtures
- [ ] no telemetry; network calls only for sources the user configured
- [ ] no third-party marks or assets bundled

## Model / tokens (optional)
<!-- model used + rough tokens, to help track community token spend -->
