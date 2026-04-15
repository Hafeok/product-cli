---
id: TC-448
title: verify_creates_completion_tag
type: scenario
status: passing
validates:
  features: [FT-037]
  adrs: [ADR-035, ADR-036]
phase: 1
runner: cargo-test
runner-args: "tc_448_verify_creates_completion_tag"
last-run: 2026-04-15T10:58:05.808853076+00:00
last-run-duration: 0.2s
---

## Scenario

When `product verify FT-XXX` transitions a feature from in-progress to complete (all TCs passing), an annotated git tag `product/FT-XXX/complete` is created.

### Given
- A git-initialized temp directory with product.toml and artifact directories
- Feature FT-001 with status in-progress, linked to TC-001
- TC-001 with runner: cargo-test and runner-args pointing to a passing test
- All TCs pass

### When
- `product verify FT-001` is run

### Then
- Feature status transitions to complete
- `git tag -l "product/FT-001/complete"` returns the tag
- The tag is annotated (has a message)
- The tag message contains "FT-001 complete" and lists the TC IDs
- stdout contains "Tagged: product/FT-001/complete"
- stdout contains "git push --tags"

### Integration test
- Function: `tc_448_verify_creates_completion_tag`
- Harness needs `git init` + initial commit in the temp directory before running verify