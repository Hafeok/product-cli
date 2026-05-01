---
id: TC-709
title: preflight_fails_when_tc_missing_runner_for_active_feature
type: scenario
status: passing
validates:
  features:
  - FT-058
  adrs:
  - ADR-021
phase: 1
runner: cargo-test
runner-args: "tc_709_preflight_fails_when_tc_missing_runner_for_active_feature"
last-run: 2026-04-30T09:25:29.076293260+00:00
last-run-duration: 0.5s
---

**Test Type:** scenario

**Why this TC exists:**

The harness gate. `product preflight FT-XXX` is the command a
harness calls just before invoking an agent — it is the right
place to catch missing runner config so the agent does not waste
a turn implementing against an unverifiable spec. This TC pins
that preflight fails with E022 when the feature is in active
development and any linked TC lacks runner config.

**Setup:**

1. Build a tempdir fixture repo with:
   - Feature `FT-001`, status `in-progress`, linked to `TC-001`
     (well-formed) and `TC-002` (missing runner config).

**Execution:**

1. Run `product preflight FT-001`.

**Expected:**

- Exit code `22`.
- Stderr contains `error[E022]` naming `TC-002`.
- The hint section names the four-gate model so the developer
  understands why this was caught here as well as elsewhere:
  `= hint: runner config is required for in-progress features —
   add it to every linked TC and re-run preflight`.
- The pre-existing preflight checks (domain coverage W010,
  acknowledgement W011) still run and any findings are reported
  alongside E022 in a single output.

**Negative case:**

1. With the same fixture, mutate `FT-001` status to `planned`.
2. Re-run `product preflight FT-001`.

**Expected (negative case):**

- Exit code `0` (or whatever the existing preflight exit code
  is for the no-other-findings case).
- No `error[E022]` text in stderr.

**Notes:**

- Together with TC-708, this gate catches the issue *before* the
  agent runs. TC-705 (verify) and TC-707 (graph check) are
  defense-in-depth.
- Preflight stays advisory in spirit for non-runner findings,
  but E022 is fatal regardless because verify will fail anyway —
  catching it here saves the agent turn.