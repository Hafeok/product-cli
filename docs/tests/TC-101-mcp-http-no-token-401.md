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
last-run: 2026-04-29T03:12:37.040470115+00:00
last-run-duration: 0.3s
---

start server with `--token test`. Send request without Authorization header. Assert 401.