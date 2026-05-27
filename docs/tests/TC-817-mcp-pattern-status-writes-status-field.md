---
id: TC-817
title: mcp_pattern_status_writes_status_field
type: scenario
status: unimplemented
validates:
  features:
  - FT-070
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_817_mcp_pattern_status_writes_status_field
---

## Description

Compose a temp repo containing pattern `PAT-001` (status `live`) and
pattern `PAT-042` (status `live`). Invoke `product_pattern_status`
over MCP with `{ id: "PAT-001", status: "deprecated", deprecated_by:
"PAT-042" }`.

Assert:

1. The MCP response carries `{ id: "PAT-001", status: "deprecated",
   previous-status: "live", deprecated-by: "PAT-042" }`.
2. The on-disk front-matter of `docs/patterns/PAT-001-*.md` reads
   `status: deprecated` and `deprecated-by: PAT-042`.
3. The file is byte-identical to the file produced by the equivalent
   CLI invocation against a sibling temp repo.
4. Invoking `product_pattern_status` over MCP with `{ id: "PAT-001",
   status: "live" }` removes the `deprecated-by` field from the
   front-matter.

## Formal specification

⟦Λ:Scenario⟧
Given a repository containing live patterns PAT-001 and PAT-042,
When an MCP client invokes `product_pattern_status` with
  `{ id: "PAT-001", status: "deprecated", deprecated_by: "PAT-042" }`,
Then the file PAT-001 on disk reads `status: deprecated` and
  `deprecated-by: PAT-042`,
And the MCP response reports the transition with both fields,
And the same payload submitted to the CLI produces a byte-identical
  file.

⟦Ε⟧⟨δ≜1.0;φ≜1;τ≜◊⁺⟩
