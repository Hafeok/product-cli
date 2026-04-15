---
id: TC-453
title: tags_list_filter_type
type: scenario
status: unimplemented
validates:
  features: [FT-037]
  adrs: [ADR-035, ADR-036]
phase: 1
---

## Scenario

`product tags list --type complete` shows only completion tags.

### Given
- A git-initialized temp directory with product.toml
- Tags: `product/FT-001/complete`, `product/ADR-002/accepted`

### When
- `product tags list --type complete` is run

### Then
- stdout contains "FT-001" and "complete"
- stdout does NOT contain "ADR-002" or "accepted"
- Exit code is 0