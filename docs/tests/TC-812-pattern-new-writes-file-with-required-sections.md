---
id: TC-812
title: pattern_new_writes_file_with_required_sections
type: scenario
status: unimplemented
validates:
  features:
  - FT-070
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_812_pattern_new_writes_file_with_required_sections
---

## Description

Compose a temp repo with `[paths].patterns = "docs/patterns"` and the
default `[patterns].body-sections` list from ADR-050 (five entries:
"When to use", "Prerequisites", "The pattern", "Anti-patterns",
"Worked example"). Run `product pattern new "Slice + Adapter module
structure"` against the temp repo.

Assert:

1. A file appears at the path `docs/patterns/PAT-001-slice-adapter-module-structure.md`
   (id allocator assigns PAT-001 in a fresh repo).
2. Reading the file with the parser yields a `PatternFrontMatter`
   with `id: "PAT-001"`, `title: "Slice + Adapter module structure"`,
   `status: Live`, and empty arrays for every link field.
3. The markdown body contains every configured H2 heading at least
   once (case-sensitive). Missing any one heading must fail this TC.
4. The TC asserts on **the file**, not on the CLI stdout — the
   stdout success line is necessary but not sufficient (ADR-051).

## Formal specification

⟦Λ:Scenario⟧
Given an empty product-cli repository configured with default
  `[patterns]` settings,
When the user runs `product pattern new "Slice + Adapter module
  structure"`,
Then the file `docs/patterns/PAT-001-slice-adapter-module-structure.md`
  exists,
And its front-matter parses to `{ id: PAT-001, title: "Slice +
  Adapter module structure", status: live, adrs: [], requires: [],
  examples: [], domains: [] }`,
And its body contains the H2 headings "When to use",
  "Prerequisites", "The pattern", "Anti-patterns", "Worked example".

⟦Ε⟧⟨δ≜1.0;φ≜1;τ≜◊⁺⟩
