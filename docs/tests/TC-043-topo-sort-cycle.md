---
id: TC-043
title: topo_sort_cycle
type: scenario
status: passing
validates:
  features:
  - FT-006
  - FT-011
  - FT-016
  - FT-024
  - FT-014
  adrs:
  - ADR-012
phase: 1
runner: cargo-test
runner-args: "tc_043_topo_sort_cycle"
last-run: 2026-04-14T15:03:33.506444091+00:00
---

FT-001 depends-on FT-002, FT-002 depends-on FT-001. Assert `product graph check` exits with code 1 and names both features in the error message.