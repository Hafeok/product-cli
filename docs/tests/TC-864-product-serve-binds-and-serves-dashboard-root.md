---
id: TC-864
title: product serve binds and serves dashboard root
type: scenario
status: unimplemented
validates:
  features:
  - FT-105
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_864_product_serve_binds_and_serves_dashboard_root
observes:
- exit-code
- stdout
- mcp-response
---

**observes:** [exit-code, stdout, mcp-response]

Boot `product serve --port 0 --bind 127.0.0.1` from a temp repo
fixture. Capture the ephemeral port that the boot stdout prints
(`serving dashboard on http://127.0.0.1:<port>/`). Issue `GET /` and
assert the response is `200 OK`, `Content-Type: text/html;
charset=utf-8`, and the body contains the project name. Shut the
process down and assert exit code `0`.

Surfaces:
- **exit-code:** `0` on graceful shutdown.
- **stdout:** contains `serving dashboard on http://127.0.0.1:<port>/`.
- **mcp-response:** `200` HTML response on `GET /`.
