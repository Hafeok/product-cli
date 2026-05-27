---
id: TC-837
title: mcp_graph_check_observes_findings_match_cli_json
type: scenario
status: passing
validates:
  features:
  - FT-072
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_837_mcp_graph_check_observes_findings_match_cli_json
observes:
- mcp-response
- stdout
last-run: 2026-05-27T14:11:07.133454142+00:00
last-run-duration: 0.3s
---

## Description

Compose a temp repo with both an `observes:`-missing TC and an
`observes:`-with-no-body-reference TC. Invoke `product_graph_check`
over MCP and `product graph check --format json` over the CLI.

Assert:

1. Both JSON envelopes contain the missing-`observes:` error.
2. Both contain the body-reference warning.
3. The envelopes are byte-identical (FT-069 parity invariant
   generalised).
4. The `summary` block totals match between MCP and CLI.

## Formal specification

⟦Λ:Scenario⟧
Given a fixture triggering both new observes-related
  diagnostics,
When `product_graph_check` (MCP) and `product graph check
  --format json` (CLI) run against the same repo,
Then both JSON envelopes are byte-identical,
And the findings include both the missing-observes error and
  the body-reference warning.

⟦Ε⟧⟨δ≜1.0;φ≜1;τ≜◊⁺⟩