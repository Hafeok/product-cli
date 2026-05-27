---
id: TC-824
title: graph_check_deprecated_pattern_cited_by_live_feature_emits_warning
type: scenario
status: passing
validates:
  features:
  - FT-071
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_824_graph_check_deprecated_pattern_cited_by_live_feature_emits_warning
last-run: 2026-05-27T13:37:22.762619987+00:00
last-run-duration: 0.3s
---

## Description

Compose a temp repo with PAT-A (status `deprecated`,
`deprecated-by: PAT-B`) and feature FT-100 (status `planned`,
`patterns: [PAT-A]`). Run `product graph check`.

Assert:

1. The stdout contains the new warning code (allocated by this
   feature) and the substring `deprecated`.
2. The exit code is non-zero only if `[graph-check].severity =
   "error"` for this code; default warning severity exits 2 per
   ADR-013 if no errors exist, or 0 if warnings alone are
   non-blocking — match whichever the existing graph-check
   behaviour does for W-tier findings.
3. A `complete` or `abandoned` feature citing the same PAT does
   **not** emit the warning (exemption verified by a second
   sub-assertion or sibling fixture).

## Formal specification

⟦Λ:Scenario⟧
Given a repository where PAT-A is deprecated and FT-100 (status
  planned) cites PAT-A,
When the user runs `product graph check`,
Then the output emits the new deprecated-pattern-cited warning
  naming FT-100 and PAT-A,
And a sibling fixture with FT-100 status complete does not emit
  the warning.

⟦Ε⟧⟨δ≜1.0;φ≜1;τ≜◊⁺⟩