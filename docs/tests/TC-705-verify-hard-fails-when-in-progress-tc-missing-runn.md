---
id: TC-705
title: verify_hard_fails_when_in_progress_tc_missing_runner
type: scenario
status: passing
validates:
  features:
  - FT-058
  adrs:
  - ADR-021
phase: 1
runner: cargo-test
runner-args: "tc_705_verify_hard_fails_when_in_progress_tc_missing_runner"
last-run: 2026-04-30T09:25:29.076293260+00:00
last-run-duration: 0.3s
---

**Test Type:** scenario

**Why this TC exists:**

The verify-time gate. Before FT-058 the verify pipeline silently
treated a TC with no `runner` configured as `UNIMPLEMENTED` and
continued, which let a feature stay perpetually `in-progress`
without surfacing the misconfiguration. This TC pins the new
behaviour: when the feature is in active development, missing
runner config is a hard E022 error that aborts the verify run
before any cargo invocation.

**Setup:**

1. Build a tempdir fixture repo with a single feature
   `FT-001` whose status is `in-progress`.
2. Link two TCs to `FT-001`:
   - `TC-001` with `runner: cargo-test`, `runner-args:
     "tc_001_..."` (well-formed).
   - `TC-002` with NO `runner` and NO `runner-args` lines in
     its front-matter.

**Execution:**

1. Run `product verify FT-001`.

**Expected:**

- Exit code `22` (the new `E022` exit).
- Stderr contains `error[E022]: TC runner configuration missing`.
- Stderr names `TC-002` (the offender) and shows its file path.
- Stderr does **not** name `TC-001`.
- Stderr includes the canonical fix snippet:
  ```
  runner: cargo-test
  runner-args: "tc_002_..."
  ```
- No cargo subprocess is spawned. The check fires before the
  TC list runs (`run_tc_list` returns early on the precondition).
- Feature `FT-001` status remains `in-progress` — the verify run
  performed no writes.
- `TC-001` and `TC-002` front-matter is unchanged
  (no `last-run`, `last-run-duration`, or `failure-message`
  injection).
- No entry is appended to the request log for this aborted run.

**Notes:**

- Pairs with TC-706 (the planned-status escape hatch) and TC-711
  (the `requires` soft-skip preservation). Together they pin the
  full decision matrix for what "unrunnable" means after FT-058.
- The check executes *before* the lifecycle gate (E016 — proposed
  ADR), but order does not matter because both are blocking
  errors; the test asserts E022 is what fires when both
  preconditions could fire.