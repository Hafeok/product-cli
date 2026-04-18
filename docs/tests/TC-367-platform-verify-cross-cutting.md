---
id: TC-367
title: platform_verify_cross_cutting
type: scenario
status: passing
validates:
  features: 
  - FT-030
  adrs:
  - ADR-027
phase: 1
runner: cargo-test
runner-args: "tc_367_platform_verify_cross_cutting"
last-run: 2026-04-18T10:41:58.940827843+00:00
last-run-duration: 0.2s
---

run `product verify --platform`. Assert TCs linked to cross-cutting ADRs are run. Assert their status is updated. Assert feature-specific TCs are not run.