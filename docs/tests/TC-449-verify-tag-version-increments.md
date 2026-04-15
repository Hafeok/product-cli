---
id: TC-449
title: verify_tag_version_increments
type: scenario
status: unimplemented
validates:
  features: [FT-037]
  adrs: [ADR-035, ADR-036]
phase: 1
---

## Scenario

When a feature is re-verified after already having a completion tag, the next version tag is created instead of failing.

### Given
- A git-initialized temp directory with product.toml
- Feature FT-001 with all TCs passing
- Tag `product/FT-001/complete` already exists from a prior verification

### When
- `product verify FT-001` is run again

### Then
- Tag `product/FT-001/complete-v2` is created
- stdout contains "Tagged: product/FT-001/complete-v2"
- Both tags exist: `product/FT-001/complete` and `product/FT-001/complete-v2`

### Edge case
- If `complete-v2` also exists, `complete-v3` is created, and so on