---
id: TC-322
title: adr_review_no_features
type: scenario
status: passing
validates:
  features: 
  - FT-022
  adrs:
  - ADR-022
phase: 1
runner: cargo-test
runner-args: "tc_322_adr_review_no_features"
last-run: 2026-04-30T09:23:14.884691727+00:00
last-run-duration: 0.3s
---

review ADR with `features: []`. Assert W001-class finding.