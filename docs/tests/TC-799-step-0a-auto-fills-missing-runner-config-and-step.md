---
id: TC-799
title: Step 0a auto-fills missing runner config and Step 0 preflight passes
type: scenario
status: passing
validates:
  features:
  - FT-068
  adrs: []
phase: 5
runner: cargo-test
runner-args: tc_799_step_0a_autofills_missing_runner_config
runner-timeout: 120
last-run: 2026-05-26T12:31:19.240471574+00:00
last-run-duration: 0.3s
---

## Scenario

A feature `FT-XXX` is `planned` and has a single linked TC whose
front-matter has no `runner` or `runner-args` field.

## When

The user runs `product implement FT-XXX --dry-run`.

## Then

Step 0a (auto-fill TC runner config) executes before Step 0
(preflight). It detects the missing runner fields, derives
`tc_NNN_<slug>` from the TC's markdown filename, and writes
`runner: cargo-test`, `runner-args: tc_NNN_<slug>`,
`runner-timeout: 120s` to the TC's front-matter.

A diagnostic line is printed:
`pre-flight: TC-NNN missing runner config — auto-setting
runner=cargo-test args=tc_NNN_<slug> timeout=120s`.

Step 0 (preflight) then runs successfully because runner config is
present on every linked TC. The pipeline continues to context
assembly. Exit code is 0.