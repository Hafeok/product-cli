---
id: TC-826
title: graph_central_with_include_patterns_surfaces_pat_ids
type: scenario
status: passing
validates:
  features:
  - FT-071
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_826_graph_central_with_include_patterns_surfaces_pat_ids
last-run: 2026-05-27T13:37:22.762619987+00:00
last-run-duration: 0.2s
---

## Description

Compose a temp repo with three patterns and existing FT / ADR
nodes such that the patterns lie on at least one shortest path
between non-pattern nodes (Brandes betweenness will give them
non-zero centrality). Run `product graph central --include
patterns`.

Assert:

1. The output (text + JSON) lists at least one PAT id in the
   returned ranking.
2. The PAT entry carries a numeric centrality value.
3. Running with `--format json` returns a list of objects
   structurally identical to the existing FT/ADR/TC output
   (`{ id, centrality, title }`).

## Formal specification

⟦Λ:Scenario⟧
Given a repository whose graph topology places PAT nodes on
  shortest paths between non-pattern nodes,
When the user runs `product graph central --include patterns`,
Then the result includes at least one PAT id with a numeric
  centrality value,
And the JSON shape is consistent with the legacy output
  schema.

⟦Ε⟧⟨δ≜1.0;φ≜1;τ≜◊⁺⟩