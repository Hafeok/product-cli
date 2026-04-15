---
id: TC-475
title: graph check emits W019 for out-of-scope feature
type: scenario
status: passing
validates:
  features:
  - FT-039
  adrs:
  - ADR-013
phase: 1
runner: cargo-test
runner-args: "tc_475_graph_check_emits_w019_for_out_of_scope_feature"
last-run: 2026-04-15T11:22:02.279019545+00:00
last-run-duration: 0.4s
---

**Given** a repository with `[product].responsibility = "A private cloud platform for Raspberry Pi"` AND a feature FT-099 titled "Grocery List Management" exists
**When** `product graph check` runs validation
**Then** stderr contains `warning[W019]: feature outside product responsibility` referencing FT-099

**Given** a repository with `[product].responsibility` set AND all features are clearly within scope
**When** `product graph check` runs
**Then** no W019 warnings are emitted