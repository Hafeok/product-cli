---
id: TC-694
title: context_bundle_preserves_subsection_structure
type: scenario
status: unimplemented
validates:
  features:
  - FT-055
  adrs:
  - ADR-006
  - ADR-047
phase: 1
---

**Covers session test ST-353** — `context-bundle-preserves-subsection-structure`.

Verifies that the subsection structure (H2/H3 nesting) survives bundle assembly without reformatting — the LLM receives the headings the author wrote, unchanged.

**Setup:**

- Feature body contains `## Functional Specification` with mixed-content subsections (numbered lists, bullet lists, code fences, tables).

**Steps:**

1. Run `product context FT-NNN --depth 1`.

**Assertions:**

- Every `### <subsection>` heading appears verbatim in the bundle.
- Code fences inside subsections (e.g. fenced sample YAML) appear verbatim — not escaped, not re-wrapped.
- The `## Out of scope` heading appears in the bundle at the H2 level, not promoted or demoted.
- Character-for-character equality between the bundled body slice and the on-disk body (modulo a possible leading/trailing newline added by the bundler).
