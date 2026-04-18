---
id: TC-569
title: drift_diff_outputs_diff_and_governing_adrs
type: scenario
status: passing
validates:
  features:
  - FT-045
  adrs:
  - ADR-023
  - ADR-040
phase: 1
runner: cargo-test
runner-args: tc_569_drift_diff_outputs_diff_and_governing_adrs
last-run: 2026-04-18T14:07:23.114790413+00:00
last-run-duration: 0.2s
---

## Session: ST-126 — drift-diff-outputs-diff-and-governing-adrs

**Validates:** FT-045, ADR-023 (amended), ADR-040

### Given

A temp repository initialised as a git repo with FT-001 marked complete (a `product/FT-001/complete` tag exists). After the tag, one implementation file listed under `[drift].source-roots` is modified.

### When

`product drift diff FT-001` is run.

### Then

- stdout is a single markdown document with sections: `## Instructions`, `## Implementation Anchor`, `## Changes Since Completion`, `## Governing ADRs`.
- Instructions lists drift codes D001–D004.
- Implementation Anchor names the completion tag and its timestamp.
- Changes Since Completion contains the actual `git diff product/FT-001/complete..HEAD -- <file>` output.
- Governing ADRs contains the depth-2 bundle for the ADRs linked to FT-001.
- No LLM call was made.
- Exit code is `0`.