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
last-run: 2026-04-18T10:41:43.286383101+00:00
last-run-duration: 0.4s
---

start server, send SIGTERM during an in-flight tool call. Assert the in-flight call completes before the process exits.