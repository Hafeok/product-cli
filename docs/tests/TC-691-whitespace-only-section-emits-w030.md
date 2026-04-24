---
id: TC-691
title: whitespace_only_section_emits_w030
type: scenario
status: unimplemented
validates:
  features:
  - FT-055
  adrs:
  - ADR-047
phase: 1
---

**Covers session test ST-350** — `whitespace-only-section-emits-w030`.

Verifies that a section heading followed only by blank lines (no non-whitespace content before the next same-or-higher-level heading) is treated as absent and triggers W030.

**Setup:**

- Feature body contains:
  ```markdown
  ### State

  

  ### Behaviour
  ...
  ```
- All other required sections and subsections are present with real content.

**Steps:**

1. Run `product graph check --format json`.

**Assertions:**

- W030 is emitted listing `Functional Specification > State` as missing.
- The empty body between `### State` and `### Behaviour` is treated as a missing section, not an empty-meaning section.
- No false positive on `### Behaviour` — its content is non-whitespace.
