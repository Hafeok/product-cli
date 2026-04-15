---
id: TC-452
title: tags_list_filter_feature
type: scenario
status: unimplemented
validates:
  features: []
  adrs: []
phase: 1
---

## Scenario

`product tags list --feature FT-001` shows only tags for that feature.

### Given
- A git-initialized temp directory with product.toml
- Tags: `product/FT-001/complete`, `product/FT-001/complete-v2`, `product/FT-002/complete`

### When
- `product tags list --feature FT-001` is run

### Then
- stdout contains "FT-001/complete" and "FT-001/complete-v2"
- stdout does NOT contain "FT-002"
- Exit code is 0