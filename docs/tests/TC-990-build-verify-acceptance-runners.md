---
id: TC-990
title: build verify runs acceptance runners
type: scenario
status: passing
validates:
  features:
  - FT-130
  adrs:
  - ADR-071
phase: 6
observes:
- stdout
- file-state
runner: cargo-test
runner-args: tc_990_build_verify_runs_acceptance_runners
---

## Scenario — build runs the §6 verify step over acceptance runners

**Given** a deliverable whose acceptance criteria carry runners (a `shell` runner
that passes, `a1`, and one that fails, `a2`),
**When** the user runs `product build place-order --role coder` (worker offline),
**Then** stdout shows a `Verify (§6)` section with `[x] a1` and `[ ] a2`, the
verdicts are recorded back into the deliverable YAML (`status: passing` /
`status: failing`), and the gate reports `not done` because `a2` failed.

## Validates

- FT-130 — product build — the SPMC build orchestrator records verification into done
- ADR-071 — verification is recorded, not judged; done is a predicate over recorded checks
