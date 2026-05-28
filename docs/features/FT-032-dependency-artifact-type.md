---
id: FT-032
title: Dependency Artifact Type
phase: 3
status: complete
depends-on: []
adrs:
- ADR-002
- ADR-030
tests:
- TC-381
- TC-382
- TC-383
- TC-384
- TC-385
- TC-386
- TC-387
- TC-388
- TC-389
- TC-390
- TC-391
- TC-392
- TC-393
- TC-394
- TC-395
- TC-396
- TC-397
- TC-398
- TC-399
- TC-400
- TC-401
- TC-403
- TC-678
domains:
- api
- data-model
domains-acknowledged:
  ADR-041: Pre-dates ADR-041; this feature does not author absence TCs or set removes/deprecates on ADRs. FT-047 owns the mechanics.
  ADR-042: Pre-dates ADR-042; this feature does not define TC types or validate the type vocabulary. FT-048 owns the mechanics.
  ADR-048: Predates ADR-048; this feature does not author the canonical .product/ folder layout. FT-057 owns the migration command and discovery fallback.
  ADR-018: Predates the 2026-04-22 scope promotion of ADR-018 to cross-cutting. Test coverage reflects the property/session/benchmark strategy as it existed when this feature shipped; not retroactively reclassified.
  ADR-047: Predates ADR-047; this feature does not author the functional-spec body convention. FT-055 owns the structural validator and W030 mechanics.
  ADR-043: Predates ADR-043; feature command adapters were written before the slice+adapter pattern was formalised and are not retroactively refactored.
  ADR-040: predates ADR-040; feature ships independently of the unified verify / LLM-boundary pipeline
---

## Description

First-class `DEP-XXX` artifact type for external dependencies (ADR-030). Six types: library, service, api, tool, hardware, runtime. Integrates with preflight (availability checks), context bundles (interface contracts), impact analysis (`product impact DEP-XXX`), gap analysis (G008), and produces a bill of materials (`product dep bom`).

---

## Functional Specification

### Inputs

- Dependency artifact files (`DEP-XXX-*.md`) in the configured dependencies directory, each with YAML front-matter declaring: `id`, `title`, `type` (library | service | api | tool | hardware | runtime), `version`, `status` (active | evaluating | deprecated | migrating), `features[]`, `adrs[]`, `availability-check`, `breaking-change-risk`, and optional `interface` block.
- `product dep list [--type TYPE] [--status STATUS]` — filter flags as inputs.
- `product dep check [DEP-ID | --all]` — dependency ID or `--all` flag.
- `product dep bom [--format json]` — optional format flag.
- `product impact DEP-XXX` — a dependency ID for reverse-graph BFS.
- `product preflight FT-XXX` — reads linked DEP nodes to run availability checks.

### Outputs

- **`product dep list`** — table of all dependencies with id, title, type, version, and status. `--format json` emits a JSON array.
- **`product dep show DEP-XXX`** — full dependency detail including interface block and availability-check command.
- **`product dep features DEP-XXX`** — list of features using the dependency.
- **`product dep check`** — runs each dependency's `availability-check` shell command; prints pass/fail per dependency. Exit 0 if all pass, exit 2 if any fail (warnings).
- **`product dep bom`** — structured bill of materials grouped by type (libraries, services, hardware, …) with versions and risk levels. JSON variant is suitable for security audit pipelines.
- **`product impact DEP-XXX`** — reverse-graph BFS output listing all directly and transitively affected features and ADRs.
- **`product preflight FT-XXX`** — dependency availability section appended to preflight output; failures are warnings (exit 2), not hard errors.
- **`product gap check`** — emits G008 when a feature has a `uses` edge to a DEP with no governing ADR (`governs` edge).
- **`product graph check`** — emits W013 when a feature is linked to a dependency with `status: deprecated` or `migrating`.

### State

Stateless. Dependency metadata is read from YAML front-matter on every invocation; no runtime cache is maintained. The knowledge graph is rebuilt from front-matter on every command (ADR-003). `availability-check` commands are executed fresh each time `product dep check` is called.

### Behaviour

1. **Parsing** — `DependencyFrontMatter` in `src/dep_types.rs` deserialises all six dependency types. The `availability-check` field maps to `Option<String>` — `~` (null) in YAML produces `None`, meaning no runtime check is required.
2. **Graph edges** — `uses` (Feature → Dependency), `governs` (ADR → Dependency), and `supersedes` (Dependency → Dependency) edges are registered in the knowledge graph and traversed by `product impact`, `product preflight`, and gap analysis.
3. **Bill of materials** — `product dep bom` groups dependencies by type, sorts alphabetically within each group, and appends a summary line with total count and breaking-change-risk distribution.
4. **Availability checks** — `product dep check` and `product preflight FT-XXX` execute the `availability-check` shell command for each relevant dependency. Exit 0 from the command means satisfied; non-zero means not satisfied. Product never installs or starts dependencies.
5. **TC `requires` resolution** — a TC can list a DEP ID in its `requires` field; Product resolves it to that dependency's `availability-check` command, eliminating duplication between TC prerequisites and dependency declarations.
6. **Impact analysis** — `product impact DEP-XXX` performs reverse-graph BFS from the dependency node, returning all features and ADRs reachable through `uses` and `governs` edges.
7. **Context bundles** — when `product context FT-XXX` assembles a bundle, a "Dependencies" section is inserted after ADRs. It includes each linked DEP's body text, type, version, and interface block so the implementing agent has the full runtime contract.
8. **Scaffolding** — `product dep new "name" --type library` creates a `DEP-XXX-*.md` file and an `ADR-XXX` stub simultaneously, enforcing the convention that every dependency requires a governing decision.

### Invariants

- Every `DEP-XXX` artifact must have a governing ADR (`governs` edge from some ADR to the DEP); absence triggers E013 (hard error, exit 1) during `product graph check`.
- A dependency `type` must be one of the six defined values: `library`, `service`, `api`, `tool`, `hardware`, `runtime`.
- A dependency `status` must be one of: `active`, `evaluating`, `deprecated`, `migrating`.
- `product dep check` failures are warnings (exit 2), never hard errors — the implementing agent may continue without the dependency running, though TCs that `requires` it will be skipped.
- Availability-check commands are executed in the shell; Product does not interpret their output, only their exit code.

### Error handling

- **E013** — dependency has no linked ADR; emitted by `product graph check`; exit code 1.
- **W013** — feature uses a deprecated or migrating dependency; emitted by `product graph check`; exit code 2.
- **W015** — dependency availability-check failed during `product preflight`; exit code 2.
- **G008** — gap-analysis code: feature `uses` a DEP with no governing ADR (`governs` edge missing from any ADR); severity medium.
- Unknown dependency type or status strings produce `ProductError::ConfigError` during parsing.
- `product dep show` / `product dep features` on a non-existent DEP ID return `ProductError::NotFound` (exit 1).

### Boundaries

- DEP artifacts store runtime facts (version, interface, availability-check). The *decision* to use a dependency belongs in an ADR (`governs` edge). These two concerns are intentionally separated.
- Product never installs, starts, or configures dependencies — it only checks them.
- `product dep new` scaffolds files but does not commit them; committing is the developer's responsibility.
- The `breaking-change-risk` field is a human-declared annotation, not computed from any registry.

## Out of scope

- Package manager integration (Cargo.lock, package.json, Gemfile.lock) — Product reads hand-maintained DEP front-matter, not lock files.
- Automatic version-constraint resolution or dependency graph satisfiability checking.
- Dependency vulnerability scanning — `product dep bom --format json` produces a machine-readable bill of materials that security tools can consume, but Product does not run advisories itself.
- Dependency installation or provisioning — availability checks verify presence only.
- DEP-to-DEP dependency graphs beyond the `supersedes` edge used for migration tracking.
