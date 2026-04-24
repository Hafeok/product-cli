---
id: TC-685
title: w030_clear_when_all_sections_present
type: scenario
status: unimplemented
validates:
  features:
  - FT-055
  adrs:
  - ADR-047
phase: 1
---

**Covers session test ST-344** — `w030-clear-when-all-sections-present`.

Verifies that W030 does not fire when a feature body contains every required top-level section and every required subsection under Functional Specification.

**Setup:**

- Feature body contains `## Description`, `## Functional Specification` with all seven default H3 subsections, and `## Out of scope` — each with at least one non-whitespace content line.
- Default `[features]` configuration.

**Steps:**

1. Run `product graph check --format json`.

**Assertions:**

- No W030 entries appear in `warnings[]` for this feature.
- Exit code matches the status of *other* checks; adding the fully-specified feature does not introduce W030.
- If the fixture otherwise has a clean graph, exit code is `0` (no warnings, no errors).
