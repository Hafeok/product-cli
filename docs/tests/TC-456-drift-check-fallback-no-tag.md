---
id: TC-456
title: drift_check_fallback_no_tag
type: scenario
status: unimplemented
validates:
  features: []
  adrs: []
phase: 1
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