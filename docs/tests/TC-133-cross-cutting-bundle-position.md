---
id: TC-133
title: cross_cutting_bundle_position
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
runner-args: "tc_133_cross_cutting_bundle_position"
---

assert cross-cutting ADRs appear before domain ADRs in the bundle, which appear before feature-linked ADRs.