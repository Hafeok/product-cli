---
id: TC-113
title: verify_unrunnable_no_block
type: scenario
status: passing
validates:
  features:
  - FT-023
  adrs:
  - ADR-021
phase: 1
runner: cargo-test
runner-args: tc_113_verify_unrunnable_no_block
last-run: 2026-04-13T14:07:16.920985096+00:00
---

all TCs have no `runner` field. Assert feature status unchanged. Assert W-class warning emitted.