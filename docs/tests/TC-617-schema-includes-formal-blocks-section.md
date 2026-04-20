---
id: TC-617
title: schema_includes_formal_blocks_section
type: scenario
status: passing
validates:
  features:
  - FT-049
  adrs:
  - ADR-031
phase: 5
runner: cargo-test
runner-args: "tc_617_schema_includes_formal_blocks_section"
last-run: 2026-04-20T13:17:28.076057385+00:00
last-run-duration: 0.2s
---

## Session — schema-includes-formal-blocks-section

### Given

A freshly built `product` binary and a clean fixture repo with
`product.toml` initialised.

### When

The user runs `product schema` with no `--type` flag.

### Then

- The output contains a top-level `## Formal Blocks` section emitted after
  `## Dependency`.
- Within that section, all five AISP block names appear verbatim using the
  parser-accepted spellings: Sigma-Types, Gamma-Invariants, Lambda-Scenario,
  Lambda-ExitCriteria, and Epsilon (the Evidence block). See
  `src/formal/parser.rs::parse_formal_blocks_with_diagnostics` for the
  authoritative list.
- The Test Criterion schema section contains a cross-reference line
  pointing at `Formal Blocks` (so an LLM reading `type: invariant`
  discovers the block spec without external context).

### And

Each block name's sub-section names which `tc-type` values require that
block, matching the W004 / G002 contract from FT-048: `invariant` TCs
require the Gamma-Invariants (or Sigma-Types) block; `chaos` TCs require
Gamma-Invariants (or Lambda-Scenario); `exit-criteria` TCs require
Lambda-ExitCriteria.