---
id: TC-710
title: error_lists_all_tcs_missing_runner_in_one_report
type: scenario
status: passing
validates:
  features:
  - FT-058
  adrs:
  - ADR-013
phase: 1
runner: cargo-test
runner-args: "tc_710_error_lists_all_tcs_missing_runner_in_one_report"
last-run: 2026-04-30T09:25:29.076293260+00:00
last-run-duration: 0.3s
---

**Test Type:** scenario

**Why this TC exists:**

ADR-013's "report all findings, not just the first" convention is
load-bearing for usability. If E022 reported one TC at a time, a
feature with five missing-runner TCs would force five fix cycles.
This TC pins the multi-offender contract: every TC linked to the
target feature lacking runner config appears in a single error
output.

**Setup:**

1. Build a tempdir fixture repo with:
   - Feature `FT-001`, status `in-progress`, linked to four TCs:
     - `TC-001` well-formed with runner config.
     - `TC-002` missing both `runner` and `runner-args`.
     - `TC-003` has `runner` but missing `runner-args`.
     - `TC-004` has `runner-args` but missing `runner`.

**Execution:**

1. Run `product verify FT-001 --format json`.

**Expected (JSON):**

- Exit code `22`.
- Stdout JSON has shape:
  ```json
  {
    "error": "E022",
    "feature_id": "FT-001",
    "tc_ids": ["TC-002", "TC-003", "TC-004"]
  }
  ```
- `tc_ids` is sorted (deterministic ordering for diff stability).
- `TC-001` is **not** in the array.

**Execution (text):**

1. Run `product verify FT-001` (default text format).

**Expected (text):**

- Stderr contains `error[E022]: TC runner configuration missing`.
- Stderr lists three `--> docs/tests/...` lines, one per
  offender, in the same sorted order.
- The summary line reads `= 3 TC(s) linked to FT-001 lack
  \`runner\` and/or \`runner-args\``.
- The fix-snippet hint appears once (not per-TC).

**Notes:**

- The two failure modes "missing runner" and "missing
  runner-args" are both reported under E022 with the same
  treatment. They are not separate error codes — both indicate
  the same root cause (incomplete runner declaration).
- The deterministic-ordering property is asserted because it
  matters for snapshot tests and for stable git diffs in CI
  output captures.