---
id: TC-309
title: verify_platform_runs_cross_cutting
type: scenario
status: passing
validates:
  features: 
  - FT-023
  adrs:
  - ADR-021
phase: 1
runner: cargo-test
runner-args: tc_309_verify_platform_runs_cross_cutting
last-run: 2026-04-18T10:41:51.294040135+00:00
last-run-duration: 0.2s
---

run `product verify --platform`. Assert TCs linked to cross-cutting ADRs run. Assert feature-specific TCs not run.