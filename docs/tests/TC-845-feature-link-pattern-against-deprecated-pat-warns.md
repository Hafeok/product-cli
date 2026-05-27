---
id: TC-845
title: feature_link_pattern_against_deprecated_pat_warns_but_writes
type: scenario
status: unimplemented
validates:
  features:
  - FT-073
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_845_feature_link_pattern_against_deprecated_pat_warns_but_writes
---

## Description

Compose a temp repo with PAT-001 (`status: deprecated`) and
FT-100. Run `product feature link FT-100 --pattern PAT-001`.

Assert:

1. The exit code is 0 — the write succeeds.
2. Stderr contains a deprecation warning naming PAT-001.
3. Both files are written bidirectionally (the deprecation does
   not block the link).
4. Subsequent `product graph check` emits the
   deprecated-pattern-cited warning per FT-071 (which fires
   regardless of which command added the link).

## Formal specification

⟦Λ:Scenario⟧
Given a deprecated PAT-001 and feature FT-100,
When the user runs `product feature link FT-100 --pattern
  PAT-001`,
Then the command exits 0 with a deprecation warning on stderr,
And both files are written bidirectionally,
And `product graph check` subsequently emits FT-071's
  deprecated-pattern-cited warning.

⟦Ε⟧⟨δ≜1.0;φ≜1;τ≜◊⁺⟩
