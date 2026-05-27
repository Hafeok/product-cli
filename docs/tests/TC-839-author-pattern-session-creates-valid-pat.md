---
id: TC-839
title: author_pattern_session_creates_valid_pat
type: scenario
status: unimplemented
validates:
  features:
  - FT-073
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_839_author_pattern_session_creates_valid_pat
---

## Description

Drive an `author-pattern` session through a session-based test
harness (per `tests/sessions/` conventions from ADR-018). The
session loads the `author-pattern-v1.md` prompt, scaffolds a new
PAT via `product_pattern_new`, fills the required body sections,
links one ADR via `product_pattern_link`, and closes by calling
`product_graph_check` with no PAT-related findings against the
new PAT.

Assert:

1. A file exists on disk at
   `docs/patterns/PAT-NNN-<slug>.md` after the session.
2. The file's front-matter is valid per the FT-070 schema.
3. The file body contains all five required H2 sections (per
   ADR-050 / FT-071's body validation).
4. The loaded graph (read via `parser::load_all`) exposes the
   new PAT node in the `patterns` map.
5. The request log (`requests.jsonl`) carries an entry recording
   the `product_pattern_new` write, with a valid hash-chain link
   to the previous entry.

## Formal specification

⟦Λ:Scenario⟧
Given an `author-pattern` session loading the v1 system prompt,
When the agent scaffolds, fills, and closes a new pattern,
Then a valid PAT file exists on disk with all required
  sections,
And the loaded graph exposes the new PAT,
And the request log records the write with a valid hash-chain
  link.

⟦Ε⟧⟨δ≜1.0;φ≜1;τ≜◊⁺⟩
