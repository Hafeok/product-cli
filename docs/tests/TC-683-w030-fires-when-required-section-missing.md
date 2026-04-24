---
id: TC-683
title: w030_fires_when_required_section_missing
type: scenario
status: unimplemented
validates:
  features:
  - FT-055
  adrs:
  - ADR-047
phase: 1
---

**Covers session test ST-342** — `w030-fires-when-required-section-missing`.

Verifies that `product graph check` emits `warning[W030]` when a feature body is missing one of the top-level sections configured in `[features].required-sections`.

**Setup:**

- Fixture repo with a feature FT-XXX at `phase: 1` whose body contains only `## Description` — no `## Functional Specification`, no `## Out of scope`.
- `product.toml` uses default `[features].required-sections` (`["Description", "Functional Specification", "Out of scope"]`).

**Steps:**

1. Run `product graph check --format json`.
2. Parse the JSON output.

**Assertions:**

- Exit code is `2` (warnings only, no errors).
- `warnings[]` contains an entry with `code == "W030"`.
- That entry's `file` matches the feature path.
- Its `detail` contains both missing section names: `"Functional Specification"` and `"Out of scope"`.
- The entry's `hint` directs the user to `product request change ... op: set ... field: body`.
