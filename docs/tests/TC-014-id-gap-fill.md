---
id: TC-014
title: id_gap_fill
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
runner-args: "tc_014_id_gap_fill"
last-run: 2026-04-14T10:48:19.709127491+00:00
---

create features `FT-001` and `FT-003` manually. Run `product feature new`. Assert the new feature is assigned `FT-004` (gaps are not filled — next ID is always `max(existing) + 1`).