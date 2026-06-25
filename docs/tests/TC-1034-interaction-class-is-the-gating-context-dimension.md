---
id: TC-1034
title: interaction class is the gating context dimension
type: invariant
status: passing
validates:
  features:
  - FT-150
  adrs:
  - ADR-092
phase: 1
runner: cargo-test
runner-args: "tc_1034_interaction_class_is_the_gating_context_dimension"
last-run: 2026-06-25T10:39:02.783524668+00:00
last-run-duration: 13.2s
---

## Description

§3.2.2 — a system may target the recognised closed-core interaction classes
(gui, tui); an unrecognised class (e.g. "holographic") is a finding. The targeted
class is emitted as `pf:targetsClass` in the Turtle export.