---
id: TC-733
title: product_feature_depends_on add writes the edge
type: scenario
status: passing
validates:
  features:
  - FT-062
  adrs:
  - ADR-020
  - ADR-037
phase: 5
runner: cargo-test
runner-args: tc_733_mcp_feature_depends_on_add_writes_edge
last-run: 2026-05-08T08:03:32.829301623+00:00
last-run-duration: 0.6s
---

## Given

A session repository with two features `FT-001` and `FT-002`, both
`status: planned`, neither linked to the other.

## When

The MCP-equivalent CLI invocation
`product feature depends-on FT-001 --add FT-002` runs.

## Then

- Exit code is `0`.
- `FT-001`'s front-matter `depends-on` array now contains `FT-002`.
- The structured response (JSON) reports
  `{ "added": ["FT-002"], "removed": [], "changed": true }`.
- Re-running the same command is a no-op: exit code `0`, `changed: false`,
  no new entry added (idempotent).
- `product graph check` exits `0` after both invocations.