---
id: FT-015
title: Test Criteria
phase: 1
status: complete
depends-on: []
adrs:
- ADR-011
- ADR-016
- ADR-018
tests:
- TC-035
- TC-036
- TC-037
- TC-038
- TC-039
- TC-040
- TC-071
- TC-072
- TC-073
- TC-074
- TC-075
- TC-076
- TC-077
- TC-078
- TC-079
- TC-153
domains:
- data-model
domains-acknowledged:
  ADR-047: Predates ADR-047; this feature does not author the functional-spec body convention. FT-055 owns the structural validator and W030 mechanics.
  ADR-048: Predates ADR-048; this feature does not author the canonical .product/ folder layout. FT-057 owns the migration command and discovery fallback.
  ADR-041: Pre-dates ADR-041; this feature does not author absence TCs or set removes/deprecates on ADRs. FT-047 owns the mechanics.
  ADR-042: Pre-dates ADR-042; this feature does not define TC types or validate the type vocabulary. FT-048 owns the mechanics.
  ADR-043: Predates ADR-043; feature command adapters were written before the slice+adapter pattern was formalised and are not retroactively refactored.
  ADR-040: predates ADR-040; feature ships independently of the unified verify / LLM-boundary pipeline
---

### TC-001 — Binary Compiles (exit-criteria)

[prose description]

⟦Λ:ExitCriteria⟧{
  binary_size < 20MB
  compile_time(rpi5, cold) < 5min
  ldd(binary) = {libc}
}
⟦Ε⟧⟨δ≜0.98;φ≜100;τ≜◊⁺⟩

### TC-002 — Raft Leader Election (scenario)

[prose description]

⟦Σ:Types⟧{ Node≜IRI; Role≜Leader|Follower|Learner }
⟦Γ:Invariants⟧{ ∀s:ClusterState: |{n | roles(n)=Leader}| = 1 }
⟦Λ:Scenario⟧{
  given≜cluster_init(nodes:2)
  when≜elapsed(10s)
  then≜∃n∈nodes: roles(n)=Leader ∧ graph_contains(n, picloud:hasRole, picloud:Leader)
}
⟦Ε⟧⟨δ≜0.95;φ≜100;τ≜◊⁺⟩
```

The bundle evidence block `⟦Ε⟧` at the top is computed as the mean of all linked test criterion `δ` values (confidence), and the percentage of criteria with formal blocks present (`φ`). An agent receiving this bundle can assess the specification quality before reading the full content.

YAML front-matter is stripped from all sections. Formal blocks in test criteria are preserved verbatim — they are the specification, not metadata.

---

```

---

## Description

Test criteria (TC-XXX) are first-class artifacts in the knowledge graph, stored as individual markdown files with YAML front-matter in `docs/tests/`. They use a hybrid format: YAML front-matter for graph metadata, AISP-influenced formal blocks for constraints and invariants (ADR-011), and plain prose for human-readable description. The formal block notation eliminates LLM interpretation variance in constraint definitions. Formal blocks are mandatory for `invariant` and `chaos` type criteria; optional but encouraged for `scenario` and `exit-criteria`.

## Functional Specification

### Inputs

- TC file body containing:
  - YAML front-matter: `id`, `title`, `type`, `status`, `validates.features`, `validates.adrs`, `phase`, `runner`, `runner-args`
  - Prose description (`## Description` section)
  - Formal blocks: `⟦Σ:Types⟧`, `⟦Γ:Invariants⟧`, `⟦Λ:Scenario⟧`, `⟦Λ:ExitCriteria⟧`, `⟦Λ:Benchmark⟧`, `⟦Ε⟧` (evidence block)
- TC type vocabulary: `scenario`, `invariant`, `chaos`, `exit-criteria`, `benchmark` (ADR-011); custom types may be declared in `product.toml` (ADR-042)

### Outputs

- TC nodes in the in-memory knowledge graph with edges: `validates` Feature(s), `validates` ADR(s)
- Formal blocks parsed into a typed AST (`FormalBlock` enum) for validation; raw text preserved for context bundle output (ADR-016)
- Evidence block fields (`δ`, `φ`, `τ`) surfaced in context bundle aggregate metrics

### State

TC status (`unimplemented`, `passing`, `failing`) is stored in front-matter and updated by `product verify`. Formal blocks and graph links are stable specification data; `status` and `last-run` fields are mutable by the verify pipeline.

### Behaviour

1. Parser reads TC front-matter and body; formal blocks are extracted using the hand-written recursive descent parser defined in ADR-016.
2. Graph builder adds a TC node and `validates` edges to all referenced features and ADRs.
3. `product graph check` validates: mandatory formal blocks present for `invariant`/`chaos` types (W004 if absent), evidence block `δ` in [0.0, 1.0] and `φ` in [0, 100] (E001 if out of range), TC type in the declared vocabulary (E006 for unknown custom type).
4. `product test new` scaffolds a new TC file with formal block stubs; `runner` and `runner-args` fields are required before `product verify` will execute the TC.
5. When a feature is abandoned, the feature's ID is auto-removed from all linked TCs' `validates.features` lists (ADR-010); TCs with empty `validates.features` become orphaned warnings (W001).
6. Formal blocks are preserved verbatim in context bundles (raw text round-trip); the AST is used only for validation, not for reformatting output.

### Invariants

- Every TC ID is unique across the repository (E-series duplicate check).
- `δ` must be in [0.0, 1.0]; `φ` must be in [0, 100] — parser enforces this as E001 (ADR-016).
- `invariant` and `chaos` type TCs without formal blocks produce W004 warnings (ADR-011).
- TC front-matter fields not recognised by the current schema are preserved on write (forward-compatible unknown-field handling, ADR-014).
- Formal block raw text is round-tripped byte-for-byte through the context bundle — no re-formatting (ADR-016).

### Error handling

- E001: malformed formal block delimiter or invalid expression; reported with file path and line number. Subsequent blocks in the same file are still parsed.
- W004: empty formal block body (`⟦Γ:Invariants⟧{}`); syntactically valid but semantically meaningless.
- W001: orphaned TC (empty `validates.features`); surfaced by `product graph check` after feature abandonment.
- E006: unknown TC type not in `product.toml` custom vocabulary; blocks context assembly for that TC.

### Boundaries

- TC files are authored by humans or `product test new`; the CLI does not generate TC content from feature prose.
- Full semantic verification of formal expressions is not performed — the parser validates structure and field ranges, not logical correctness (ADR-016).
- TC execution (running tests) is out of scope for this feature; that is handled by `product verify`.

## Out of scope

- Generating TC content from feature descriptions (LLM-assisted TC authoring is a separate concern).
- Executing TCs or interpreting their pass/fail result (that is `product verify` / FT-007).
- Defining the runner infrastructure (runner configuration is specified in front-matter; execution is delegated to cargo or the configured test runner).
