---
id: TC-875
title: product serve exit-criteria — boot, browse, shutdown lifecycle
type: exit-criteria
status: unimplemented
validates:
  features:
  - FT-105
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_875_product_serve_exit_criteria_boot_browse_shutdown_lifecycle
runner-timeout: 30
observes:
- exit-code
- stdout
- mcp-response
---

**observes:** [exit-code, stdout, mcp-response]

**Exit-criteria for FT-105.** A single scripted scenario that
exercises the full lifecycle end-to-end:

1. Boot `product serve --port 0 --token testtok`. Parse the bound
   port from stdout.
2. `GET /healthz` (no token) → `200 ok`.
3. `GET /?token=testtok` → `200` HTML containing the project name.
4. `GET /features?token=testtok` → `200` HTML containing every seeded
   feature ID.
5. `GET /api/status.json?token=testtok` → `200` JSON matching
   `product status --format json`.
6. `GET /admin` (unknown route) → `404`.
7. `POST /` → `405`.
8. Send SIGTERM. Process exits `0` within 10 seconds.

This TC gates feature completion. If any step fails, the feature is
not done.

Surfaces:
- **exit-code:** `0` on graceful shutdown.
- **stdout:** bound URL is printed at boot.
- **mcp-response:** every HTTP step returns the expected status and
  body shape.
