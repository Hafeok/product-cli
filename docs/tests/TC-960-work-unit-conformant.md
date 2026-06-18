---
id: TC-960
title: work-unit validate conformant example
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
runner-args: tc_960_work_unit_validate_conformant_example
---

## Scenario — validate a conformant SPMC work unit

**Given** the bundled example work unit at `.product/work-unit.yaml`,
**When** the user runs `product work-unit validate`,
**Then** the process exits 0 and stdout reports `conformant` naming the
produced artifact.

## Validates

- FT-116 — product work-unit — validate an SPMC work unit against the What graph and How
- ADR-058 — Work units are validated as frozen SPMC manifests cross-checked against What and How
