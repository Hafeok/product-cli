---
id: TC-963
title: work-unit show and init
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
runner-args: tc_963_work_unit_show_and_init
---

## Scenario — scaffold, then inspect

**Given** a repo with no work unit,
**When** the user runs `product work-unit init my-unit`,
**Then** the process exits 0 and writes `.product/work-unit.yaml`; a second
`init` without `--force` exits 1; and `work-unit show` stdout reports the
id and `frozen=true`.

## Validates

- FT-116 — product work-unit — validate an SPMC work unit against the What graph and How
- ADR-058 — Work units are validated as frozen SPMC manifests cross-checked against What and How
