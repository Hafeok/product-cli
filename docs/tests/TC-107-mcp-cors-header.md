---
id: TC-107
title: mcp_cors_header
type: scenario
status: unimplemented
validates:
  features:
  - FT-021
  adrs:
  - ADR-020
phase: 1
---

configure `cors-origins = ["https://claude.ai"]`. Assert CORS response headers are correct for a preflight request from that origin.