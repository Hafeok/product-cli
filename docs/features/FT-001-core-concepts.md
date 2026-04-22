---
id: FT-001
title: Core Concepts
phase: 1
status: complete
depends-on: []
adrs:
- ADR-001
- ADR-004
- ADR-005
tests:
- TC-001
- TC-002
- TC-003
- TC-004
- TC-011
- TC-012
- TC-013
- TC-014
- TC-015
- TC-156
domains:
- data-model
domains-acknowledged:
  ADR-040: predates ADR-040; feature ships independently of the unified verify / LLM-boundary pipeline
  ADR-042: Pre-dates ADR-042; this feature does not define TC types or validate the type vocabulary. FT-048 owns the mechanics.
  ADR-018: Predates the 2026-04-22 scope promotion of ADR-018 to cross-cutting. Test coverage reflects the property/session/benchmark strategy as it existed when this feature shipped; not retroactively reclassified.
  ADR-041: Pre-dates ADR-041; this feature does not author absence TCs or set removes/deprecates on ADRs. FT-047 owns the mechanics.
  ADR-043: Predates ADR-043; feature command adapters were written before the slice+adapter pattern was formalised and are not retroactively refactored.
---

### Artifact Types

**Feature (`FT-XXX`)** — A unit of product capability. Corresponds to a section of a PRD. Declares its phase, status, linked ADRs, and linked test criteria. A feature is the primary navigation unit of the knowledge graph: everything else is reachable from it.

**Architectural Decision Record (`ADR-XXX`)** — A single architectural decision. Declares context, decision, rationale, rejected alternatives, and the features it applies to. An ADR may apply to multiple features. An ADR may supersede or be superseded by another ADR.

**Test Criterion (`TC-XXX`)** — A single verifiable assertion about system behaviour. A test criterion has a type (scenario, invariant, chaos, exit-criteria), is linked to one or more features and one or more ADRs, and belongs to a phase. Test criteria are extracted from ADRs during migration — they are not co-located with the decisions they verify.

### Relationships

```
Feature ──── implementedBy ────► ADR
Feature ──── validatedBy ───────► TestCriterion
ADR     ──── testedBy ──────────► TestCriterion
ADR     ──── supersedes ────────► ADR
```

Edges are declared in the *source* artifact's front-matter. The derived graph is bidirectional — every edge is traversable in both directions by the CLI.

### The Derived Graph

Product reads all front-matter declarations on every command invocation and builds an in-memory graph. There is no persistent graph store. The graph is always consistent with the files. `product graph rebuild` writes `index.ttl` as a snapshot for external tooling, but this file is never read by Product itself.

### The Context Bundle

A context bundle is a single markdown document containing a feature, all its linked ADRs, and all its linked test criteria — assembled in a deterministic order and formatted for direct injection into an LLM context window. This is the primary output of Product. Everything else in the tool exists to make context bundles accurate and complete.

---