---
id: TC-321
title: adr_review_missing_section
type: scenario
status: passing
validates:
  features: 
  - FT-022
  adrs:
  - ADR-022
phase: 1
runner: cargo-test
runner-args: "tc_321_adr_review_missing_section"
last-run: 2026-04-14T17:42:46.235479401+00:00
---

review ADR missing Rejected alternatives. Assert finding with file path and section name.