---
id: TC-682
title: feature_body_parser_recognizes_all_subsections
type: scenario
status: unimplemented
validates:
  features: []
  adrs: []
phase: 1
---

**Covers session test ST-341** — `feature-body-parser-recognizes-all-subsections`.

Verifies that `parse_body_sections` identifies every H3 heading nested beneath the `## Functional Specification` H2 and attributes each to its parent section.

**Given** a feature body containing `## Functional Specification` followed by all seven default subsections (Inputs, Outputs, State, Behaviour, Invariants, Error handling, Boundaries) as `### ...` H3 headings with at least one non-whitespace line each.

**When** `parse_body_sections(body)` is called.

**Then** `sections.h3_under.get("Functional Specification")` returns a set containing all seven subsection names in the order they appear.

**Additional assertions:**

- H3 headings appearing *outside* the Functional Specification section (e.g. under `## Description`) are not attributed to `h3_under["Functional Specification"]`.
- Duplicate subsection headings under the same parent are de-duplicated in the returned set.
- H4 and deeper headings under a subsection do not appear in `h3_under`.
