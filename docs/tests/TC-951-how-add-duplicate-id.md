---
id: TC-951
title: how add duplicate id is rejected
type: scenario
status: passing
validates:
  features:
  - FT-115
  adrs:
  - ADR-057
phase: 6
observes:
- exit-code
- stderr
runner: cargo-test
runner-args: tc_951_how_add_duplicate_id_is_rejected
---

## Scenario — the Why cascade shares one id namespace

**Given** a How with a principle `x`,
**When** the user runs `product how add pattern x …`,
**Then** the process exits 1 and stderr reports that the id already exists.

## Validates

- FT-115 — product how add/set — granular authoring of the Why cascade and contracts
- ADR-057 — How elements are authored granularly via add/set on the contract file
