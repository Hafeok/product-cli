---
id: TC-487
title: request type change round-trips
type: scenario
status: passing
validates:
  features:
  - FT-041
  adrs:
  - ADR-038
phase: 1
runner: cargo-test
runner-args: tc_487_request_type_change_round_trips
last-run: 2026-04-18T10:42:07.406621710+00:00
last-run-duration: 0.2s
---

Validates FT-041 / ADR-038 decision 1.

**Setup:** a fixture repo containing an existing feature FT-X with `domains: [api]`.

**Act:** write a `type: change` request with one change targeting `FT-X` and two mutations: `append field: domains value: security` and `append field: adrs value: ADR-Y` (where ADR-Y exists). Apply it.

**Assert:**
- `apply` exits 0
- FT-X front-matter now has `domains: [api, security]` and `adrs` includes ADR-Y
- MCP output: `{ "applied": true, "created": [], "changed": [{ "id": "FT-X", "mutations": 2 }] }` (shape TBD in implementation but the `changed` array is non-empty with the target)
- No new files created
- Applying the same request a second time produces the same file content (idempotent `append`)