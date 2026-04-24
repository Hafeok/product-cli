---
id: TC-687
title: completeness_severity_warning_w030_is_w_class
type: scenario
status: unimplemented
validates:
  features:
  - FT-055
  adrs:
  - ADR-047
phase: 1
---

**Covers session test ST-346** — `completeness-severity-warning-w030-is-w-class`.

Verifies that the default `[features].completeness-severity = "warning"` surfaces W030 in the `warnings` array with exit code 2, not in `errors` with exit code 1.

**Setup:**

- Feature with a missing required section.
- `product.toml` does not override `completeness-severity` (default `"warning"`).

**Steps:**

1. Run `product graph check --format json`.

**Assertions:**

- `errors[]` contains no entries with `code == "W030"`.
- `warnings[]` contains a W030 entry.
- Exit code is `2` (warnings, no errors).
- The entry's `tier` field (when exposed in JSON) is `"warning"`.
