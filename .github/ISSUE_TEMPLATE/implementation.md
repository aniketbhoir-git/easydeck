---
name: Implementation task
about: Implement one slice of docs/SPEC.md (LLM-assisted contributions welcome)
title: "[impl] "
labels: ["llm-friendly"]
---

### Summary
<!-- One line: what this slice delivers. -->

### Tier / size
tier: <!-- `tier:max` or `tier:any` --> · size: <!-- S / M / L -->

### Depends on
<!-- Issue ids (for example R1-1), or "none". -->

### Requirements
<!-- Files to create/modify, plus the exact behavior. Cite docs/SPEC.md sections. -->

### Acceptance criteria
- [ ]
- [ ]

### Tests
- [ ] Unit/integration tests as described in `docs/SPEC.md` §8
- [ ] `cargo test` + `cargo clippy` + `cargo fmt` pass
- [ ] Windows verification (describe what you ran)

### Paste instructions
Paste `docs/SPEC.md` §0 (ground rules) followed by this issue body into your model.
An LLM changes the deck file only, never app code; every deck change needs user approval.
