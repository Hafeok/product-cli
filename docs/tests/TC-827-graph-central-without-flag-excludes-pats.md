---
id: TC-827
title: graph_central_without_flag_excludes_pats
type: scenario
status: passing
validates:
  features:
  - FT-071
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_827_graph_central_without_flag_excludes_pats
last-run: 2026-05-27T13:37:22.762619987+00:00
last-run-duration: 0.2s
---

## Description

Compose the same fixture as TC-826. Run `product graph central`
(no `--include patterns` flag).

Assert:

1. The output contains zero PAT ids — the legacy behaviour is
   preserved.
2. The output is byte-identical to a snapshot captured against the
   same fixture rendered with patterns excluded via the legacy
   code path (regression guard for the AGENTS.md "top-5
   foundational ADRs" workflow).

## Formal specification

⟦Λ:Scenario⟧
Given a repository whose graph contains PAT nodes with non-zero
  centrality,
When the user runs `product graph central` with no
  `--include patterns` flag,
Then no PAT ids appear in the output,
And the output is byte-identical to the pre-FT-071 baseline for
  the same fixture.

⟦Ε⟧⟨δ≜1.0;φ≜1;τ≜◊⁺⟩