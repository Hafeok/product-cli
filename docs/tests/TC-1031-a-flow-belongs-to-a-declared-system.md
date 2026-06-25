---
id: TC-1031
title: a flow belongs to a declared system
type: scenario
status: passing
validates:
  features:
  - FT-148
  adrs:
  - ADR-090
phase: 1
runner: cargo-test
runner-args: "tc_1031_a_flow_belongs_to_a_declared_system"
last-run: 2026-06-25T09:33:55.949776253+00:00
last-run-duration: 0.3s
---

## Description

§3.2.5 — a flow may declare the one system it belongs to. A flow owned by a
declared system is accepted and emits a `pf:systemOf` edge in the Turtle export;
a flow naming an undeclared system is a finding, rejected with no change made.