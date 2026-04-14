---
id: TC-380
title: single_responsibility_and
type: scenario
status: passing
validates:
  features:
  - FT-031
  adrs:
  - ADR-029
phase: 1
runner: cargo-test
runner-args: "tc_380_single_responsibility_and"
last-run: 2026-04-14T16:41:17.424364011+00:00
---

file with `//! Graph construction and traversal.` Assert exit 1 with the violating comment.