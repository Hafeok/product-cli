---
id: TC-101
title: mcp_http_no_token_401
type: contract
status: passing
validates:
  features:
  - FT-021
  adrs:
  - ADR-020
phase: 1
runner: cargo-test
runner-args: "tc_101_mcp_http_no_token_401"
last-run: 2026-04-30T09:23:08.718018813+00:00
last-run-duration: 0.4s
---

start server with `--token test`. Send request without Authorization header. Assert 401.