---
id: TC-844
title: graph_check_advisory_for_feature_with_no_patterns_when_enabled
type: scenario
status: unimplemented
validates:
  features:
  - FT-073
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_844_graph_check_advisory_for_feature_with_no_patterns_when_enabled
---

## Description

Compose a temp repo with `[features].patterns-required-severity
= "warning"` and an `in-progress` feature FT-100 with
`patterns: []`. Run `product graph check` and capture
stdout/exit code. Then flip the config to `severity = "off"`
and rerun.

Assert:

1. With `severity = "warning"`, the output contains the new
   advisory warning code naming FT-100.
2. With `severity = "off"`, the warning is absent.
3. With `severity = "error"`, the same finding escalates and the
   exit code is non-zero.
4. A feature whose `patterns` array is non-empty never trips this
   check at any severity.

## Formal specification

⟦Λ:Scenario⟧
Given FT-100 (in-progress, patterns: []) and configurable
  severity,
When the user runs `product graph check` at each severity
  setting,
Then severity=warning emits the advisory naming FT-100,
And severity=off silences the advisory,
And severity=error escalates the finding to error tier with
  non-zero exit.

⟦Ε⟧⟨δ≜1.0;φ≜1;τ≜◊⁺⟩
