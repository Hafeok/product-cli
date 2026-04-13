---
id: TC-101
title: mcp_http_no_token_401
type: scenario
status: unimplemented
validates:
  features:
  - FT-021
  adrs:
  - ADR-020
phase: 1
---

start server with `--token test`. Send request without Authorization header. Assert 401.