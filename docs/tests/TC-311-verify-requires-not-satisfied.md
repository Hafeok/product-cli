---
id: TC-311
title: verify_requires_not_satisfied
type: scenario
status: passing
validates:
  features: 
  - FT-023
  adrs:
  - ADR-021
phase: 1
runner: cargo-test
runner-args: tc_311_verify_requires_not_satisfied
last-run: 2026-04-14T18:03:59.846825760+00:00
last-run-duration: 0.2s
---

TC with `requires: [two-node-cluster]`. Prerequisite command exits 1. Assert TC status becomes `unrunnable`, `failure-message` contains prerequisite name. Assert feature status unchanged.