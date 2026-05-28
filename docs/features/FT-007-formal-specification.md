---
id: FT-007
title: Formal Specification
phase: 1
status: complete
depends-on: []
adrs:
- ADR-004
- ADR-011
tests:
- TC-011
- TC-012
- TC-152
domains:
- data-model
domains-acknowledged:
  ADR-047: Predates ADR-047; this feature does not author the functional-spec body convention. FT-055 owns the structural validator and W030 mechanics.
  ADR-043: Predates ADR-043; feature command adapters were written before the slice+adapter pattern was formalised and are not retroactively refactored.
  ADR-041: Pre-dates ADR-041; this feature does not author absence TCs or set removes/deprecates on ADRs. FT-047 owns the mechanics.
  ADR-040: predates ADR-040; feature ships independently of the unified verify / LLM-boundary pipeline
  ADR-042: Pre-dates ADR-042; this feature does not define TC types or validate the type vocabulary. FT-048 owns the mechanics.
  ADR-048: Predates ADR-048; this feature does not author the canonical .product/ folder layout. FT-057 owns the migration command and discovery fallback.
  ADR-018: Predates the 2026-04-22 scope promotion of ADR-018 to cross-cutting. Test coverage reflects the property/session/benchmark strategy as it existed when this feature shipped; not retroactively reclassified.
---

⟦Σ:Types⟧{
  Graph≜⟨nodes:Node+, edges:Edge*⟩
  CentralityScore≜Float
}

⟦Γ:Invariants⟧{
  ∀g:Graph, ∀n∈g.nodes: betweenness(g,n) ≥ 0.0 ∧ betweenness(g,n) ≤ 1.0
}

⟦Ε⟧⟨δ≜0.99;φ≜100;τ≜◊⁺⟩
```

**Benchmark example:**
```markdown
---
id: TC-030
title: LLM Context Quality — Raft Leader Election
type: benchmark
status: unimplemented
validates:
  features: [FT-001]
  adrs: [ADR-006, ADR-012]
phase: 3
benchmark:
  task: benchmarks/tasks/task-001-raft-leader-election
  rubric: benchmarks/tasks/task-001-raft-leader-election/rubric.md
  conditions: [none, naive, product]
  runs-per-condition: 5
  pass-threshold:
    product: 0.80
    delta-vs-naive: 0.15
---

---

## Description

FT-007 specifies the formal document and formal block contracts that govern how artifact files are parsed and injected into context bundles. It covers two interlocking concerns from ADR-004 and ADR-011. From ADR-004: all artifact files are CommonMark markdown with YAML front-matter; front-matter is stripped before file content is injected into a context bundle; code blocks, tables, and headings are preserved verbatim. From ADR-011: Test Criterion files use a hybrid format where the YAML front-matter carries graph metadata and the file body may contain AISP-influenced formal blocks (`⟦Σ:Types⟧`, `⟦Γ:Invariants⟧`, `⟦Λ:Scenario⟧`, `⟦Λ:Benchmark⟧`, `⟦Ε⟧`) alongside prose; these formal blocks are preserved verbatim in context bundle output and their evidence fields (`δ`, `φ`, `τ`) are aggregated in the bundle header. Together, these rules define the complete format contract between artifact files and the context bundles consumed by LLM agents.

## Functional Specification

### Inputs

- Artifact files (Feature, ADR, Test Criterion) in CommonMark markdown with YAML front-matter.
- For TC files: optional formal block sections in the file body as defined in ADR-011 and ADR-016.
- Context bundle assembly requests (`product context FT-XXX`) specifying a seed artifact and optional depth.

### Outputs

- **Context bundle**: a single markdown document containing the stripped bodies of the seed feature, its linked ADRs (ordered by betweenness centrality), and its linked test criteria. Front-matter delimiters and raw YAML fields are absent from the bundle (TC-011). Code blocks, tables, nested lists, and headings are reproduced verbatim (TC-012).
- **Formal block content**: `⟦Σ:Types⟧`, `⟦Γ:Invariants⟧`, `⟦Λ:Scenario⟧`, `⟦Λ:Benchmark⟧`, and `⟦Ε⟧` blocks within TC file bodies are preserved byte-for-byte in the context bundle (TC-040, TC-152).
- **Evidence aggregation**: the `⟦Ε⟧` evidence fields (`δ` confidence, `φ` coverage, `τ` stability) across all TCs in a bundle are aggregated and surfaced in the bundle's `⟦Ω:Bundle⟧` header block.

### State

Stateless. Context bundles are assembled fresh on every invocation. No bundle cache is maintained. Formal block ASTs are in-memory only; they are rebuilt from file content on every parse.

### Behaviour

1. **Front-matter stripping (ADR-004)**: when a file's body is assembled into a context bundle, the `---`-delimited front-matter block is removed and only the content below the closing `---` is included. The bundle contains no raw YAML fields and no `---` delimiters from front-matter.
2. **Markdown passthrough (ADR-004)**: the stripped body is included in the bundle without format conversion. CommonMark constructs — fenced code blocks, tables, headings, nested lists, blockquotes — are reproduced verbatim. No reformatting or indentation normalisation is applied.
3. **Formal block passthrough (ADR-011)**: formal blocks (`⟦…⟧{…}`) in TC file bodies are included verbatim in the bundle. They are neither reformatted nor interpreted for bundle output — the raw text captured between the outer `{` and `}` delimiters is written to the bundle as-is (ADR-016, `Invariant.raw`).
4. **Evidence aggregation**: after all TC bodies are assembled, Product collects all `⟦Ε⟧` evidence blocks, computes the mean `δ` and `φ` across TCs that have evidence blocks, and writes an aggregate evidence summary to the bundle's `⟦Ω:Bundle⟧` header.
5. **Graph invariant enforced (ADR-012)**: betweenness centrality values used to order ADRs in the bundle are always in [0.0, 1.0]. This is a computed property of the graph; Product validates that the Brandes algorithm produces values in range before writing the bundle.

### Invariants

- The context bundle contains no `---` delimiters and no YAML fields from any artifact's front-matter (TC-011).
- All CommonMark constructs in artifact file bodies are present in the bundle output without modification (TC-012).
- Formal blocks are present in TC section output identical to their source representation (no reformatting).
- Betweenness centrality values for all ADR nodes lie in [0.0, 1.0] (ADR-012).
- Each artifact appears at most once in a bundle, regardless of how many BFS paths reach it (deduplication).

### Error handling

- A file with no `---` delimiters is treated as having an empty front-matter and a full-file body; a warning is emitted but the file content is still included in the bundle (consistent with graceful degradation under ADR-013).
- A formal block that fails to parse (E001) is included in the bundle as raw text (the bytes between the `⟦` and `⟧` delimiters, verbatim), so the bundle is still useful to the agent even when the parser cannot construct a typed AST.
- Evidence blocks with out-of-range `δ` or `φ` values are excluded from the aggregate computation; E001 is emitted for the specific file.

### Boundaries

- The formal block parser handles only the minimal AISP symbol subset defined in ADR-011. Full AISP notation (category theory operators, tri-vector decomposition) is not supported and would produce E001 parse errors.
- The context bundle format itself (the `⟦Ω:Bundle⟧` header, bundle depth semantics, artifact ordering) is covered by FT-011 (Context Bundle Format). FT-007 covers only the per-file content contract (front-matter stripping, markdown passthrough, formal block preservation).
- Evidence aggregation is a read-only summary in the bundle header; it does not update any artifact's front-matter.

## Out of scope

- The formal block grammar specification — defined in ADR-016.
- Context bundle depth semantics and BFS traversal — covered by FT-011 and ADR-012.
- The `benchmark` TC formal block execution — the `⟦Λ:Benchmark⟧` block is preserved in the bundle but its conditions and rubric are consumed by an external runner, not by Product.
- Conversion from other document formats (AsciiDoc, Org-mode) — rejected in ADR-004; only CommonMark is supported.
