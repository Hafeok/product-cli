---
id: TC-1030
title: product domain new system captures a first-class system node
type: scenario
status: passing
validates:
  features:
  - FT-148
  adrs:
  - ADR-090
phase: 1
runner: cargo-test
runner-args: "tc_1030_system_is_a_first_class_what_node"
last-run: 2026-06-25T09:33:55.949776253+00:00
last-run-duration: 15.6s
---

## Description

§3.2.5 — `product domain new system` captures a system with its kind, purpose,
target platforms, and target interaction classes. A system missing its purpose
is rejected; a complete system validates and is serialized to Turtle under
`pf:System` with `pf:systemKind` and `pf:targetsClass`.