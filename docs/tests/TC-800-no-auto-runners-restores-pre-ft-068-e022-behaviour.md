---
id: TC-800
title: --no-auto-runners restores pre-FT-068 E022 behaviour
type: scenario
status: passing
validates:
  features:
  - FT-068
  adrs: []
phase: 5
runner: cargo-test
runner-args: tc_800_no_auto_runners_restores_e022
runner-timeout: 120
last-run: 2026-05-26T12:31:19.240471574+00:00
last-run-duration: 0.2s
---

## Scenario

A feature `FT-XXX` is `planned` and has one linked TC with no
runner or runner-args configured.

## When

The user runs `product implement FT-XXX --no-auto-runners`.

## Then

Step 0a is skipped entirely (the opt-out flag disables the
auto-fill). Step 0 fires `error[E022]: TC runner configuration
missing` naming TC-NNN. The agent is not invoked. The exit code
is 22 (matching the existing FT-058 contract).

The TC's front-matter on disk is unchanged — nothing was written.
This proves the opt-out path is exactly the pre-FT-068 behaviour.