---
id: TC-818
title: pattern_status_to_live_clears_deprecated_by
type: scenario
status: unimplemented
validates:
  features:
  - FT-070
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_818_pattern_status_to_live_clears_deprecated_by
---

## Description

Compose a temp repo with pattern `PAT-001` already at
`status: deprecated`, `deprecated-by: PAT-042`. Run
`product pattern status PAT-001 live`.

Assert:

1. After the transition, the on-disk front-matter reads
   `status: live`.
2. The `deprecated-by:` field is **absent** from the front-matter
   (not merely empty-string; the line is removed by the writer).
3. Reading the file back through the parser yields
   `deprecated_by: None`.

## Formal specification

⟦Λ:Scenario⟧
Given a repository containing PAT-001 with `status: deprecated,
  deprecated-by: PAT-042`,
When the user runs `product pattern status PAT-001 live`,
Then PAT-001's front-matter reads `status: live`,
And the `deprecated-by` line is no longer present in the
  serialised YAML,
And the parser exposes `deprecated_by: None` for PAT-001.

⟦Ε⟧⟨δ≜1.0;φ≜1;τ≜◊⁺⟩
