---
id: TC-529
title: request log hash chain exit criteria
type: exit-criteria
status: unimplemented
validates:
  features: []
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_529_request_log_hash_chain_exit_criteria
---

## Description

Exit criteria for FT-042. This TC gates feature completion: every behavioural TC on the feature passes, and the system-level properties below hold on the live repository.

## Gates

1. **All FT-042 scenario and invariant TCs pass** (TC-505 through TC-528).
2. **`product request log verify` on the live `requests.jsonl` exits 0.** The project's own log is tamper-free.
3. **`product request log verify --against-tags` on the live repo exits 0 (or exits 2 only with documented acknowledged tags).** All completion tags correspond to log entries.
4. **`product request replay --full --output /tmp/product-replay-ci` followed by `diff -r docs/ /tmp/product-replay-ci/docs/` produces no output.** The log and the files are byte-equivalent.
5. **`product graph check` with `[log] verify-on-check = true` exits 0.** Integrated log verification passes as part of the standard health check.
6. **Error code reconciliation complete.** The spec (`docs/product-request-log-spec.md`) and any test titles or strings no longer reference E015/E016/W021 as placeholders — the implementation has picked the next free codes (likely E017/E018/W022) and both the spec and the test fixtures reflect them.
7. **Feature depends-on is recorded.** `depends-on` in FT-042's front-matter lists FT-041, FT-018, FT-020, FT-034, FT-036, and the value was set through the eventual request interface or equivalent tool.
8. **Cross-platform determinism.** The byte output of `canonical_json(e)` is identical on Linux, macOS, and Windows for the same `e` (run on CI matrix).

All eight gates must pass before `status: complete`.
