---
id: TC-458
title: tags_config_defaults
type: scenario
status: passing
validates:
  features: [FT-037]
  adrs: [ADR-035, ADR-036]
phase: 1
runner: cargo-test
runner-args: "tc_458_tags_config_defaults"
last-run: 2026-04-15T10:58:05.808853076+00:00
last-run-duration: 0.2s
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