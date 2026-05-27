---
id: TC-816
title: mcp_pattern_new_writes_to_disk
type: scenario
status: unimplemented
validates:
  features:
  - FT-070
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_816_mcp_pattern_new_writes_to_disk
---

## Description

Generalisation of the FT-066 TC-778 shape to the new pattern surface.
Compose a temp repo. Invoke `product_pattern_new` over MCP with
`{ title: "Slice + Adapter module structure" }`. Capture both the
MCP JSON-RPC response **and** the on-disk file produced.

Assert:

1. The MCP response is a success envelope containing
   `{ id: "PAT-001", path: "<absolute>" }`.
2. The file at `<absolute>` exists on disk and parses to a
   `PatternFrontMatter` with the correct id, title, status.
3. The file is byte-identical to the file produced by running
   `product pattern new "Slice + Adapter module structure"` against
   a sibling temp repo with the same configuration (MCP / CLI parity
   — FT-066 invariant generalisation).
4. The legacy "envelope without disk write" anti-pattern is absent:
   if the file does not exist, the TC fails even when the response
   looks correct. This is the explicit lesson from FT-046 → FT-066
   codified in this TC.

## Formal specification

⟦Λ:Scenario⟧
Given a fresh temp repo with patterns configured,
When an MCP client invokes `product_pattern_new` with
  `{ title: "Slice + Adapter module structure" }`,
Then the MCP response envelope reports
  `{ id: "PAT-001", path: P }`,
And the file at P exists on disk,
And its content is byte-identical to the file produced by the
  equivalent CLI invocation against a sibling temp repo,
And the `tests/sessions/` harness fails the TC when the file is
  missing, even if the response envelope is well-formed.

⟦Ε⟧⟨δ≜1.0;φ≜1;τ≜◊⁺⟩
