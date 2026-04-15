---
id: TC-454
title: tags_show_feature
type: scenario
status: passing
validates:
  features: [FT-037]
  adrs: [ADR-035, ADR-036]
phase: 1
runner: cargo-test
runner-args: "tc_454_tags_show_feature"
last-run: 2026-04-15T10:58:05.808853076+00:00
last-run-duration: 0.2s
---

## Scenario

`product tags show FT-001` displays full detail for a feature's tags including message content.

### Given
- A git-initialized temp directory with product.toml
- Annotated tag `product/FT-001/complete` with message "FT-001 complete: 2/2 TCs passing (TC-001, TC-002)"

### When
- `product tags show FT-001` is run

### Then
- stdout contains "product/FT-001/complete"
- stdout contains the tag message (TC list)
- stdout contains a timestamp
- Exit code is 0

### Not found
- `product tags show FT-999` returns exit code 1 or prints "no tags found"