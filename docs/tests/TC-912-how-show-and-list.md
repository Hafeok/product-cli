---
id: TC-912
title: how show and list render the contract
type: scenario
status: passing
validates:
  features:
  - FT-111
  adrs:
  - ADR-054
phase: 6
observes:
- exit-code
- stdout
runner: cargo-test
runner-args: tc_912_how_show_and_list
---

## Scenario — show and list

**Given** the bundled example How contract,
**When** the user runs `product how show` and `product how list patterns`,
**Then** both exit 0; `show` stdout includes the archetype and the principle
count, and `list patterns` stdout includes the pattern ids.

## Validates

- FT-111 — product how — validate, show, and project an archetype's How contract
- ADR-054 — How contracts are file-based YAML with a native checker mirroring how.shacl.ttl
