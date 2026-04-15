---
id: TC-458
title: tags_config_defaults
type: scenario
status: unimplemented
validates:
  features: []
  adrs: []
phase: 1
---

## Scenario

The `[tags]` config section in product.toml is optional with sensible defaults.

### Given
- A product.toml with NO `[tags]` section

### When
- Any tag-related operation runs (verify completion, drift check, tags list)

### Then
- `auto-push-tags` defaults to `false`
- `implementation-depth` defaults to `20`
- No parse error, no crash

### With explicit config
- A product.toml with `[tags]\nauto-push-tags = false\nimplementation-depth = 30` parses correctly
- `implementation-depth` of 30 is used instead of the default 20