---
id: TC-529
title: request log hash chain exit criteria
type: exit-criteria
status: passing
validates:
  features:
  - FT-042
  adrs:
  - ADR-039
phase: 5
runner: cargo-test
runner-args: tc_529_request_log_hash_chain_exit_criteria
last-run: 2026-04-18T10:42:11.121063959+00:00
last-run-duration: 0.2s
---

## Description

Exit criteria for FT-042. This TC gates feature completion: every behavioural TC on the feature passes, and the system-level properties below hold on the live repository.

## Gates

1. **All FT-042 scenario and invariant TCs pass** (TC-505 through TC-528).
2. **`product request log verify` on the live `requests.jsonl` exits 0.** The project's own log is tamper-free.
3. **`product request log verify --against-tags` on the live repo exits 0 (or exits 2 only with documented acknowledged tags).** All completion tags correspond to log entries.
4. **`product request replay --full --output /tmp/product-replay-ci` followed by `diff -r docs/ /tmp/product-replay-ci/docs/` produces no output.** The log and the files are byte-equivalent.
5. **`product graph check` with `[log] verify-on-check = true` exits 0.** Integrated log verification passes as part of the standard health check.
6. **Validation codes E017, E018, W021 are used consistently.** The implementation emits these codes (allocated by ADR-039, no collision with ADR-032 or ADR-034) in all the paths described by TC-509, TC-510, TC-511, TC-518, and TC-519.
7. **Cross-platform determinism.** The byte output of `canonical_json(e)` is identical on Linux, macOS, and Windows for the same `e` (run on CI matrix).

All seven gates must pass before `status: complete`.