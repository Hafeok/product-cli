---
id: TC-410
title: AGENT.md contains working protocol section
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

Run `product agent-init`. Assert `AGENT.md` contains a "Working Protocol" section listing the expected sequence of MCP calls: `product_graph_check`, `product_graph_central`, `product_feature_list`, `product_context`.
