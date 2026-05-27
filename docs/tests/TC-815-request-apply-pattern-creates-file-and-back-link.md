---
id: TC-815
title: request_apply_pattern_creates_file_and_back_link
type: scenario
status: unimplemented
validates:
  features:
  - FT-070
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_815_request_apply_pattern_creates_file_and_back_link
---

## Description

Compose a temp repo with feature `FT-100`. Apply the following
request via `product_request_apply`:

```yaml
type: create
schema-version: 1
artifacts:
  - type: pattern
    title: "MCP tool with disk side-effect"
    status: live
    adrs: [ADR-020]
    examples: [FT-100]
```

Assert:

1. The response `created` array contains exactly one entry of
   `kind: pattern` with the allocated PAT id.
2. The new pattern file exists on disk at
   `docs/patterns/PAT-NNN-mcp-tool-with-disk-side-effect.md`.
3. The pattern file's front-matter parses to the expected shape,
   including `examples: [FT-100]`.
4. The feature file `docs/features/FT-100-*.md` has been updated in
   the same atomic batch — `patterns: [PAT-NNN]` is present.
5. `requests.jsonl` contains one new entry whose hash-chain
   verifies (FT-042 invariant) and whose `created` list matches the
   response.

## Formal specification

⟦Λ:Scenario⟧
Given a repository containing FT-100,
When the user submits a `product_request_apply` payload creating a
  pattern with `examples: [FT-100]`,
Then a new pattern file exists on disk with the correct
  front-matter and body scaffolding,
And FT-100's `patterns` array contains the new PAT id,
And `requests.jsonl` records the atomic batch with a valid
  hash-chain link to the previous entry.

⟦Ε⟧⟨δ≜1.0;φ≜1;τ≜◊⁺⟩
