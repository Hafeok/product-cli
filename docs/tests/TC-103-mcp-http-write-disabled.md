---
id: TC-103
title: mcp_http_write_disabled
type: scenario
status: passing
validates:
  features:
  - FT-021
  adrs:
  - ADR-020
phase: 1
runner: cargo-test
runner-args: "tc_103_mcp_http_write_disabled"
last-run: 2026-04-14T17:29:27.893830767+00:00
---

start server with `mcp.write = false`. Call a write tool. Assert tool error (not HTTP error), message "write tools disabled".