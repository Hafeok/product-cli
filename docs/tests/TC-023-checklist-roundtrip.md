---
id: TC-023
title: checklist_roundtrip
type: scenario
status: passing
validates:
  features:
  - FT-017
  - FT-014
  adrs:
  - ADR-007
phase: 1
runner: cargo-test
runner-args: "tc_023_checklist_roundtrip"
last-run: 2026-04-14T14:18:28.985359737+00:00
---

generate checklist, change a feature status, regenerate. Assert the checklist reflects the updated status with no residue from the previous generation.