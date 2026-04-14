---
id: TC-410
title: AGENT.md contains working protocol section
type: scenario
status: passing
validates:
  features:
  - FT-033
  adrs:
  - ADR-031
phase: 3
runner: cargo-test
runner-args: "tc_410_agent_md_contains_working_protocol_section"
last-run: 2026-04-14T17:21:07.545864789+00:00
---

## Description

Run `product agent-init`. Assert `AGENT.md` contains a "Working Protocol" section listing the expected sequence of MCP calls: `product_graph_check`, `product_graph_central`, `product_feature_list`, `product_context`.