---
id: TC-451
title: tags_list_all
type: scenario
status: passing
validates:
  features: [FT-037]
  adrs: [ADR-035, ADR-036]
phase: 1
runner: cargo-test
runner-args: "tc_451_tags_list_all"
last-run: 2026-04-15T10:58:05.808853076+00:00
last-run-duration: 0.2s
---

## Scenario

`product tags list` displays all product/* tags with artifact ID, event, and date.

### Given
- A git-initialized temp directory with product.toml
- Two annotated tags exist: `product/FT-001/complete` and `product/FT-002/complete`

### When
- `product tags list` is run

### Then
- stdout contains "FT-001" and "complete"
- stdout contains "FT-002" and "complete"
- Exit code is 0

### JSON variant
- `product tags list --format json` returns a JSON array with tag objects containing `name`, `artifact_id`, `event`, `timestamp`