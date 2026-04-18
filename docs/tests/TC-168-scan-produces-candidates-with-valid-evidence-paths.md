---
id: TC-168
title: Scan produces candidates with valid evidence paths
type: scenario
status: passing
validates:
  features:
  - FT-030
  adrs:
  - ADR-027
phase: 1
runner: cargo-test
runner-args: "tc_168_scan_produces_candidates_with_valid_evidence_paths"
last-run: 2026-04-18T10:41:58.940827843+00:00
last-run-duration: 0.2s
---

## Description

Run `product onboard scan` against a small test codebase (10 files with deliberate architectural patterns: consistent error handling, a module boundary, a pinned dependency). Assert that every candidate in the output `candidates.json` has an `evidence` array where each entry's `file` path exists on disk and `line` is within the file's line count.

## Verification

```bash
product onboard scan tests/fixtures/onboard-sample/ --output /tmp/candidates.json
# Parse candidates.json, for each evidence entry:
#   assert file exists
#   assert line <= wc -l file
# Assert at least 2 candidates produced
```

---