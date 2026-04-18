---
id: TC-144
title: preflight_acknowledgement_without_reason_fails
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
runner-args: "tc_144_preflight_acknowledgement_without_reason_fails"
last-run: 2026-04-18T10:41:54.811678685+00:00
last-run-duration: 0.2s
---

run `product feature acknowledge FT-009 --domain security --reason ""`. Assert E011. Assert front-matter not mutated.