---
id: TC-376
title: module_structure_missing
type: scenario
status: passing
validates:
  features:
  - FT-031
  adrs:
  - ADR-029
phase: 1
runner: cargo-test
runner-args: "tc_376_module_structure_missing"
last-run: 2026-04-14T16:41:17.424364011+00:00
---

delete `src/graph/`. Assert exit 1 naming `src/graph/`.