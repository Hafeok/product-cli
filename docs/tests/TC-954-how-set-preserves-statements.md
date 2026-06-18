---
id: TC-954
title: how set preserves added statements
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
- stdout
runner: cargo-test
runner-args: tc_954_how_set_preserves_added_statements
---

## Scenario — re-setting a contract preserves its statements

**Given** an application contract with one added statement,
**When** the user re-runs `product how set app-contract` to change its
metadata,
**Then** the process exits 0 and `how show` stdout still reports
`statements: 1` — the statement survives the re-set.

## Validates

- FT-115 — product how add/set — granular authoring of the Why cascade and contracts
- ADR-057 — How elements are authored granularly via add/set on the contract file
