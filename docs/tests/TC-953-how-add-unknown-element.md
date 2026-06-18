---
id: TC-953
title: how add unknown element is rejected
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
runner-args: tc_953_how_add_unknown_element_is_rejected
---

## Scenario — an unknown element kind is rejected

**Given** any repo,
**When** the user runs `product how add widget x`,
**Then** the process exits 1 and stderr reports an unknown element with the
accepted kinds.

## Validates

- FT-115 — product how add/set — granular authoring of the Why cascade and contracts
- ADR-057 — How elements are authored granularly via add/set on the contract file
