---
id: TC-103
title: mcp_http_write_disabled
type: scenario
status: unimplemented
validates:
  features:
  - FT-021
  adrs:
  - ADR-020
phase: 1
---

start server with `mcp.write = false`. Call a write tool. Assert tool error (not HTTP error), message "write tools disabled".