---
id: TC-454
title: tags_show_feature
type: scenario
status: unimplemented
validates:
  features: []
  adrs: []
phase: 1
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