---
id: TC-469
title: MCP tools mirror CLI for all field mutations
type: scenario
status: unimplemented
validates:
  features: []
  adrs: []
phase: 1
---

For each new MCP tool (`product_feature_domain`, `product_feature_acknowledge`, `product_adr_domain`, `product_adr_scope`, `product_adr_supersede`, `product_adr_source_files`, `product_test_runner`): invoke the tool via the MCP server and assert the front-matter file is updated identically to the CLI equivalent. Assert all tools require `mcp.write = true` — calls with write disabled return a tool error.