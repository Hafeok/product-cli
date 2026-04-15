---
id: TC-456
title: drift_check_fallback_no_tag
type: scenario
status: passing
validates:
  features: [FT-037]
  adrs: [ADR-035, ADR-036]
phase: 1
runner: cargo-test
runner-args: "tc_456_drift_check_fallback_no_tag"
last-run: 2026-04-15T10:58:05.808853076+00:00
last-run-duration: 0.2s
---

## Scenario

When no completion tag exists for a feature or ADR, drift check falls back to the existing source-files/pattern-based discovery from ADR-023 without error.

### Given
- A git-initialized temp directory with product.toml
- Feature FT-001 linked to ADR-001, status complete
- NO tag `product/FT-001/complete` exists
- ADR-001 has `source-files: [src/main.rs]` in its body

### When
- `product drift check FT-001` is run

### Then
- stderr contains "W019" or "no completion tag" indicating fallback
- Drift check proceeds using the source-files/pattern fallback
- No crash, no unexpected error
- Exit code is 0 or 1 depending on findings (not a hard failure)