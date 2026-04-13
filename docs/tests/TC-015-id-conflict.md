---
id: TC-015
title: id_conflict
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
runner-args: "tc_015_id_conflict"
---

attempt to create a feature with an ID that already exists. Assert the CLI returns an error and does not overwrite the existing file.