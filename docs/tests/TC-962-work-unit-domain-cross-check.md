---
id: TC-962
title: work-unit domain pointer cross-checks the graph
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
- stderr
runner: cargo-test
runner-args: tc_962_work_unit_domain_pointer_cross_checks_the_graph
---

## Scenario — a frozen-input domain pointer is checked against the What graph

**Given** a work unit `derived_from` `domain:Task`, and a captured What graph
without a Task entity,
**When** the user runs `product work-unit validate`,
**Then** the process exits 0 (a dangling domain ref is a warning), stdout
reports `domain: cross-checked`, and stderr warns about `domain:Task`.

## Validates

- FT-116 — product work-unit — validate an SPMC work unit against the What graph and How
- ADR-058 — Work units are validated as frozen SPMC manifests cross-checked against What and How
