---
id: TC-787
title: mcp_feature_link_returns_structured_writes_report
type: scenario
status: passing
validates:
  features:
  - FT-066
  adrs:
  - ADR-020
phase: 5
runner: cargo-test
runner-args: tc_787_mcp_feature_link_returns_structured_writes_report
last-run: 2026-05-22T07:23:52.891705159+00:00
last-run-duration: 0.2s
---

## Description

Invoke `product_feature_link` over MCP with all three of
`id: FT-X, test: TC-Y, adr: ADR-Z` against a temp repo. Assert the
MCP response JSON contains:

1. A `writes` array with three entries (feature, TC, ADR), each
   carrying `path` (absolute file path) and `kind` (`feature` /
   `tc` / `adr`).
2. A `reciprocated` array with two entries naming the back-references
   filled in: `{ id: TC-Y, field: validates.features }` and
   `{ id: ADR-Z, field: features }`.
3. No `linked` boolean field (the legacy shape is gone).
4. The `id` field at the top level still echoes `FT-X` for backwards
   compatibility with simple readers.

Invoke a second time with the same arguments (now idempotent) and
assert the `writes` array is empty and `reciprocated` is empty —
no-op writes do not show up in the structured report.