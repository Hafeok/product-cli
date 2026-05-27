---
id: TC-808
title: mcp_graph_check_surfaces_w028_due_date_finding
type: scenario
status: passing
validates:
  features:
  - FT-069
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_808_mcp_graph_check_surfaces_w028_due_date_finding
last-run: 2026-05-27T11:04:45.120555493+00:00
last-run-duration: 0.1s
---

## Scenario

The MCP `product_graph_check` tool surfaces W028 due-date-passed
findings (FT-053 / ADR-045) on the same fixture the CLI does.

### Given

A temp repository fixture containing a non-complete feature whose
`due-date` front-matter field is set to a date earlier than the
reference `today` (which the test pins via fixture-controlled clock
or via `due-date: 1970-01-01`).

### When

The MCP client invokes `product_graph_check` against the fixture.

### Then

- The returned JSON envelope contains a finding with `code = "W028"`
  naming the overdue feature.
- The CLI `product graph check --format json` returns the same
  finding.
- The set of `W028` codes in both envelopes is exactly equal.