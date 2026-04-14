---
id: TC-375
title: module_structure_passes
type: scenario
status: passing
validates:
  features:
  - FT-031
  adrs:
  - ADR-029
phase: 1
runner: cargo-test
runner-args: "tc_375_module_structure_passes"
last-run: 2026-04-14T16:41:17.424364011+00:00
---

all required modules present, `main.rs` under 80 lines. Assert exit 0.