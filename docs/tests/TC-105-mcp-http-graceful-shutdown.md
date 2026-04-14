---
id: TC-105
title: mcp_http_graceful_shutdown
type: scenario
status: passing
validates:
  features:
  - FT-021
  adrs:
  - ADR-020
phase: 1
runner: cargo-test
runner-args: "tc_105_mcp_http_graceful_shutdown"
last-run: 2026-04-14T17:29:27.893830767+00:00
---

start server, send SIGTERM during an in-flight tool call. Assert the in-flight call completes before the process exits.