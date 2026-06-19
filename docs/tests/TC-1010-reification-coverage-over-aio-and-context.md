---
id: TC-1010
title: reification coverage over AIO and context
type: scenario
status: unimplemented
validates:
  features:
  - FT-139
  adrs:
  - ADR-083
phase: 7
observes:
- graph
- exit-code
runner: cargo-test
runner-args: tc_1010_reification_coverage_over_aio_and_context
---

## Scenario — every (AIO, context) a screen can encounter must have a reifying rule

**Given** a product whose UI steps reference the `single-select` and
`trigger-action` AIOs across the contexts {phone, tablet},
**And** reification rules covering every (AIO, context) pair those steps can
encounter,
**When** the user runs the reification-coverage check,
**Then** the process exits 0 — coverage is complete.

**And given** the rule for (`single-select`, phone) is removed, **when** the
coverage check runs again, **then** the process exits non-zero and the check
names the uncovered (AIO, context) pair — a screen left unspecified for some
device, the design-system analogue of a Decider's missing command coverage.

## Validates

- FT-139 — Design system and reification rules
- ADR-083 — Screens bind to a design system; AIOs reify to CIOs by context of use
