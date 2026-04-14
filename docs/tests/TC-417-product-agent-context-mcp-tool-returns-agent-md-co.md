---
id: TC-417
title: product_agent_context MCP tool returns AGENT.md content
type: scenario
status: unimplemented
validates:
  features:
  - FT-033
  adrs:
  - ADR-031
phase: 3
---

## Description

Run `product agent-init` to generate `AGENT.md`. Call `product_agent_context` MCP read tool. Assert the response content matches the content of the generated `AGENT.md` file on disk.
