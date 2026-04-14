---
id: TC-338
title: domain_top2_centrality
type: scenario
status: unimplemented
validates:
  features: 
  - FT-018
  - FT-019
  adrs:
  - ADR-025
phase: 1
---

domain `security` has 6 ADRs with known centrality scores. Feature FT-009 declares `domains: [security]` with no acknowledged ADRs. Assert the context bundle includes exactly the 2 highest-centrality security ADRs.