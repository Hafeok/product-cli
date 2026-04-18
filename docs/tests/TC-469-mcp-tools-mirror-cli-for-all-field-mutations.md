---
id: TC-469
title: MCP tools mirror CLI for all field mutations
type: scenario
status: passing
validates:
  features:
  - FT-038
  adrs:
  - ADR-037
phase: 1
runner: cargo-test
runner-args: "tc_469_mcp_tools_mirror_cli_for_all_field_mutations"
last-run: 2026-04-18T10:42:03.345580667+00:00
last-run-duration: 0.3s
---

For each new MCP tool (`product_feature_domain`, `product_feature_acknowledge`, `product_adr_domain`, `product_adr_scope`, `product_adr_supersede`, `product_adr_source_files`, `product_test_runner`): invoke the tool via the MCP server and assert the front-matter file is updated identically to the CLI equivalent. Assert all tools require `mcp.write = true` — calls with write disabled return a tool error.