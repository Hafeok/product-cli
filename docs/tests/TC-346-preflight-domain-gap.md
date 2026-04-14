---
id: TC-346
title: preflight_domain_gap
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

FT-009 declares `domains: [security]`, no security ADRs linked or acknowledged. Assert preflight reports security gap with the top-2 security ADRs by centrality named.