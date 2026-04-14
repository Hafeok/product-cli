---
id: TC-042
title: topo_sort_parallel
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
runner-args: "tc_042_topo_sort_parallel"
last-run: 2026-04-14T14:53:21.175394484+00:00
---

FT-002 and FT-003 both depend-on FT-001, no dependency between FT-002 and FT-003. Assert FT-001 appears before both; FT-002 and FT-003 order is unspecified.