---
id: TC-869
title: dashboard rejects every non-GET method with 405
type: invariant
status: unimplemented
validates:
  features:
  - FT-105
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_869_dashboard_rejects_every_non_get_method_with_405
observes:
- mcp-response
---

**observes:** [mcp-response]

For each route in `{/, /features, /features/FT-001, /adrs, /adrs/ADR-001,
/tests, /api/status.json, /api/features.json, /api/adrs.json, /healthz}`,
issue `POST`, `PUT`, `PATCH`, `DELETE` with an empty body. Every
response MUST be `405 Method Not Allowed` with `Allow: GET` header.

`OPTIONS` requests are allowed only when CORS is configured; otherwise
also `405`.

Surface:
- **mcp-response:** `405` status code on every non-GET method.
