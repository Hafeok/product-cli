---
id: TC-707
title: graph_check_flags_tc_missing_runner_when_feature_in_progress
type: scenario
status: passing
validates:
  features:
  - FT-058
  adrs:
  - ADR-021
phase: 1
runner: cargo-test
runner-args: "tc_707_graph_check_flags_tc_missing_runner_when_feature_in_progress"
last-run: 2026-04-30T09:25:29.076293260+00:00
last-run-duration: 0.3s
---

**Test Type:** scenario

**Why this TC exists:**

The CI gate. `product graph check` is the command CI calls on
every push; it is the place a manual-edit drift (someone deleted
a `runner:` line in a YAML file) gets caught even if no developer
runs `verify` locally. This TC pins that `graph check` reports
E022 (fatal, not a W-class warning) when an in-progress feature
has any TC missing runner config.

**Setup:**

1. Build a tempdir fixture repo with:
   - Feature `FT-001`, status `in-progress`, linked to `TC-001`
     and `TC-002`.
   - `TC-001` well-formed with runner config.
   - `TC-002` missing both `runner` and `runner-args`.
   - Feature `FT-002`, status `complete`, linked to `TC-003`,
     where `TC-003` is also missing runner config.
   - Feature `FT-003`, status `planned`, linked to `TC-004`,
     where `TC-004` is also missing runner config.

**Execution:**

1. Run `product graph check`.

**Expected:**

- Exit code `1` (E-class fatal — same exit code class as other
  graph errors).
- Stderr contains `error[E022]` at least twice (once per
  offending feature/TC pair: `FT-001`/`TC-002` and
  `FT-002`/`TC-003`).
- Stderr does **not** flag `FT-003`/`TC-004` — that pairing is
  exempt because the feature is `planned`.
- Each E022 finding names the feature ID, the TC ID, and the
  TC's file path.
- The fix-snippet hint appears at least once.
- A `--format json` invocation returns a top-level `errors`
  array containing exactly two objects with code `E022`, each
  with `feature_id` and `tc_ids` fields.

**Notes:**

- This is the gate the user originally raised the issue about:
  "the test cases with no runners configured should be a hard
  fail". `graph check` is the lowest-friction place a CI run
  catches it.
- The complete-feature case (`FT-002`/`TC-003`) is included so
  the gate also surfaces drift on already-shipped features. That
  is intentional: if a TC's runner config is silently deleted
  after the feature is marked complete, the audit trail catches
  it on the next CI run rather than the next attempt to verify
  (which might be months away).