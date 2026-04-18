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
last-run: 2026-04-18T10:41:54.811678685+00:00
last-run-duration: 0.2s
---

ADR-038 is cross-cutting, not linked or acknowledged by FT-009. Assert preflight report names ADR-038. Assert exit code 1.