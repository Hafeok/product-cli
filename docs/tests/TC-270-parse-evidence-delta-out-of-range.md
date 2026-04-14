---
id: TC-270
title: parse_evidence_delta_out_of_range
type: scenario
status: unimplemented
validates:
  features: 
  - FT-003
  - FT-008
  - FT-015
  adrs:
  - ADR-016
phase: 1
---

parse `⟦Ε⟧⟨δ≜1.5;φ≜100;τ≜◊⁺⟩`. Assert E001 with the file path, line number, and the out-of-range value.