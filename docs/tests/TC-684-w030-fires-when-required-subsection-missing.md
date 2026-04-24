---
id: TC-684
title: w030_fires_when_required_subsection_missing
type: scenario
status: unimplemented
validates:
  features:
  - FT-055
  adrs:
  - ADR-047
phase: 1
---

**Covers session test ST-343** — `w030-fires-when-required-subsection-missing`.

Verifies that W030 fires when `## Functional Specification` is present but one or more of its required H3 subsections is missing.

**Setup:**

- Feature body contains `## Description`, `## Functional Specification` with only `### Inputs` and `### Outputs`, and `## Out of scope`.
- Missing subsections: State, Behaviour, Invariants, Error handling, Boundaries.

**Steps:**

1. Run `product graph check --format json`.

**Assertions:**

- Exit code is `2`.
- A single W030 warning is emitted for this feature (not five — one warning per feature with the missing subsection list in `detail`).
- The `detail` string lists each missing subsection as `Functional Specification > <name>` (e.g. `Functional Specification > Behaviour`).
- The parent section `Functional Specification` is *not* listed as missing — it is present; only its required subsections are absent.
