---
id: TC-023
title: checklist_roundtrip
type: scenario
status: passing
validates:
  features:
  - FT-017
  adrs:
  - ADR-007
phase: 1
runner: cargo-test
runner-args: "tc_023_checklist_roundtrip"
---

generate checklist, change a feature status, regenerate. Assert the checklist reflects the updated status with no residue from the previous generation.