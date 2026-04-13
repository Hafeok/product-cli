---
id: TC-160
title: FT-009 formal specification blocks parse
type: exit-criteria
status: passing
validates:
  features:
  - FT-004
  - FT-009
  adrs:
  - ADR-005
phase: 1
runner: cargo-test
runner-args: "tc_160_ft009_exit_criteria"
---

## Description

All FT-009 formal specification scenarios pass end-to-end: formal block types (⟦Σ:Types⟧, ⟦Γ:Invariants⟧, ⟦Λ:Scenario⟧, ⟦Ε⟧ evidence) are correctly parsed from test criterion files, preserved verbatim in context bundles, and validated through the graph check pipeline. Diagnostic reporting (E001 errors for out-of-range values, W004 warnings for empty blocks) functions correctly. Evidence aggregation (δ, φ, τ) is computed and surfaced in context output.