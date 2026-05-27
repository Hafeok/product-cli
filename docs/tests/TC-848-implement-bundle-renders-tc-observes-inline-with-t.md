---
id: TC-848
title: implement_bundle_renders_tc_observes_inline_with_tc_body
type: scenario
status: unimplemented
validates:
  features:
  - FT-074
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_848_implement_bundle_renders_tc_observes_inline_with_tc_body
---

## Description

Compose a temp repo with FT-100 linked to two TCs: TC-A
(`observes: [file]`) and TC-B (`observes: [graph, mcp-response]`).
Run `product implement FT-100 --dry-run` and capture the bundle.

Assert:

1. For TC-A, the bundle contains a line of the form `observes:
   [file]` adjacent to (immediately before or directly inside)
   the TC body block.
2. For TC-B, the equivalent `observes: [graph, mcp-response]`
   line is present.
3. The observes lines render in the rendered order of the TC
   listing — the table is not separately collated.

## Formal specification

⟦Λ:Scenario⟧
Given FT-100 with TC-A (observes: [file]) and TC-B (observes:
  [graph, mcp-response]),
When the user runs `product implement FT-100 --dry-run`,
Then the bundle renders each TC's observes list adjacent to
  its body,
And the lines appear inline rather than in a separate collated
  table.

⟦Ε⟧⟨δ≜1.0;φ≜1;τ≜◊⁺⟩
