---
id: TC-307
title: verify_failure_message_written
type: scenario
status: passing
validates:
  features: 
  - FT-023
  adrs:
  - ADR-021
phase: 1
runner: cargo-test
runner-args: tc_307_verify_failure_message_written
last-run: 2026-04-18T10:41:51.294040135+00:00
last-run-duration: 0.2s
---

failing TC. Assert `failure-message` written with test output.