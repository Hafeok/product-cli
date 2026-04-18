---
id: TC-132
title: cross_cutting_always_in_bundle
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
runner-args: "tc_132_cross_cutting_always_in_bundle"
last-run: 2026-04-18T10:41:39.917241637+00:00
last-run-duration: 0.2s
---

ADR-013 marked `scope: cross-cutting`. Feature FT-009 has no explicit link to ADR-013. Assert `product context FT-009` includes ADR-013 in the bundle.