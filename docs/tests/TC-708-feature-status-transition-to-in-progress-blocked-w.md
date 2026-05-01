---
id: TC-708
title: feature_status_transition_to_in_progress_blocked_without_runner
type: scenario
status: passing
validates:
  features:
  - FT-058
  adrs:
  - ADR-021
  - ADR-038
phase: 1
runner: cargo-test
runner-args: "tc_708_feature_status_transition_to_in_progress_blocked_without_runner"
last-run: 2026-04-30T09:25:29.076293260+00:00
last-run-duration: 0.3s
---

**Test Type:** scenario

**Why this TC exists:**

The status-transition gate. The cleanest UX for catching missing
runner config is to refuse the `planned → in-progress`
transition. Once the gate fires here, the developer fixes the
TCs first and only then promotes the feature, so all four
downstream gates remain trivially satisfied. This TC exercises
the gate via both `product feature status` and `product request
apply`, since both routes write the same field.

**Setup:**

1. Build a tempdir fixture repo with:
   - Feature `FT-001`, status `planned`, linked to `TC-001` and
     `TC-002`.
   - `TC-001` well-formed with runner config.
   - `TC-002` missing both `runner` and `runner-args`.

**Execution (CLI route):**

1. Run `product feature status FT-001 in-progress`.

**Expected (CLI route):**

- Exit code `22`.
- Stderr contains `error[E022]: TC runner configuration missing`
  with `TC-002` named.
- `FT-001` status remains `planned` (the transition was rejected
  before any write).

**Execution (request route):**

1. With `FT-001` still at `planned` and `TC-002` still missing
   runner config, run `product request apply` with the YAML:
   ```yaml
   type: change
   reason: "Promote FT-001 to in-progress"
   changes:
     - target: FT-001
       mutations:
         - op: set
           field: status
           value: in-progress
   ```

**Expected (request route):**

- Exit code `22`.
- Stderr / JSON output contains an E022 finding naming `TC-002`.
- `FT-001` status remains `planned`.
- No entry appended to the request log (the apply was rejected
  before any I/O — request atomicity is preserved).

**Recovery path:**

1. Add `runner: cargo-test` and `runner-args: "tc_002_..."` to
   `TC-002`.
2. Re-run either of the above commands.

**Expected (recovery):**

- Exit code `0`.
- `FT-001` status is now `in-progress`.

**Notes:**

- This is the most user-friendly gate of the four. Catching the
  problem here means the harness's preflight call (TC-709) and
  the verify call (TC-705) never have to fail — the developer
  has already been forced to fix it.
- Asserts the same error code from both CLI and request routes
  to pin that the predicate lives in one place
  (`tc::runner_required::find_offenders`) and is not duplicated.