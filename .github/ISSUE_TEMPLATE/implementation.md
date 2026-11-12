---
name: Implementation task
about: Implement one slice of docs/SPEC.md (agent-assisted contributions welcome)
title: "[impl] "
labels: ["llm-friendly"]
---

### Summary
<!-- One line: what this slice delivers. -->

### Proof / size
proof: <!-- `verify:auto` or `verify:human`; add `review:owner` for a protected path --> · size: <!-- S / M / L -->

### Depends on
<!-- Issue ids (for example R1-1), or "none". -->

### Requirements
<!-- Files to create/modify, plus the exact behavior. Cite docs/SPEC.md sections. -->

### Acceptance criteria
- [ ]
- [ ]

### Contract
<!-- The PR that merged the types and the ignored tests, and the ignore tag to remove
     (for example `#[ignore = "R3-3"]`). -->

### Tests
- [ ] The contract tests pass with no change to what they assert
- [ ] `cargo xtask check` passes
- [ ] Manual steps (`verify:human` only): <!-- list them -->

### For the agent
Read `AGENTS.md` (or `CLAUDE.md`), `docs/SPEC.md` §0 and this issue. Stop and ask when the
issue conflicts with the spec.
