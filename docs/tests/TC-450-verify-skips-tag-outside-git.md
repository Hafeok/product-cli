---
id: TC-450
title: verify_skips_tag_outside_git
type: scenario
status: passing
validates:
  features: [FT-037]
  adrs: [ADR-035, ADR-036]
phase: 1
runner: cargo-test
runner-args: "tc_450_verify_skips_tag_outside_git"
last-run: 2026-04-15T10:58:05.808853076+00:00
last-run-duration: 0.2s
---

## Scenario

When `product verify` completes in a directory that is not a git repository, tag creation is skipped gracefully with a warning. Verify still succeeds.

### Given
- A temp directory with product.toml (NO `git init`)
- Feature FT-001 with all TCs passing

### When
- `product verify FT-001` is run

### Then
- Feature status transitions to complete (verify works normally)
- stderr contains "W018" or "not a git repository"
- No crash, no error exit code
- Exit code is 0