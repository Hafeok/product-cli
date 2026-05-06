---
id: TC-720
title: mcp preflight returns cross-cutting domain and dep coverage
type: scenario
status: passing
validates:
  features:
  - FT-059
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_720_mcp_preflight_cross_cutting_domain_dep_coverage
last-run: 2026-05-06T13:31:35.056793027+00:00
last-run-duration: 0.2s
---

## Given

A temp Product repository with:
- One feature `FT-200` declaring `domains: ["api"]` and linking ADR-A and ADR-B (both cross-cutting, ADR-A linked, ADR-B unlinked but acknowledged with reason).
- A third cross-cutting ADR-C that is neither linked nor acknowledged (a real gap).
- One dependency `DEP-100` linked to FT-200 with an `availability_check` shell command that exits 0.
- A second dependency `DEP-101` linked to FT-200 with `status: deprecated`.

`product` is launched in MCP stdio mode against the temp repo.

## When

The test sends a `tools/call` JSON-RPC request for `product_preflight` with `{ "id": "FT-200" }`.

## Then

- The response is a `success` result.
- `feature == "FT-200"` and `feature_domains == ["api"]`.
- `cross_cutting_gaps` is an array of three entries with statuses `linked`, `acknowledged`, and `gap` respectively (matching ADR-A / ADR-B / ADR-C).
- `dep_availability` contains an entry for DEP-100 with `available: true`, `deprecated: false` and an entry for DEP-101 with `deprecated: true`.
- `summary.cross_cutting_gaps == 1`, `summary.dep_warnings == 1`.
- `status == "warnings"` (because there is at least one cross-cutting gap or dep warning).
- The CLI invocation `product preflight FT-200` against the same temp repo lists the same gap ADRs and same dep warnings (parity).