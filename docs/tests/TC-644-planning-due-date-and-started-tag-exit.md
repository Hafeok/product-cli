---
id: TC-644
title: planning_due_date_and_started_tag_exit
type: exit-criteria
status: passing
validates:
  features:
  - FT-053
  adrs:
  - ADR-045
phase: 5
runner: cargo-test
runner-args: tc_644_planning_due_date_and_started_tag_exit
last-run: 2026-04-21T13:09:33.430942991+00:00
last-run-duration: 0.5s
---

## Exit Criteria — FT-053 Planning Due Dates and Started Tags

FT-053 is complete when all of the following hold:

1. The feature schema accepts an optional `due-date: YYYY-MM-DD`
   field; invalid shapes produce E006 with an `expected
   YYYY-MM-DD` hint.
2. W028 fires when `due-date < today AND status != complete`
   and W029 fires within the configured warning window AND
   `status != complete`. Both are W-class (exit 2).
3. `[planning].due-date-warning-days` defaults to 3, can be set
   to any non-negative integer, and setting 0 disables W029.
4. `product/FT-XXX/started` is created exactly once per feature
   on the first `planned → in-progress` transition (or at apply
   time for features created with `status: in-progress`). It is
   never overwritten on replan.
5. Tag creation is best-effort: when git is unavailable, apply
   still succeeds and emits a W-class warning.
6. `product status` renders a due-date cell for features with
   the field set, flags overdue features, and omits the cell
   for features without the field.
7. `product tags list --type started` returns started tags;
   `--feature FT-XXX` includes both started and complete tags.
8. `due-date` never causes `product verify` to exit 1 and has
   no effect on phase gate evaluation, TC execution, or feature
   completion.
9. `cargo test`, `cargo clippy -- -D warnings -D
   clippy::unwrap_used`, and `cargo build` all pass.
10. Every TC under FT-053 has `runner: cargo-test` and
    `runner-args` set to the integration test function name.