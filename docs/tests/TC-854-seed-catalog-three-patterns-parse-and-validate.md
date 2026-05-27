---
id: TC-854
title: seed_catalog_three_patterns_parse_and_validate
type: scenario
status: passing
validates:
  features:
  - FT-075
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_854_seed_catalog_three_patterns_parse_and_validate
last-run: 2026-05-27T15:36:27.790359954+00:00
last-run-duration: 2.5s
---

## Description

Apply the seed batch (three patterns + reciprocal feature
writes) against a fresh temp repo. After application:

Assert:

1. The three files
   `docs/patterns/PAT-001-slice-adapter-module-structure.md`,
   `docs/patterns/PAT-002-mcp-tool-with-disk-side-effect.md`,
   `docs/patterns/PAT-003-tc-authoring-observability-and-causation.md`
   exist on disk.
2. Each file parses successfully via `parser::load_all`.
3. The reloaded graph exposes all three in
   `graph.patterns` keyed by id.
4. `product graph check` against the resulting repo exits 0
   (zero errors, zero warnings introduced by the seed batch).
5. The `requests.jsonl` chain verifies (FT-042 invariant).

## Formal specification

⟦Λ:Scenario⟧
Given a fresh temp repo,
When the F6 seed batch is applied,
Then the three PAT files exist and parse,
And the graph exposes them in the patterns map,
And `product graph check` exits 0,
And the request log hash-chain verifies.

⟦Ε⟧⟨δ≜1.0;φ≜1;τ≜◊⁺⟩