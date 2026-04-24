---
id: TC-692
title: absent_section_emits_w030
type: scenario
status: unimplemented
validates:
  features:
  - FT-055
  adrs:
  - ADR-047
phase: 1
---

**Covers session test ST-351** — `absent-section-emits-w030`.

Verifies that an entirely missing section heading triggers W030 — the primary, most-common case.

**Setup:**

- Feature body contains `## Description` and `## Functional Specification` with all seven subsections, but no `## Out of scope` H2.

**Steps:**

1. Run `product graph check --format json`.

**Assertions:**

- W030 warning is emitted listing `Out of scope` as a missing top-level section.
- The W030 `detail` field lists exactly one missing section (no spurious duplicates).
- Adding `## Out of scope` with a single bullet satisfies the check on re-run.

**Negative check:**

- A feature with `## out-of-scope` (kebab-case) instead of `## Out of scope` still triggers W030 — the match is exact after trimming, not a slug comparison.
