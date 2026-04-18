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
last-run: 2026-04-18T10:41:48.879855342+00:00
last-run-duration: 0.2s
---

review an ADR with empty `features: []`. Assert W001-class finding.