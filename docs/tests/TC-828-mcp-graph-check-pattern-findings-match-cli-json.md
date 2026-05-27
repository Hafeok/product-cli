---
id: TC-828
title: mcp_graph_check_pattern_findings_match_cli_json
type: scenario
status: passing
validates:
  features:
  - FT-071
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_828_mcp_graph_check_pattern_findings_match_cli_json
last-run: 2026-05-27T13:37:22.762619987+00:00
last-run-duration: 0.3s
---

## Description

Compose a temp repo that triggers every new diagnostic
introduced by FT-071: a requires-cycle PAT pair, a deprecated PAT
cited by a planned feature, and a live PAT with a missing body
section. Run `product_graph_check` over MCP and `product graph
check --format json` over the CLI; capture both JSON envelopes.

Assert:

1. Both envelopes contain all three findings with matching codes,
   messages, and file paths.
2. The two envelopes are byte-identical (FT-069 parity invariant
   generalised to the new diagnostics).
3. The `summary` block reports the correct error / warning totals
   on both sides.

## Formal specification

⟦Λ:Scenario⟧
Given a fixture triggering every new graph-check diagnostic from
  FT-071,
When the MCP `product_graph_check` and the CLI `product graph
  check --format json` are invoked against the same temp repo,
Then both JSON envelopes are byte-identical,
And the findings list contains the requires-cycle error, the
  deprecated-pattern warning, and the missing-body-section
  warning.

⟦Ε⟧⟨δ≜1.0;φ≜1;τ≜◊⁺⟩