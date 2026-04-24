---
id: TC-696
title: functional_spec_subsections_configurable
type: scenario
status: unimplemented
validates:
  features: []
  adrs: []
phase: 1
---

**Covers session test ST-355** — `functional-spec-subsections-configurable`.

Verifies that `[features].functional-spec-subsections` overrides the default H3 subsection list required under `## Functional Specification`.

**Setup:**

- `product.toml` sets:
  ```toml
  [features]
  functional-spec-subsections = ["Inputs", "Outputs"]
  ```
- Feature body contains `## Functional Specification` with `### Inputs`, `### Outputs`, and `### Behaviour` subsections.

**Steps:**

1. Run `product graph check --format json`.

**Assertions:**

- No W030 warning about subsections — only the two required (`Inputs`, `Outputs`) are enforced, and both are present. Extra subsections (`Behaviour`) do not fire warnings.
- Removing `### Outputs` from the body surfaces W030 with `Functional Specification > Outputs` in `detail`.
- Setting `functional-spec-subsections = []` suppresses subsection W030 entirely, even when `## Functional Specification` is present.
