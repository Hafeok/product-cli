---
id: TC-377
title: module_structure_main_too_long
type: scenario
status: passing
validates:
  features:
  - FT-031
  adrs:
  - ADR-029
phase: 1
runner: cargo-test
runner-args: "tc_377_module_structure_main_too_long"
last-run: 2026-04-14T16:41:17.424364011+00:00
---

`main.rs` with 100 lines. Assert exit 1 with line count.