---
id: TC-806
title: mcp_graph_check_surfaces_w030_functional_spec_finding
type: scenario
status: passing
validates:
  features:
  - FT-069
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_806_mcp_graph_check_surfaces_w030_functional_spec_finding
last-run: 2026-05-27T11:04:45.120555493+00:00
last-run-duration: 0.1s
---

## Scenario

The MCP `product_graph_check` tool surfaces W030 functional-spec
completeness findings on the same fixture the CLI does.

### Given

A temp repository fixture containing a non-abandoned feature whose
markdown body is missing one or more of the required H2 sections
defined by FT-055 / ADR-047 (`## Description`, `## Functional
Specification`, `## Out of scope`) **and** `[features].completeness-
severity = "warning"` (the default) in `product.toml`.

### When

The MCP client invokes the `product_graph_check` tool against the
fixture (via the compiled binary, JSON-RPC over stdio).

### Then

- The returned JSON envelope contains at least one finding with
  `code = "W030"` referencing the offending feature.
- The same finding appears when running `product graph check
  --format json` against the fixture.
- The set of `W030` codes in both envelopes is exactly equal.