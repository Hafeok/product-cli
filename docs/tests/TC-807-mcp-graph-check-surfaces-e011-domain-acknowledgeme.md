---
id: TC-807
title: mcp_graph_check_surfaces_e011_domain_acknowledgement_finding
type: scenario
status: passing
validates:
  features:
  - FT-069
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_807_mcp_graph_check_surfaces_e011_domain_acknowledgement_finding
last-run: 2026-05-27T11:04:45.120555493+00:00
last-run-duration: 0.1s
---

## Scenario

The MCP `product_graph_check` tool surfaces E011 domain-acknowledgement
findings on the same fixture the CLI does.

### Given

A temp repository fixture containing a feature whose front-matter
includes `domains-acknowledged.<some-domain>: ""` (acknowledgement
without reasoning, ADR-025 / ADR-026).

### When

The MCP client invokes `product_graph_check` against the fixture.

### Then

- The returned JSON envelope contains a finding with `code = "E011"`
  naming the feature and the empty-reason domain.
- The CLI `product graph check --format json` returns the same
  finding.
- The set of `E011` codes in both envelopes is exactly equal.