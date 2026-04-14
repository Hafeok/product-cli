---
id: TC-311
title: verify_requires_not_satisfied
type: scenario
status: unimplemented
validates:
  features: 
  - FT-023
  adrs:
  - ADR-021
phase: 1
---

TC with `requires: [two-node-cluster]`. Prerequisite command exits 1. Assert TC status becomes `unrunnable`, `failure-message` contains prerequisite name. Assert feature status unchanged.