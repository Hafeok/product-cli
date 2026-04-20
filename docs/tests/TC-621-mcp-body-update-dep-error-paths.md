---
id: TC-621
title: mcp_body_update_dep_error_paths
type: scenario
status: passing
validates:
  features:
  - FT-050
  adrs:
  - ADR-031
phase: 5
runner: cargo-test
runner-args: "tc_621_mcp_body_update_dep_error_paths"
last-run: 2026-04-20T13:27:05.074524328+00:00
last-run-duration: 0.1s
---

## Session — mcp-body-update-dep-error-paths

### Given

A fixture repo with a loaded graph that contains no `DEP-999` and knows
only the default four prefixes (`FT-`, `ADR-`, `TC-`, `DEP-`).

### When

The caller invokes `product_body_update` twice:

1. `{"id": "DEP-999", "body": "..."}` — prefix is valid, ID is unknown.
2. `{"id": "FOO-001", "body": "..."}` — prefix is not configured.

### Then

- The first call returns an error whose message names `DEP-999` (mirrors
  the wording of the feature / ADR / TC "not found" errors).
- The second call returns an error whose message is the existing
  `"Unknown artifact ID prefix: FOO-001"` string — the new dep branch has
  not changed the fallback wording.
- Neither call mutates any file on disk (checksum of the dependencies
  directory is unchanged before vs. after).