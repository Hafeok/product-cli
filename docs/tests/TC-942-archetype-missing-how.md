---
id: TC-942
title: archetype missing how is nonconformant
type: scenario
status: passing
validates:
  features:
  - FT-114
  adrs:
  - ADR-056
phase: 6
observes:
- exit-code
- stderr
runner: cargo-test
runner-args: tc_942_archetype_missing_how_is_nonconformant
---

## Scenario — an archetype must declare a How contract

**Given** an archetype directory with no `how-contract.yaml`,
**When** the user runs `product archetype validate`,
**Then** the process exits 1 and stderr reports that an archetype must declare a
How contract.

## Validates

- FT-114 — product archetype — assemble and validate How, layout, and cells as one
- ADR-056 — An archetype assembles How, layout, and cells from a directory and validates the whole
