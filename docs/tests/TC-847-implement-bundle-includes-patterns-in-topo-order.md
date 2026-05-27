---
id: TC-847
title: implement_bundle_includes_patterns_in_topo_order
type: scenario
status: unimplemented
validates:
  features:
  - FT-074
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_847_implement_bundle_includes_patterns_in_topo_order
---

## Description

Compose a temp repo with PAT-A, PAT-B (requires PAT-A), and
FT-100 (`patterns: [PAT-B]`). Run `product implement FT-100
--dry-run` and capture the bundle that would be passed to the
agent (stdout under `--dry-run`).

Assert:

1. The captured bundle contains a "## Patterns" section.
2. Both PAT-A's body and PAT-B's body are present (transitive
   over `requires:`).
3. PAT-A precedes PAT-B in the output (topo order).
4. A sibling fixture where PAT-A and PAT-B have no `requires:`
   edge produces both patterns in either order — the topo
   constraint applies only when an edge exists.

## Formal specification

⟦Λ:Scenario⟧
Given FT-100 citing PAT-B which requires PAT-A,
When the user runs `product implement FT-100 --dry-run`,
Then the captured bundle contains "## Patterns" with both PATs,
And PAT-A appears before PAT-B in the rendered output.

⟦Ε⟧⟨δ≜1.0;φ≜1;τ≜◊⁺⟩
