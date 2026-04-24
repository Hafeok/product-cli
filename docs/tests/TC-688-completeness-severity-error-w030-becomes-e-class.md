---
id: TC-688
title: completeness_severity_error_w030_becomes_e_class
type: scenario
status: unimplemented
validates:
  features:
  - FT-055
  adrs:
  - ADR-047
phase: 1
---

**Covers session test ST-347** — `completeness-severity-error-w030-becomes-e-class`.

Verifies that setting `[features].completeness-severity = "error"` promotes W030 from warning tier to error tier. The code string `"W030"` remains stable.

**Setup:**

- `product.toml` contains `[features]` with `completeness-severity = "error"`.
- Feature with a missing required section.

**Steps:**

1. Run `product graph check --format json`.

**Assertions:**

- `errors[]` contains an entry with `code == "W030"` — the code is stable across tiers.
- `warnings[]` does **not** contain a W030 entry.
- Exit code is `1` (errors present).
- The error entry's `tier` field is `"error"`.
- The same code string lets CI filters like `grep W030` continue to work regardless of tier.
