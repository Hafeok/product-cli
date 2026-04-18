---
id: TC-136
title: w010_unacknowledged_cross_cutting
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
runner-args: "tc_136_w010_unacknowledged_cross_cutting"
last-run: 2026-04-18T10:41:39.917241637+00:00
last-run-duration: 0.1s
---

ADR-013 is cross-cutting. FT-009 neither links nor acknowledges it. Run `product graph check`. Assert W010 naming FT-009 and ADR-013.