---
id: TC-620
title: mcp_body_update_rewrites_dep_body
type: scenario
status: unimplemented
validates:
  features:
  - FT-050
  adrs:
  - ADR-030
  - ADR-031
phase: 5
runner: cargo-test
runner-args: "tc_620_mcp_body_update_rewrites_dep_body"
---

## Session — mcp-body-update-rewrites-dep-body

### Given

A fixture repo containing `DEP-001` with a known body ("Original rationale
text.") and fully populated front-matter. The `product` MCP server is
loaded against the repo's graph.

### When

The caller invokes `product_body_update` with arguments
`{"id": "DEP-001", "body": "Replacement rationale — now with migration
plan."}`.

### Then

- The tool result is `{"id": "DEP-001", "updated": true}`.
- Reading `docs/dependencies/DEP-001-*.md` back from disk shows the body
  region replaced with the new text.
- The YAML front-matter is byte-identical to the original (id, title,
  type, source, version, status, features, adrs, supersedes,
  availability-check, breaking-change-risk).
- The graph re-scanned from disk still parses cleanly (no E-class errors
  from `product graph check`).
