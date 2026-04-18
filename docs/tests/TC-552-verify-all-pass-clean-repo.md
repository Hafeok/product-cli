---
id: TC-552
title: verify_all_pass_clean_repo
type: scenario
status: unimplemented
validates:
  features:
  - FT-044
  adrs:
  - ADR-040
phase: 1
runner: cargo-test
runner-args: tc_552_verify_all_pass_clean_repo
---

## Session: ST-110 — verify-all-pass-clean-repo

**Validates:** FT-044, ADR-040 (Stage 1–6 all pass on a healthy repo)

### Given

A temp repository with:
- At least one feature whose status is `complete` and whose TCs all pass
- A clean `requests.jsonl` with a valid hash chain
- No graph errors or warnings
- `product.toml` `schema-version` matching the binary
- Every configured metric threshold satisfied

### When

`product verify` is run with no arguments.

### Then

- All six stages execute and emit `pass`.
- Exit code is `0`.
- Pretty output shows `Result: PASS` and lists all six stage rows with `✓`.
- `--ci` JSON mode produces a document whose every `stages[i].status == "pass"` and whose top-level `passed: true`.
- No stderr output beyond the formatted report.
