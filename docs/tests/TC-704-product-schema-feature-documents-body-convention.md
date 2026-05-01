---
id: TC-704
title: product_schema_feature_documents_body_convention
type: scenario
status: passing
validates:
  features:
  - FT-055
  adrs:
  - ADR-047
phase: 5
runner: cargo-test
runner-args: "schema_feature_documents_body_structure_convention"
last-run: 2026-04-30T09:25:16.632272257+00:00
last-run-duration: 0.2s
---

**Test Type:** scenario

**Why this TC exists:**

FT-055 declared as a deliverable that `product schema feature`
should document the body-structure convention (required H2 / H3
sections) "in addition to the YAML front-matter schema". The
original headless implementation landed the parser, the W030
warning, and the config keys, but the schema renderer was never
updated — `product schema feature` printed only the YAML front-matter
block and was silent on the convention. TC-704 codifies the
acceptance check so the gap can't reopen.

**Setup:**

1. Build the product binary (`cargo build`) — no external state needed.

**Execution:**

1. Run `product schema feature` and capture stdout.

**Expected:**

- Stdout contains the heading `### Body Structure Convention (FT-055, ADR-047)`.
- Stdout lists every required H2 section (`## Description`,
  `## Functional Specification`, `## Out of scope`).
- Stdout lists every default H3 subsection under
  `## Functional Specification` (`### Inputs`, `### Outputs`,
  `### State`, `### Behaviour`, `### Invariants`,
  `### Error handling`, `### Boundaries`).
- Stdout references the W030 warning code.
- Stdout names the configured severity (`warning` by default).

**Library-level coverage:**

- `agent_context::schema::feature_schema()` includes the
  body-structure block when called without config.
- `agent_context::schema::feature_schema_with_config(Some(&cfg))`
  honours the project's `[features]` overrides — if a project sets
  `completeness-severity = "error"`, the schema render reflects that.

**Notes:**

- The `runner-args` hooks into the unit test
  `agent_context::schema::tests::schema_feature_documents_body_structure_convention`
  added at the same time as this TC.
- Together with TC-703, this TC closes the FT-057-style failure
  mode: a feature marked `complete` whose user-visible deliverables
  were never wired up.