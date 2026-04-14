---
id: TC-013
title: id_auto_increment
type: scenario
status: passing
validates:
  features:
  - FT-001
  - FT-004
  - FT-009
  adrs:
  - ADR-005
phase: 1
runner: cargo-test
runner-args: "tc_013_id_auto_increment"
last-run: 2026-04-14T10:48:19.709127491+00:00
---

create three features in sequence. Assert their IDs are `FT-001`, `FT-002`, `FT-003`.