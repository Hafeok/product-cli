---
id: TC-120
title: adr_review_structural_no_features
type: scenario
status: passing
validates:
  features:
  - FT-022
  adrs:
  - ADR-022
phase: 1
runner: cargo-test
runner-args: "tc_120_adr_review_structural_no_features"
last-run: 2026-04-14T17:42:46.235479401+00:00
---

review an ADR with empty `features: []`. Assert W001-class finding.