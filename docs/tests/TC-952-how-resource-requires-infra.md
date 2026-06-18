---
id: TC-952
title: how add resource requires infra contract
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
runner-args: tc_952_how_add_resource_requires_infra_contract
---

## Scenario — a resource needs its infrastructure contract first

**Given** a How with no infrastructure contract,
**When** the user runs `product how add resource db …`,
**Then** the process exits 1 and stderr tells the user to set the
infrastructure contract first.

## Validates

- FT-115 — product how add/set — granular authoring of the Why cascade and contracts
- ADR-057 — How elements are authored granularly via add/set on the contract file
