---
id: TC-711
title: requires_failure_remains_unrunnable_not_hard_fail
type: scenario
status: passing
validates:
  features:
  - FT-058
  adrs:
  - ADR-021
phase: 1
runner: cargo-test
runner-args: "tc_711_requires_failure_remains_unrunnable_not_hard_fail"
last-run: 2026-04-30T09:25:29.076293260+00:00
last-run-duration: 0.4s
---

**Test Type:** scenario

**Why this TC exists:**

ADR-021's `requires` field is the orthogonal "unrunnable" branch.
A TC may have a perfectly valid `runner` configured but fail a
declarative prerequisite (e.g. `binary-compiled`,
`two-node-cluster`). That case is environmental, not
configuration: the user cannot fix the YAML to make it pass; they
need to satisfy the prerequisite. FT-058 must not collapse this
case into E022 — it must remain a soft `unrunnable` status update
exactly as before.

**Setup:**

1. Build a tempdir fixture repo with:
   - `[verify.prerequisites]` in `product.toml` declaring
     `nonexistent = "false"` (a command that always exits 1).
   - Feature `FT-001`, status `in-progress`, linked to `TC-001`.
   - `TC-001` has `runner: cargo-test`, `runner-args:
     "tc_001_..."`, AND `requires: [nonexistent]`.

**Execution:**

1. Run `product verify FT-001`.

**Expected:**

- Exit code `0` (or whatever the pre-FT-058 exit code is for
  "all TCs unrunnable" — it does **not** become 22).
- Stdout contains `TC-001  <title>  UNRUNNABLE (prerequisite
  'nonexistent' not satisfied)`.
- No `error[E022]` text in stderr.
- `TC-001` front-matter is updated: `status: unrunnable`,
  `failure-message: "prerequisite 'nonexistent' not satisfied"`.
- `FT-001` status is **not** auto-promoted to `complete`
  (zero TCs passed; the existing W016 warning is emitted).

**Companion check:**

1. Configure a TC `TC-002` with `runner: cargo-test`,
   `runner-args: "tc_002_..."`, and `requires:
   [undefined-prerequisite-name]` where
   `undefined-prerequisite-name` is **not** declared in
   `[verify.prerequisites]`.
2. Run `product verify FT-001` (with `TC-002` linked instead of
   `TC-001`).

**Expected (companion):**

- Exit code matches the pre-FT-058 E011 path
  (`prerequisite 'undefined-prerequisite-name' is not defined in
  [verify.prerequisites]`).
- It is **not** an E022 — the runner is configured; only the
  prerequisite definition is missing.

**Notes:**

- This TC is a regression guard. Without it, an over-eager
  refactor of `run_tc_list` could fold both unrunnable branches
  under E022 and break the wrapper-script escape hatch.
- The `requires` semantics are an ADR-021 invariant. FT-058
  amends ADR-021 only on the no-runner clause; this TC pins the
  remaining `requires` clause.