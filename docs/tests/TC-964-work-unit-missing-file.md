---
id: TC-964
title: work-unit validate without file is a clear error
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
runner-args: tc_964_work_unit_validate_without_file_is_a_clear_error
---

## Scenario — a missing work-unit file is a clear error

**Given** a repo with no work-unit file,
**When** the user runs `product work-unit validate`,
**Then** the process exits 1 and stderr explains there is no work unit,
pointing at `product work-unit init`.

## Validates

- FT-116 — product work-unit — validate an SPMC work unit against the What graph and How
- ADR-058 — Work units are validated as frozen SPMC manifests cross-checked against What and How
