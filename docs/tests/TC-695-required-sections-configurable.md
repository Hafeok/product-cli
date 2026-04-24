---
id: TC-695
title: required_sections_configurable
type: scenario
status: unimplemented
validates:
  features: []
  adrs: []
phase: 1
---

**Covers session test ST-354** — `required-sections-configurable`.

Verifies that `[features].required-sections` in `product.toml` overrides the default top-level section list.

**Setup:**

- `product.toml` sets:
  ```toml
  [features]
  required-sections = ["Description", "Acceptance criteria"]
  ```
  (deliberately omitting "Functional Specification" and "Out of scope", adding "Acceptance criteria").
- Feature body contains `## Description` and `## Functional Specification` but no `## Acceptance criteria`.

**Steps:**

1. Run `product graph check --format json`.

**Assertions:**

- W030 warning lists `Acceptance criteria` as missing — the custom configuration is honoured.
- The default required sections (Functional Specification, Out of scope) are **not** reported as missing — they are no longer required under the override.
- Setting `required-sections = []` suppresses top-level W030 entirely for any feature.
