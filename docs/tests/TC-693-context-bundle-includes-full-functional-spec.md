---
id: TC-693
title: context_bundle_includes_full_functional_spec
type: scenario
status: passing
validates:
  features:
  - FT-055
  adrs:
  - ADR-006
  - ADR-047
phase: 1
runner: cargo-test
runner-args: "tc_693_context_bundle_includes_full_functional_spec"
last-run: 2026-04-30T09:25:16.632272257+00:00
last-run-duration: 0.3s
---

**Covers session test ST-352** — `context-bundle-includes-full-functional-spec`.

Verifies that `product context FT-NNN --depth 2` includes the entire feature body — including the Functional Specification section and every subsection — with no truncation.

**Setup:**

- Feature with a complete functional specification (all seven subsections, each multi-paragraph), linked ADRs, and TCs.

**Steps:**

1. Run `product context FT-NNN --depth 2`.
2. Capture stdout.

**Assertions:**

- The output contains every required subsection heading (`### Inputs`, `### Outputs`, ..., `### Boundaries`).
- The output contains sample content from the middle of the body — confirming that the bundle is not a header-only view.
- The feature body appears in the bundle in the documented order (before the ADR section, before the TC section — per ADR-006 / ADR-025 bundle order).
- Running with `--format json` includes the body under the feature's artifact entry as a string field.