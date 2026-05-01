---
id: TC-312
title: verify_requires_missing_prereq_def
type: scenario
status: passing
validates:
  features: 
  - FT-023
  adrs:
  - ADR-021
phase: 1
runner: cargo-test
runner-args: tc_312_verify_requires_missing_prereq_def
last-run: 2026-04-30T09:23:18.004925059+00:00
last-run-duration: 0.3s
---

TC requires a prerequisite not defined in `product.toml`. Assert E-class error with the prerequisite name and a hint to add it to `[verify.prerequisites]`.