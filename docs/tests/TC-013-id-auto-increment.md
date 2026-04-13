---
id: TC-013
title: id_auto_increment
type: scenario
status: passing
validates:
  features:
  - FT-001
  - FT-009
  adrs:
  - ADR-005
phase: 1
runner: cargo-test
runner-args: "tc_013_id_auto_increment"
---

create three features in sequence. Assert their IDs are `FT-001`, `FT-002`, `FT-003`.