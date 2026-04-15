---
id: TC-457
title: drift_check_all_complete
type: scenario
status: passing
validates:
  features: [FT-037]
  adrs: [ADR-035, ADR-036]
phase: 1
runner: cargo-test
runner-args: "tc_457_drift_check_all_complete"
last-run: 2026-04-15T10:58:05.808853076+00:00
last-run-duration: 0.2s
---

## Scenario

`product drift check --all-complete` checks drift for every complete feature that has a completion tag.

### Given
- A git-initialized temp directory with product.toml
- FT-001 (complete, with tag `product/FT-001/complete`)
- FT-002 (complete, with tag `product/FT-002/complete`)
- FT-003 (in-progress, no tag)

### When
- `product drift check --all-complete` is run

### Then
- Drift is checked for FT-001 and FT-002
- FT-003 is skipped (not complete or no tag)
- Output reports per-feature findings
- Exit code reflects aggregate findings