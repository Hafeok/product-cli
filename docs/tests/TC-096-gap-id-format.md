---
id: TC-096
title: gap_id_format
type: scenario
status: passing
validates:
  features:
  - FT-029
  adrs:
  - ADR-019
phase: 1
runner: cargo-test
runner-args: "tc_096_gap_id_format"
last-run: 2026-04-14T17:25:14.338071018+00:00
---

all gap IDs must match `GAP-[A-Z]+-[A-Z0-9]+-[A-Z0-9]{4,8}` pattern.