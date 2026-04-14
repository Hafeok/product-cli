---
id: TC-300
title: mcp_http_graceful_shutdown
type: scenario
status: unimplemented
validates:
  features: 
  - FT-021
  adrs:
  - ADR-020
phase: 1
---

start server, send SIGTERM during an in-flight tool call. Assert the in-flight call completes before the process exits.