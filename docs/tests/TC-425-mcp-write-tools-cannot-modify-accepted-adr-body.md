---
id: TC-425
title: MCP write tools cannot modify accepted ADR body
type: scenario
status: passing
validates:
  features: [FT-034]
  adrs: [ADR-032]
phase: 1
runner: cargo-test
runner-args: "tc_425_mcp_write_tools_cannot_modify_accepted_adr_body"
last-run: 2026-04-14T14:44:11.097422144+00:00
---

## Description

Start the MCP server. Create and accept an ADR. Attempt to call any MCP write tool that would modify the body of the accepted ADR. Verify the tool returns an error indicating that the body of an accepted ADR cannot be modified. Verify that `product_adr_status` (which only touches front-matter) still works. Verify that `product_feature_link` (which adds to the `features` array, excluded from hash) still works.