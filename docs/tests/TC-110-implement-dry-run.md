---
id: TC-110
title: implement_dry_run
type: scenario
status: passing
validates:
  features:
  - FT-023
  adrs:
  - ADR-021
phase: 1
runner: cargo-test
runner-args: tc_110_implement_dry_run
last-run: 2026-04-13T14:07:16.920985096+00:00
---

run `product implement FT-001 --dry-run`. Assert temp file is created and its path printed. Assert no agent is invoked.