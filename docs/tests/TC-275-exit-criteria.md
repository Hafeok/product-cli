---
id: TC-275
title: '### Exit criteria'
type: exit-criteria
status: passing
validates:
  features: 
  - FT-020
  adrs:
  - ADR-017
phase: 1
runner: cargo-test
runner-args: "tc_275_exit_criteria_heading_context"
last-run: 2026-04-14T14:25:40.415822949+00:00
---

Bullets under a `### Exit criteria` heading in an ADR document produce test files with `type: exit-criteria`, even when the bullet title does not contain the word "exit". The heading context determines the default type for all bullets in that section.