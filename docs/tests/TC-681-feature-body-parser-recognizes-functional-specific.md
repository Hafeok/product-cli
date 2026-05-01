---
id: TC-681
title: feature_body_parser_recognizes_functional_specification_section
type: scenario
status: passing
validates:
  features:
  - FT-055
  adrs:
  - ADR-047
phase: 1
runner: cargo-test
runner-args: "tc_681_feature_body_parser_recognizes_functional_specification_section"
last-run: 2026-04-30T09:25:16.632272257+00:00
last-run-duration: 0.6s
---

**Covers session test ST-340** — `feature-body-parser-recognizes-functional-specification-section`.

Verifies that the pure `parse_body_sections` function in `src/feature/body_sections.rs` detects the `## Functional Specification` H2 heading in a markdown feature body.

**Given** a feature body string containing:

```markdown
## Description

Some prose.

## Functional Specification

### Inputs

- foo
```

**When** `parse_body_sections(body)` is called.

**Then** the returned `BodySections` struct has `"Functional Specification"` in its `h2` vector.

**Negative cases covered:**

- Body with `## functional specification` (lowercase) — not recognised (case-sensitive).
- Body with `## Functional Specification:` (trailing colon) — not recognised (exact trimmed text).
- Body where the phrase appears inside a fenced code block — not recognised.