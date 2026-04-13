---
id: ADR-011
title: AISP-Influenced Formal Notation for Test Criteria
status: accepted
features: []
supersedes: []
superseded-by: []
domains: []
scope: domain
---

**Status:** Accepted

**Context:** Test criteria files currently express constraints, invariants, and assertions in natural language prose. When an LLM implementation agent receives a context bundle, it must interpret prose like "exactly one node holds the Leader role at all times" and infer the precise semantics. Two agents, or the same agent on two invocations, may interpret this differently — producing implementations with subtly different invariant checks.

AISP (AI Symbolic Protocol) is a formal notation language designed to reduce LLM interpretation variance. Its key insight is that symbolic, typed expressions with formal semantics have near-zero ambiguity, whereas natural language descriptions of the same constraints have 40–65% interpretation variance. Rather than adopting AISP wholesale, we evaluated where its notation patterns deliver the most value in Product's artifact model.

Test criteria are the highest-value target for formal notation because:
- They express assertions that must be verified, not explained
- Ambiguity in a constraint definition leads directly to incorrect implementations or missed test cases
- They are consumed primarily by agents, not humans reading for understanding

ADR prose (context, rationale, rejected alternatives) is explicitly excluded from this decision — that content is argumentative and explanatory, where prose is the correct medium.

**Decision:** Test criterion files use a hybrid format: YAML front-matter for graph metadata, AISP-influenced formal blocks for constraints and invariants, and plain prose for the human-readable description only. The formal blocks are mandatory for `invariant` and `chaos` type test criteria. They are optional but encouraged for `scenario` and `exit-criteria` types.

**Format:**

```markdown
---
id: TC-002
title: Raft Leader Election
type: scenario
status: unimplemented
validates:
  features: [FT-001]
  adrs: [ADR-002]
phase: 1
---

## Description

Bootstrap a two-node cluster. Assert that exactly one node is elected leader
within 10 seconds, and that the leader identity is reflected in the RDF graph.

## Formal Specification

⟦Σ:Types⟧{
  Node≜IRI
  Role≜Leader|Follower|Learner
  ClusterState≜⟨nodes:Node+, roles:Node→Role⟩
}

⟦Γ:Invariants⟧{
  ∀s:ClusterState: |{n∈s.nodes | s.roles(n)=Leader}| = 1
}

⟦Λ:Scenario⟧{
  given≜cluster_init(nodes:2)
  when≜elapsed(10s)
  then≜∃n∈nodes: roles(n)=Leader
       ∧ graph_contains(n, picloud:hasRole, picloud:Leader)
}

⟦Ε⟧⟨δ≜0.95;φ≜100;τ≜◊⁺⟩
```

**Block semantics:**

| Block | Symbol | Purpose | Required for type |
|---|---|---|---|
| `⟦Σ:Types⟧` | Type definitions | Name the domain types used in rules | invariant, chaos |
| `⟦Γ:Invariants⟧` | Constraint rules | Formal assertions that must hold | invariant, chaos |
| `⟦Λ:Scenario⟧` | Given/when/then | Structured test flow | scenario |
| `⟦Λ:ExitCriteria⟧` | Measurable thresholds | Numeric pass/fail bounds | exit-criteria |
| `⟦Λ:Benchmark⟧` | Quality measurement | Conditions, scorer, pass threshold | benchmark |
| `⟦Ε⟧` | Evidence block | Confidence, coverage, stability | all types |

**Evidence block fields:**

| Field | Meaning | Range |
|---|---|---|
| `δ` | Specification confidence | 0.0–1.0 |
| `φ` | Coverage completeness (%) | 0–100 |
| `τ` | Stability signal | `◊⁺` (stable), `◊⁻` (unstable), `◊?` (unknown) |

**Symbol subset in use:**

Product uses a minimal AISP symbol subset, not the full specification. Only these symbols appear in Product test criteria:

| Symbol | Meaning |
|---|---|
| `≜` | Definition ("is defined as") |
| `≔` | Assignment |
| `∀` | For all |
| `∃` | There exists |
| `∧` | Logical and |
| `∨` | Logical or |
| `→` | Function type or implication |
| `⟨⟩` | Tuple or record |
| `\|` | Union type (in type definitions) |
| `⟦⟧` | Block delimiter |

This subset is sufficient for all constraint and invariant patterns encountered in the PiCloud ADRs. Full AISP notation (category theory operators, tri-vector decomposition, ghost intent search) is not adopted — it exceeds what is needed and would make files unreadable to contributors unfamiliar with the full spec.

**Rationale:**
- The formal blocks are consumed by LLM agents receiving context bundles. Replacing prose invariants with typed, symbolic expressions eliminates interpretation decisions at the agent side — the constraint is unambiguous
- The hybrid approach preserves human readability: the prose description remains the primary entry point for a human reading the file. The formal blocks are additive, not a replacement
- `⟦Γ:Invariants⟧` maps exactly to the invariant patterns already present in the PiCloud ADRs ("exactly one leader", "log index is strictly monotonically increasing") — this is not a new concept, it is a more precise notation for concepts already being expressed
- The `⟦Λ:Scenario⟧` given/when/then pattern is equivalent to Gherkin (BDD) but typed — agents familiar with either convention recognise it immediately
- The evidence block `⟦Ε⟧` makes specification confidence explicit and queryable. `product graph stats` can report aggregate confidence across all test criteria
- The symbol subset is stable: every symbol used is in Unicode's standard mathematical operators block, renders correctly in any markdown viewer, and is representable in all major editors without custom font configuration

**Rejected alternatives:**
- **Full AISP adoption** — the complete AISP 5.1 spec includes category theory constructs, tri-vector signal decomposition, and proof-by-layers that go well beyond what test criteria need. Full adoption would make files unreadable to contributors not trained in the spec. Rejected: overhead exceeds benefit.
- **Gherkin (BDD) format** — `Given/When/Then` in plain English. More familiar to many engineers, good tooling. Rejected because it still relies on natural language for the assertion content — `"Then exactly one leader exists"` has the same interpretation problem as prose. Gherkin structures the test but does not eliminate ambiguity in the assertion.
- **JSON Schema / OpenAPI assertions** — machine-readable, well-tooled. Rejected because JSON is not a natural fit for logical quantifiers (`∀`, `∃`) and temporal assertions (`within 10s`). The resulting schemas are verbose and hard to scan.
- **Keep prose only** — minimal friction for authors. Rejected because the context bundle's primary consumer is an LLM agent, and prose invariants demonstrably require interpretation decisions that formal notation eliminates.

**Migration:**

Existing test criteria extracted from the PiCloud ADRs are in prose. Migration is incremental:
1. `product test new` scaffolds new criteria with formal block stubs
2. Existing criteria get prose descriptions only — the formal blocks are absent, not malformed
3. `product graph check` reports criteria with missing formal blocks as warnings (not errors) when the criterion type is `invariant` or `chaos`
4. `product graph stats` reports `φ` (formal coverage) — the percentage of invariant/chaos criteria that have formal blocks — so coverage is visible without being a hard gate