---
id: TC-568
title: gap_check_g003_no_rejected_alternatives
type: scenario
status: passing
validates:
  features:
  - FT-045
  adrs:
  - ADR-019
  - ADR-040
phase: 1
runner: cargo-test
runner-args: tc_568_gap_check_g003_no_rejected_alternatives
last-run: 2026-04-18T14:07:23.114790413+00:00
last-run-duration: 0.2s
---

## Session: ST-125 — gap-check-g003-no-rejected-alternatives

**Validates:** FT-045, ADR-019 (amended), ADR-040 (G003 structural detection)

### Given

A temp repository with an accepted ADR whose markdown body has **no** `**Rejected alternatives:**` section (or an empty one).

### When

`product gap check` is run.

### Then

- Exit code is `1`.
- A finding with code `G003` is reported, naming the ADR.
- If the ADR is edited to include a non-empty rejected-alternatives section (and re-sealed via amend), the next `product gap check` no longer reports G003 for that ADR.
- No LLM call was made.