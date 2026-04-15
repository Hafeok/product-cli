---
id: TC-443
title: W017 does not fire for planned feature with proposed ADR
type: scenario
status: passing
validates:
  features: [FT-036]
  adrs: [ADR-034]
phase: 1
runner: cargo-test
runner-args: "tc_443_w017_does_not_fire_for_planned_feature_with_proposed_adr"
last-run: 2026-04-15T10:35:59.328815871+00:00
last-run-duration: 0.3s
---

## Description

Create a feature with `status: planned` linked to an ADR with `status: proposed`. Run `product graph check`. Assert:

1. No W017 warning appears for this feature.
2. Linking a proposed ADR to a planned feature is valid forward-planning, not a lifecycle violation.