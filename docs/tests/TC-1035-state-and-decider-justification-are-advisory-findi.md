---
id: TC-1035
title: state and decider justification are advisory findings
type: invariant
status: passing
validates:
  features:
  - FT-151
  adrs:
  - ADR-093
phase: 1
runner: cargo-test
runner-args: "tc_1035_state_and_decider_justification_are_advisory_findings"
last-run: 2026-06-25T11:32:01.135720180+00:00
last-run-duration: 20.0s
---

## Description

§3.3/§3.4 — a guard-less Decider that evolves a field it never reads draws a
State justification warning, and a Decider with no reachable rejection draws a
Decider justification warning. Both are advisory: `decider validate` prints them
and still exits 0 (conformant).