---
id: TC-134
title: domain_top2_centrality
type: scenario
status: passing
validates:
  features:
  - FT-018
  - FT-019
  adrs:
  - ADR-025
phase: 1
runner: cargo-test
runner-args: "tc_134_domain_top2_centrality"
---

domain `security` has 6 ADRs with known centrality scores. Feature FT-009 declares `domains: [security]` with no acknowledged ADRs. Assert the context bundle includes exactly the 2 highest-centrality security ADRs.