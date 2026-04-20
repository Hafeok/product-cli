---
id: TC-618
title: schema_type_formal_returns_just_formal_section
type: scenario
status: unimplemented
validates:
  features:
  - FT-049
  adrs:
  - ADR-031
phase: 5
runner: cargo-test
runner-args: "tc_618_schema_type_formal_returns_just_formal_section"
---

## Session — schema-type-formal-returns-just-formal-section

### Given

A freshly built `product` binary and a clean fixture repo.

### When

The user runs `product schema --type formal`.

### Then

- The exit code is 0.
- The output contains the five AISP block names using the parser-accepted
  spellings: Sigma-Types, Gamma-Invariants, Lambda-Scenario,
  Lambda-ExitCriteria, and the Epsilon evidence block.
- The output does **not** contain any of the other schema section
  headings (`## Feature`, `## ADR`, `## Test Criterion`, `## Dependency`) —
  this is the targeted render only.

### And

`product schema --type unknown` still returns a non-zero exit and the
existing error hint, unchanged.
