---
id: TC-842
title: mcp_feature_link_with_pattern_arg_writes_to_disk
type: scenario
status: unimplemented
validates:
  features:
  - FT-073
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_842_mcp_feature_link_with_pattern_arg_writes_to_disk
---

## Description

Compose a temp repo with PAT-001 and FT-100. Invoke
`product_feature_link` over MCP with `{ id: "FT-100", pattern:
"PAT-001" }`.

Assert:

1. The MCP response carries the FT-066 writes/reciprocated
   shape.
2. The on-disk files match the CLI invocation byte-for-byte
   (parity invariant against a sibling temp repo).
3. An empty MCP response without a disk write fails the TC even
   when the envelope is well-formed (explicit FT-046 anti-stub
   guard).

## Formal specification

⟦Λ:Scenario⟧
Given a repository with PAT-001 and FT-100,
When an MCP client invokes `product_feature_link` with
  `{ id: "FT-100", pattern: "PAT-001" }`,
Then the on-disk files reflect the link bidirectionally,
And the result is byte-identical to the CLI equivalent against
  a sibling temp repo,
And an envelope-only response with no disk write fails this TC.

⟦Ε⟧⟨δ≜1.0;φ≜1;τ≜◊⁺⟩
