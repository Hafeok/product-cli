---
id: TC-104
title: mcp_http_concurrent_writes
type: scenario
status: unimplemented
validates:
  features:
  - FT-021
  adrs:
  - ADR-020
phase: 1
---

send two concurrent write tool calls. Assert one succeeds, one returns the lock-held error with PID.