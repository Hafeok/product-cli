---
id: TC-103
title: mcp_http_write_disabled
type: contract
status: passing
validates:
  features:
  - FT-021
  adrs:
  - ADR-020
phase: 1
runner: cargo-test
runner-args: "tc_103_mcp_http_write_disabled"
last-run: 2026-04-18T10:41:43.286383101+00:00
last-run-duration: 0.2s
---

start server with `mcp.write = false`. Call a write tool. Assert tool error (not HTTP error), message "write tools disabled".