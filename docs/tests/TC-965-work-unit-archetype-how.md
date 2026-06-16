---
id: TC-965
title: work-unit validate discovers the archetype's How contract
type: scenario
status: passing
validates:
  features:
  - FT-116
  adrs:
  - ADR-058
phase: 6
observes:
- exit-code
- stdout
runner: cargo-test
runner-args: tc_965_work_unit_validate_discovers_archetype_how
---

## Scenario — a dispatched unit cross-checks against its archetype's How

**Given** a work unit under `.product/archetypes/demo/work-units/` that applies
a pattern defined in that archetype's `how-contract.yaml`,
**When** the user runs `product work-unit validate --file <that unit>`,
**Then** the process exits 0, stdout reports `how: cross-checked` (the
archetype's How was discovered, not only the top-level one), and the applied
pattern raises no "not a pattern" warning.

## Validates

- FT-116 — product work-unit — validate an SPMC work unit against the What graph and How
- ADR-058 — Work units are validated as frozen SPMC manifests cross-checked against What and How
