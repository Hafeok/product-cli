---
id: FT-003
title: Front-Matter Schema
phase: 1
status: complete
depends-on: []
adrs:
- ADR-002
- ADR-014
- ADR-016
tests:
- TC-005
- TC-006
- TC-007
- TC-008
- TC-060
- TC-061
- TC-062
- TC-063
- TC-064
- TC-065
- TC-071
- TC-072
- TC-073
- TC-074
- TC-075
- TC-076
- TC-077
- TC-078
- TC-079
- TC-155
domains:
- data-model
domains-acknowledged:
  ADR-048: Predates ADR-048; this feature does not author the canonical .product/ folder layout. FT-057 owns the migration command and discovery fallback.
  ADR-041: Pre-dates ADR-041; this feature does not author absence TCs or set removes/deprecates on ADRs. FT-047 owns the mechanics.
  ADR-040: predates ADR-040; feature ships independently of the unified verify / LLM-boundary pipeline
  ADR-042: Pre-dates ADR-042; this feature does not define TC types or validate the type vocabulary. FT-048 owns the mechanics.
  ADR-043: Predates ADR-043; feature command adapters were written before the slice+adapter pattern was formalised and are not retroactively refactored.
  ADR-018: Predates the 2026-04-22 scope promotion of ADR-018 to cross-cutting. Test coverage reflects the property/session/benchmark strategy as it existed when this feature shipped; not retroactively reclassified.
  ADR-047: Predates ADR-047; this feature does not author the functional-spec body convention. FT-055 owns the structural validator and W030 mechanics.
---

### Feature

```yaml
---
id: FT-001
title: Cluster Foundation
phase: 1
status: in-progress          # planned | in-progress | complete | abandoned
depends-on: []               # feature IDs that must be complete before this one
domains: [consensus, networking, storage, iam, observability]
                             # concern domains this feature touches
adrs: [ADR-001, ADR-002, ADR-003, ADR-006]
tests: [TC-001, TC-002, TC-003, TC-004]
domains-acknowledged:        # explicit reasoning for domains with no linked ADR
  scheduling: >
    No workload scheduling in phase 1. Cluster foundation does not
    place containers — that is phase 2. Intentionally out of scope.
---
```

The `depends-on` field declares implementation dependencies between features. Product validates that these edges form a DAG — cycles are a hard error. `product feature next` uses topological sort over this DAG to determine the correct implementation order, replacing the previous phase-label ordering.

### ADR

```yaml
---
id: ADR-002
title: openraft for Cluster Consensus
status: accepted             # proposed | accepted | superseded | abandoned
features: [FT-001]
supersedes: []
superseded-by: []
domains: [consensus, networking]   # concern domains this ADR governs
scope: domain               # cross-cutting | domain | feature-specific (default)
source-files:                # optional: source files that implement this decision
  - src/consensus/raft.rs    # used by `product drift check` for precise analysis
  - src/consensus/leader.rs  # if absent, Product uses pattern-based discovery
---
```

### Test Criterion

Test criterion files use a hybrid format. The YAML front-matter carries graph metadata. The file body contains a prose description followed by optional AISP-influenced formal blocks (see ADR-011).

**Types and formal block requirements:**

| Type | Description | Formal blocks |
|---|---|---|
| `scenario` | Given/when/then integration test | Optional (`⟦Λ:Scenario⟧`) |
| `invariant` | Property that must hold for all valid inputs | Mandatory (`⟦Γ:Invariants⟧`) |
| `chaos` | System behaviour under fault injection | Mandatory (`⟦Γ:Invariants⟧`) |
| `exit-criteria` | Measurable threshold for phase completion | Optional (`⟦Λ:ExitCriteria⟧`) |
| `benchmark` | Quality measurement producing a score over time | Mandatory (`⟦Λ:Benchmark⟧`) |

The `benchmark` type is distinct from the others: it does not produce a binary pass/fail result. It produces a score in [0.0, 1.0] tracked over releases. A benchmark test criterion references an external task directory and rubric file rather than expressing an inline assertion.

**Scenario example:**
```markdown
---
id: TC-002
title: Raft Leader Election
type: scenario
status: unimplemented        # unimplemented | implemented | passing | failing
validates:
  features: [FT-001]
  adrs: [ADR-002]
phase: 1
runner: cargo-test           # cargo-test | bash | pytest | custom
                             # omit if test infrastructure not yet available
runner-args: ["--test", "raft_leader_election", "--", "--nocapture"]
runner-timeout: 60s          # optional, default 30s
---

```

---

## Description

FT-003 defines the complete YAML front-matter schema for each of the three artifact types: Feature (`FT-XXX`), Architectural Decision Record (`ADR-XXX`), and Test Criterion (`TC-XXX`). Front-matter is the sole source of truth for artifact identity and graph relationships (ADR-002). The schema specifies which fields are required, which are optional with defaults, and what values are permitted in enumerated fields such as `status` and `type`. The `schema-version` field in `product.toml` governs schema evolution (ADR-014). Unknown fields in artifact files are preserved on write and never stripped, ensuring that tooling layered on top of Product can add custom fields without loss. Test criterion files additionally support a hybrid format where the file body after the front-matter contains an optional AISP-influenced formal block section (ADR-011, ADR-016).

## Functional Specification

### Inputs

- Each `.md` artifact file in the configured directories, containing a YAML front-matter block delimited by `---` lines.
- The `schema-version` value from `product.toml`, used to validate forward/backward compatibility (ADR-014).
- For test criterion files: optional formal block sections in the file body (ADR-011).

### Outputs

- Typed Rust structs (`FeatureFrontMatter`, `AdrFrontMatter`, `TcFrontMatter`) populated from the deserialized YAML. These structs are the in-memory representation used by all commands.
- Validation diagnostics (E001, E006, E007, E008, W004, W007) emitted to stderr when fields are malformed, missing, or incompatible with the current schema version.
- Preserved round-trip output: when Product writes a file it modifies, all front-matter fields — including those unknown to the current schema — are preserved verbatim (ADR-014).

### State

Stateless. The schema is compiled into the binary. No external schema registry is consulted. `product.toml`'s `schema-version` is the only runtime schema state; it is read on every invocation and compared against the binary's supported range.

### Behaviour

1. **Feature front-matter**: required fields are `id` (prefix `FT-`, zero-padded numeric), `title` (string), `phase` (positive integer), `status` (`planned | in-progress | complete | abandoned`). Optional fields include `depends-on` (list of Feature IDs, forms a DAG validated by the graph engine), `adrs` (list of ADR IDs), `tests` (list of TC IDs), `domains` (list of concern domain strings declared in `product.toml`), and `domains-acknowledged` (map of domain or ADR key to reasoning string).
2. **ADR front-matter**: required fields are `id` (prefix `ADR-`), `title`, `status` (`proposed | accepted | superseded | abandoned`). Optional fields include `features` (list of Feature IDs), `supersedes` and `superseded-by` (lists of ADR IDs forming a supersession DAG validated for cycles), `domains`, `scope` (`cross-cutting | domain | feature-specific`), and `source-files` (list of source paths used by `product drift check`).
3. **Test Criterion front-matter**: required fields are `id` (prefix `TC-` or sub-namespace `TC-CQ-`, `TC-PLT-`), `title`, `type` (`scenario | invariant | chaos | exit-criteria | benchmark`), `status` (`unimplemented | implemented | passing | failing`), and `validates` (map with `features` and `adrs` lists). Optional fields include `runner` (`cargo-test | bash | pytest | custom`), `runner-args` (string matching the test function name), `runner-timeout`, and `benchmark` (sub-map used only for `type: benchmark`).
4. **Formal blocks in TC files** (ADR-011, ADR-016): after the front-matter, a TC file body may contain zero or more formal blocks (`⟦Σ:Types⟧`, `⟦Γ:Invariants⟧`, `⟦Λ:Scenario⟧`, `⟦Λ:ExitCriteria⟧`, `⟦Λ:Benchmark⟧`, `⟦Ε⟧`). `invariant` and `chaos` type TCs are expected to include at least `⟦Γ:Invariants⟧`; absence is warned as W004. Formal blocks are parsed by the hand-written recursive descent parser defined in ADR-016.
5. **Schema version compatibility**: on startup Product reads `schema-version` from `product.toml`. If the file version exceeds the binary's supported maximum, Product exits with E008. If the file version is below the binary's current version, Product emits W007 and continues; `product migrate schema` upgrades in place (ADR-014).
6. **Unknown fields**: fields not in the schema are deserialized into a catch-all map and rewritten verbatim on file update. Product never strips fields it does not recognise.

### Invariants

- `id` fields must match the pattern `[A-Z]+-\d{3,}` (as enforced by `parser::validate_id`). Files with invalid IDs are rejected with E005.
- `status` values are validated against the closed enum for each artifact type; an unrecognised value yields E007.
- The `depends-on` list in Feature front-matter must not introduce a cycle in the feature dependency DAG; cycles are reported as E003.
- The `supersedes`/`superseded-by` lists in ADR front-matter must not introduce a cycle; cycles are reported as E004.
- Every domain string declared in Feature or ADR front-matter must appear in the `[domains]` table of `product.toml`; unknown domains are reported as E012.
- `domains-acknowledged` entries with an empty or missing reasoning string are reported as E011.
- Evidence block fields: `δ` must be in [0.0, 1.0] and `φ` must be in [0, 100]; values outside these ranges are E001.

### Error handling

- Malformed YAML front-matter → E001 with file path and line number from `serde_yaml`'s location info; the file is skipped and parsing of remaining files continues (ADR-013).
- Missing required `id` field → E006 with field name and file path.
- Invalid ID format → E005 with file path and the offending value.
- Unknown `status` or `type` value → E007 with the offending value and the accepted vocabulary.
- Schema version in `product.toml` exceeds binary support → E008, hard error on startup.
- Schema version below binary's current version → W007, command continues; `product migrate schema` resolves it.
- `domains-acknowledged` entry with empty reasoning → E011.
- Domain declared in front-matter absent from `product.toml` vocabulary → E012.
- Formal block with unclosed delimiter or invalid expression → E001 at the offending line; subsequent blocks in the same file are still parsed.
- Empty formal block body → W004.

### Boundaries

- The schema covers only the three artifact types managed by Product. Free-form markdown files in the same directories that lack recognisable front-matter are ignored.
- The formal block grammar (ADR-016) is the exclusive concern of the `formal` module in `src/formal/`; front-matter parsing and formal block parsing are separate parse passes over the same file.
- The `benchmark` TC sub-map (`task`, `rubric`, `conditions`, `runs-per-condition`, `pass-threshold`) is validated for presence of required keys but its content is otherwise opaque to the graph engine — it is consumed by the external benchmark runner, not by Product.

## Out of scope

- Schema definition for `product.toml` itself (beyond the `schema-version` field) — that is configuration parsing, not artifact front-matter schema.
- The `runner` and `runner-args` execution mechanics — those are covered by the verify pipeline (FT-021 / `product verify`).
- Authoring commands that scaffold new artifact files with correct front-matter defaults — covered by FT-004.
- In-place schema migration of existing front-matter when the schema version increments — covered by FT-008.
