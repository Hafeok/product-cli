---
id: TC-379
title: single_responsibility_missing
type: scenario
status: passing
validates:
  features:
  - FT-031
  adrs:
  - ADR-029
phase: 1
runner: cargo-test
runner-args: "tc_379_single_responsibility_missing"
last-run: 2026-04-14T16:41:17.424364011+00:00
---

file with no `//!` first line. Assert exit 1 naming the file.