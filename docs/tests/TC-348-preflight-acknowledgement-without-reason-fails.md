---
id: TC-348
title: preflight_acknowledgement_without_reason_fails
type: scenario
status: unimplemented
validates:
  features: 
  - FT-019
  - FT-027
  adrs:
  - ADR-026
phase: 1
---

run `product feature acknowledge FT-009 --domain security --reason ""`. Assert E011. Assert front-matter not mutated.