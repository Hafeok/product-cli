---
id: TC-101
title: mcp_http_no_token_401
type: scenario
status: passing
validates:
  features:
  - FT-021
  adrs:
  - ADR-020
phase: 1
runner: cargo-test
runner-args: "tc_101_mcp_http_no_token_401"
last-run: 2026-04-18T10:41:43.286383101+00:00
last-run-duration: 0.2s
---

start server with `--token test`. Send request without Authorization header. Assert 401.