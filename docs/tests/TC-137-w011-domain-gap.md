---
id: TC-137
title: w011_domain_gap
type: scenario
status: passing
validates:
  features:
  - FT-018
  - FT-019
  adrs:
  - ADR-025
phase: 1
---

FT-009 declares `domains: [security]`. Security domain has ADRs. FT-009 neither links nor acknowledges security. Assert W011.