---
id: TC-014
title: id_gap_fill
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
runner-args: "tc_014_id_gap_fill"
---

create features `FT-001` and `FT-003` manually. Run `product feature new`. Assert the new feature is assigned `FT-004` (gaps are not filled — next ID is always `max(existing) + 1`).