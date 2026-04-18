---
id: TC-310
title: verify_requires_satisfied
type: scenario
status: passing
validates:
  features: 
  - FT-023
  adrs:
  - ADR-021
phase: 1
runner: cargo-test
runner-args: tc_310_verify_requires_satisfied
last-run: 2026-04-18T10:41:51.294040135+00:00
last-run-duration: 0.2s
---

TC with `requires: [binary-compiled]`. Prerequisite command exits 0. Assert TC runs normally.