---
id: TC-038
title: formal_block_evidence
type: scenario
status: passing
validates:
  features:
  - FT-015
  adrs:
  - ADR-011
phase: 1
runner: cargo-test
runner-args: "tc_038_formal_block_evidence"
last-run: 2026-04-14T14:03:36.445391644+00:00
---

parse `⟦Ε⟧⟨δ≜0.95;φ≜100;τ≜◊⁺⟩`. Assert `delta=0.95`, `phi=100`, `tau=Stable`.