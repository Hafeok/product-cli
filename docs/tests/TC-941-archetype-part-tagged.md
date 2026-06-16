---
id: TC-941
title: archetype validate reports part-tagged violations
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
runner-args: tc_941_archetype_validate_reports_part_tagged_violations
---

## Scenario — a part violation is attributed to its part

**Given** an archetype whose layout has a rule with no `enforces` (a Guard 1
violation),
**When** the user runs `product archetype validate`,
**Then** the process exits 1 and stderr reports the violation tagged with its
part (`layout/…`) and names Guard 1.

## Validates

- FT-114 — product archetype — assemble and validate How, layout, and cells as one
- ADR-056 — An archetype assembles How, layout, and cells from a directory and validates the whole
