---
id: TC-102
title: mcp_http_wrong_token_401
type: contract
status: passing
validates:
  features:
  - FT-021
  adrs:
  - ADR-020
phase: 1
runner: cargo-test
runner-args: "tc_102_mcp_http_wrong_token_401"
last-run: 2026-04-30T09:23:08.718018813+00:00
last-run-duration: 0.3s
---

send request with wrong bearer token. Assert 401.