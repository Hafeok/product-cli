---
id: TC-455
title: drift_check_feature_tag_based
type: scenario
status: unimplemented
validates:
  features: [FT-037]
  adrs: [ADR-035, ADR-036]
phase: 1
---

## Scenario

`product drift check FT-XXX` uses the completion tag to detect changes to implementation files since the feature was completed.

### Given
- A git-initialized temp directory with product.toml, features, and source files
- Feature FT-001 linked to ADR-001, status complete
- Tag `product/FT-001/complete` exists at a commit that touched `src/foo.rs`
- After the tag, a new commit modifies `src/foo.rs`

### When
- `product drift check FT-001` is run

### Then
- stdout reports drift — files changed since completion
- The output includes `src/foo.rs` as a changed implementation file
- Exit code reflects drift findings (0 if no high-severity, 1 if high)

### No drift case
- If no files changed since the tag, output says "No drift" or equivalent
- Exit code is 0