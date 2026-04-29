---
id: TC-365
title: reverse_inference_updates_feature
type: scenario
status: passing
validates:
  features: 
  - FT-030
  adrs:
  - ADR-027
phase: 1
runner: cargo-test
runner-args: "tc_365_reverse_inference_updates_feature"
last-run: 2026-04-29T03:12:55.121081119+00:00
last-run-duration: 0.2s
---

after inference adds FT-001 to TC-002.validates.features, assert FT-001.tests now includes TC-002.