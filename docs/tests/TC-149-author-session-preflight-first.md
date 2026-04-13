---
id: TC-149
title: author_session_preflight_first
type: scenario
status: unimplemented
validates:
  features:
  - FT-019
  - FT-027
  adrs:
  - ADR-026
phase: 1
---

start `product author feature` for FT-009 with preflight gaps. Assert the first MCP tool call from the session is `product_preflight`, not a content scaffold call.