---
id: TC-1032
title: trigger block issues a command
type: scenario
status: passing
validates:
  features:
  - FT-149
  adrs:
  - ADR-091
phase: 1
runner: cargo-test
runner-args: "tc_1032_trigger_block_issues_a_command"
last-run: 2026-06-25T10:21:08.278592637+00:00
last-run-duration: 18.4s
---

## Description

§3.2.0 — `product domain new trigger` captures a Trigger whose source is one of
user/external/automated and that issues a declared command. A bad source is
rejected; a user trigger issuing `PlaceOrder` validates and emits `pf:Trigger`
with a `pf:issues` edge in the Turtle export.