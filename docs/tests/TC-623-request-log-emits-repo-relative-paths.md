---
id: TC-623
title: request_log_emits_repo_relative_paths
type: invariant
status: passing
validates:
  features:
  - FT-051
  adrs:
  - ADR-039
phase: 5
runner: cargo-test
runner-args: "tc_623_request_log_emits_repo_relative_paths"
last-run: 2026-04-20T13:50:09.260044626+00:00
last-run-duration: 0.2s
---

## Session — request-log-emits-repo-relative-paths

### Given

Two freshly-initialised clones of a test repo at two distinct absolute
paths (`<tmpdir-A>/product-cli` and `<tmpdir-B>/product-cli`), each with
`requests.jsonl` empty.

### When

Identical `product request apply` calls run in both clones — each
creating the same fixture feature, ADR, and TC.

### Then

For the newly-appended entry in each clone:

- Every `file` value under `request.created[].file`,
  `request.changed[].file`, and `result.created[].file` (if present) is a
  **relative** path starting with `docs/features/`, `docs/adrs/`, or
  `docs/tests/` (no leading `/`, no drive letter).
- No `file` value contains either absolute tmpdir prefix
  (`/tmp/…/clone-a`, `/tmp/…/clone-b`) — confirming no machine-specific
  path leaks into the log.

## ⟦Γ:Invariants⟧

for every entry E appended after FT-051 ships:
  for every path field P in E:
    starts_with(P, "/") == false
    starts_with(P, "C:\\") == false
    starts_with(P, repo_root) == false

## ⟦Σ:Types⟧

PathField ≜ { "request.created[].file", "request.changed[].file",
              "result.created[].file" }