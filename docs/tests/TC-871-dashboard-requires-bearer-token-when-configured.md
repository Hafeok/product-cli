---
id: TC-871
title: dashboard requires bearer token when configured
type: scenario
status: unimplemented
validates:
  features:
  - FT-105
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_871_dashboard_requires_bearer_token_when_configured
observes:
- mcp-response
---

**observes:** [mcp-response]

Boot the server with `--token secret123`. Assert:

1. `GET /` with no `Authorization` header → `401 Unauthorized`.
2. `GET /` with `Authorization: Bearer wrong` → `401`.
3. `GET /` with `Authorization: Bearer secret123` → `200`.
4. `GET /?token=secret123` (query param) → `200`.
5. `GET /api/status.json` with wrong token → `401` JSON envelope.
6. `GET /healthz` with no token → `200` (liveness probe is exempt).

Also assert that the access log line for a query-string token redacts
the token value.

Surface:
- **mcp-response:** auth gate enforced on every route except
  `/healthz`.
