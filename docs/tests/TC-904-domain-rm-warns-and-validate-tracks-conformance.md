---
id: TC-904
title: domain rm warns and validate tracks conformance
type: scenario
status: passing
validates:
  features:
  - FT-110
  adrs:
  - ADR-053
phase: 6
observes:
- exit-code
- stderr
runner: cargo-test
runner-args: tc_904_domain_rm_and_validate_exit_codes
---

## Scenario — delete warns on dangling refs; validate exit code tracks conformance

**Given** a conformant graph where event `OrderPlaced` changes entity
`Order`,
**When** the user runs `product domain validate` (exit 0), then
`product domain rm Order`,
**Then** `rm` exits 0 and prints a dangling-reference warning on stderr, and a
subsequent `product domain validate` exits 1 (the orphaned event is now
non-conformant).

## Validates

- FT-110 — product domain — CLI list, show, and CRUD over the captured What graph
- ADR-053 — Domain authoring is a separate What graph with native in-loop conformance
