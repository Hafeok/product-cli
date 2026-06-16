---
id: TC-961
title: work-unit unfrozen context is a violation
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
- stderr
runner: cargo-test
runner-args: tc_961_work_unit_unfrozen_context_is_a_violation
---

## Scenario — a work unit's context must be frozen

**Given** a work unit whose context has `frozen: false`,
**When** the user runs `product work-unit validate`,
**Then** the process exits 1 and stderr reports that the context must be frozen.

## Validates

- FT-116 — product work-unit — validate an SPMC work unit against the What graph and How
- ADR-058 — Work units are validated as frozen SPMC manifests cross-checked against What and How
