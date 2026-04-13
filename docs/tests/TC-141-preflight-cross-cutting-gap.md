---
id: TC-141
title: preflight_cross_cutting_gap
type: scenario
status: passing
validates:
  features:
  - FT-019
  - FT-027
  adrs:
  - ADR-026
phase: 1
runner: cargo-test
runner-args: "tc_141_preflight_cross_cutting_gap"
last-run: 2026-04-13T14:12:26.396687298+00:00
---

ADR-038 is cross-cutting, not linked or acknowledged by FT-009. Assert preflight report names ADR-038. Assert exit code 1.