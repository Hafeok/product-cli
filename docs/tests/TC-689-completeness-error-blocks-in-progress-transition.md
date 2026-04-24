---
id: TC-689
title: completeness_error_blocks_in_progress_transition
type: scenario
status: unimplemented
validates:
  features:
  - FT-055
  adrs:
  - ADR-047
phase: 1
---

**Covers session test ST-348** — `completeness-error-blocks-in-progress-transition`.

Verifies that when `completeness-severity = "error"` and a feature is missing required sections, `product feature status FT-NNN in-progress` refuses the transition.

**Setup:**

- `product.toml` sets `[features].completeness-severity = "error"`.
- Feature FT-X is in `status: planned` with a body missing `### Boundaries`.

**Steps:**

1. Run `product feature status FT-X in-progress`.

**Assertions:**

- Exit code is `1`.
- Stderr contains `"W030"` and the missing subsection name (`Boundaries`).
- The feature file on disk is **unchanged** — the transition did not commit.
- `product feature show FT-X` still reports `status: planned`.

**Also exercises:** setting `completeness-severity = "warning"` and re-running the same command succeeds (the gate is tier-gated, not code-gated).
