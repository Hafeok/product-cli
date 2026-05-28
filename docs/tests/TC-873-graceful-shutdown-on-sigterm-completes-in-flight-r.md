---
id: TC-873
title: graceful shutdown on SIGTERM completes in-flight requests
type: chaos
status: unimplemented
validates:
  features:
  - FT-105
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_873_graceful_shutdown_on_sigterm_completes_in_flight_requests
runner-timeout: 30
observes:
- exit-code
- mcp-response
---

**observes:** [exit-code, mcp-response]

Boot the server. Start a slow `GET /` request that the handler is
forced to stall on (test seam: inject a sleep into the loader for this
fixture). Send `SIGTERM` to the process while the request is in
flight. Assert:

1. The in-flight `GET /` completes with `200`.
2. A new `GET /` issued after the SIGTERM is rejected with a
   connection error (listener stopped accepting).
3. The process exits with code `0` within the 10-second drain budget.

Repeat with `SIGINT` — identical behaviour.

Surfaces:
- **exit-code:** `0` after graceful shutdown.
- **mcp-response:** in-flight request finishes cleanly; post-signal
  request fails.
