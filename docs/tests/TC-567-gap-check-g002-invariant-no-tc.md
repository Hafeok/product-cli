---
id: TC-567
title: gap_check_g002_invariant_no_tc
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
runner-args: tc_567_gap_check_g002_invariant_no_tc
last-run: 2026-04-18T14:07:23.114790413+00:00
last-run-duration: 0.2s
---

## Session: ST-124 — gap-check-g002-invariant-no-tc

**Validates:** FT-045, ADR-019 (amended), ADR-040 (G002 structural detection)

### Given

A temp repository with an ADR whose body contains an Invariants formal block (the `Gamma:Invariants` block per ADR-016), but no TC of type `scenario` or `chaos` is linked to that ADR (via `validates.adrs`).

### When

`product gap check` is run.

### Then

- Exit code is `1` (unsuppressed new finding).
- stdout / stderr contains a finding with code `G002` naming the ADR.
- If the finding is added to `gaps.json` via `product gap suppress`, a subsequent `product gap check` exits `0`.
- No LLM call was made.