---
id: TC-074
title: parse_evidence_block
type: scenario
status: passing
validates:
  features:
  - FT-003
  - FT-004
  - FT-015
  adrs:
  - ADR-016
phase: 1
runner: cargo-test
runner-args: "tc_074_parse_evidence_block"
last-run: 2026-04-14T14:03:36.445391644+00:00
---

parse `⟦Ε⟧⟨δ≜0.95;φ≜100;τ≜◊⁺⟩`. Assert `delta=0.95`, `phi=100`, `tau=Stable`.