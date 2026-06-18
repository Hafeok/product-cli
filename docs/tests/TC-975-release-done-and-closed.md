---
id: TC-975
title: release done requires members done and closed cut
type: scenario
status: passing
validates:
  features:
  - FT-127
  adrs:
  - ADR-068
phase: 6
observes:
- exit-code
- stdout
runner: cargo-test
runner-args: tc_975_release_done_requires_members_done_and_closed
---

## Scenario — release_done = all members done AND the cut is closed

**Given** a release whose one deliverable is done (acceptance recorded passing,
in-scope domain conformant) and whose slice scope is dependency-complete,
**When** the user runs `product release done R1`,
**Then** the process exits 0 and reports `DONE` with `cut closed`.

## Validates

- FT-127 — product deliverable/release done — the §7.2 computed delivery predicates
- ADR-068 — Done is computed from existing verifications plus recorded acceptance
