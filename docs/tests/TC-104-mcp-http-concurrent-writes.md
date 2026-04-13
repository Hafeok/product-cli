---
id: TC-104
title: mcp_http_concurrent_writes
type: scenario
status: passing
validates:
  features:
  - FT-021
  adrs:
  - ADR-020
phase: 1
runner: cargo-test
runner-args: "tc_104_mcp_http_concurrent_writes"
---

send two concurrent write tool calls. Assert one succeeds, one returns the lock-held error with PID.