---
id: TC-931
title: cell validate flags slot without inline audit
type: scenario
status: passing
validates:
  features:
  - FT-113
  adrs:
  - ADR-055
phase: 6
observes:
- exit-code
- stderr
runner: cargo-test
runner-args: tc_931_cell_validate_flags_slot_without_inline_audit
---

## Scenario — a slot with no backing audit is rejected

**Given** a task type where a slot's inline `audit` field is blank,
**When** the user runs `product cell validate`,
**Then** the process exits 1 and stderr names the §5/§6.1 rule 'no slot without
a backing audit'.

## Validates

- FT-113 — product cell — validate task-types against the What graph and How contract
- ADR-055 — Task-types (cells) are cross-validated against the captured What graph and How contract
