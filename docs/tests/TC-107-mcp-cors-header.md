---
id: TC-107
title: mcp_cors_header
type: scenario
status: passing
validates:
  features:
  - FT-021
  adrs:
  - ADR-020
phase: 1
runner: cargo-test
runner-args: "tc_107_mcp_cors_header"
last-run: 2026-04-18T10:41:43.286383101+00:00
last-run-duration: 0.3s
---

configure `cors-origins = ["https://claude.ai"]`. Assert CORS response headers are correct for a preflight request from that origin.