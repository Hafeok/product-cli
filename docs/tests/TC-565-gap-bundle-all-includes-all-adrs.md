---
id: TC-565
title: gap_bundle_all_includes_all_adrs
type: scenario
status: failing
validates:
  features:
  - FT-045
  adrs:
  - ADR-019
  - ADR-040
phase: 1
runner: cargo-test
runner-args: tc_565_gap_bundle_all_includes_all_adrs
last-run: 2026-04-18T10:42:24.877521871+00:00
last-run-duration: 0.2s
failure-message: "No matching test function found (0 tests ran)"
---

## Session: ST-122 — gap-bundle-all-includes-all-adrs

**Validates:** FT-045, ADR-019 (amended), ADR-040

### Given

A temp repository with N ADRs (N ≥ 3).

### When

`product gap bundle --all` is run.

### Then

- The output contains exactly N bundles.
- Every ADR ID appears exactly once as a bundle title.
- The order is deterministic (sorted by ADR ID).
- No ADR is omitted even if it has no linked features or TCs.
- Exit code is `0`.