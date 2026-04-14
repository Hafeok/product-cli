## Overview

Front-matter schema defines the YAML metadata format that every artifact file in a Product repository must carry. Each feature, ADR, and test criterion file declares its identity, relationships, and status in a structured YAML block at the top of the file. This front-matter is the sole source of truth for the knowledge graph — Product rebuilds the graph from these declarations on every invocation, with no persistent graph store (ADR-002). The schema is versioned, validated on parse, and migrateable across versions (ADR-014).

## Tutorial

### Step 1: Create your first feature

Run the following command to create a new feature file with valid front-matter:

```bash
product feature new --title "User Authentication"
```

Product assigns the next sequential ID automatically. Open the created file in `docs/features/`:

```yaml
---
id: FT-001
title: User Authentication
phase: 1
status: planned
depends-on: []
domains: []
adrs: []
tests: []
---
```

### Step 2: Link an ADR to the feature

Create an ADR and link it:

```bash
product adr new --title "JWT for Session Tokens"
```

Edit the feature file to add the ADR reference:

```yaml
adrs: [ADR-001]
```

Edit the ADR file to add the reverse link:

```yaml
features: [FT-001]
```

### Step 3: Validate the graph

Run the graph check to confirm your front-matter is well-formed and all links resolve:

```bash
product graph check
```

If you referenced a non-existent ID (e.g., `ADR-999`), the check reports the broken link and exits with code 1.

### Step 4: Add a test criterion

Create a test criterion file:

```bash
product test new --title "JWT Token Validation" --type scenario
```

The generated front-matter includes the `validates` block for linking back to features and ADRs:

```yaml
---
id: TC-001
title: JWT Token Validation
type: scenario
status: unimplemented
validates:
  features: [FT-001]
  adrs: [ADR-001]
phase: 1
runner: cargo-test
runner-args: "tc_001_jwt_token_validation"
---
```

### Step 5: Check schema version

Inspect `product.toml` to see your repository's schema version:

```toml
schema-version = "1"
```

Product validates this on every invocation. If your binary is newer than the schema, you will see a W007 warning suggesting an upgrade.

## How-to Guide

### Add a dependency between features

Edit the `depends-on` field in the dependent feature's front-matter:

1. Open the feature file (e.g., `docs/features/FT-002-api-layer.md`).
2. Add the dependency: `depends-on: [FT-001]`.
3. Run `product graph check` to confirm no cycles exist. Product enforces that `depends-on` edges form a DAG — cycles are a hard error.

### Declare concern domains

1. Add domains to the feature front-matter: `domains: [consensus, networking]`.
2. For domains with no linked ADR, add explicit reasoning:
   ```yaml
   domains-acknowledged:
     scheduling: >
       Out of scope for this phase.
   ```
3. Run `product graph check` to validate domain coverage.

### Mark an ADR as superseded

1. Open the superseded ADR file.
2. Set `status: superseded` and `superseded-by: [ADR-005]`.
3. Open the new ADR file and set `supersedes: [ADR-003]`.
4. Run `product graph check` to validate the supersession chain.

### Migrate to a new schema version

1. Check what would change: `product migrate schema --dry-run`.
2. Apply the migration: `product migrate schema`.
3. Re-run to confirm idempotency: `product migrate schema` (reports zero files changed).

Custom fields added to your front-matter are preserved through migration — Product never strips fields it does not understand.

### Add formal blocks to a test criterion

For `invariant` and `chaos` type TCs, formal blocks are mandatory. Add them after the front-matter and prose description:

```markdown
⟦Γ:Invariants⟧{
  ∀n:Node: connected(n) ∧ reachable(n, leader)
}
```

For `scenario` type TCs, formal blocks are optional:

```markdown
⟦Λ:Scenario⟧{
  given≜ cluster(3, "healthy")
  when≜  kill(leader)
  then≜  elected(new_leader) ∧ new_leader ≠ old_leader
}
```

Run `product graph check` to validate block syntax.

### Add source-file tracking to an ADR

1. Add the `source-files` field to the ADR front-matter:
   ```yaml
   source-files:
     - src/consensus/raft.rs
     - src/consensus/leader.rs
   ```
2. This enables `product drift check` to perform precise spec-vs-code analysis for that ADR.

## Reference

### Feature front-matter fields

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `id` | string | Yes | — | Unique identifier, format `FT-XXX` |
| `title` | string | Yes | — | Human-readable name |
| `phase` | integer | Yes | — | Implementation phase |
| `status` | enum | Yes | — | `planned`, `in-progress`, `complete`, `abandoned` |
| `depends-on` | list | No | `[]` | Feature IDs that must complete first (must form a DAG) |
| `domains` | list | No | `[]` | Concern domains this feature touches |
| `adrs` | list | No | `[]` | Linked ADR IDs |
| `tests` | list | No | `[]` | Linked TC IDs |
| `domains-acknowledged` | map | No | — | Explicit reasoning for domains with no linked ADR |

### ADR front-matter fields

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `id` | string | Yes | — | Unique identifier, format `ADR-XXX` |
| `title` | string | Yes | — | Human-readable name |
| `status` | enum | Yes | — | `proposed`, `accepted`, `superseded`, `abandoned` |
| `features` | list | No | `[]` | Linked feature IDs |
| `supersedes` | list | No | `[]` | ADR IDs this decision replaces |
| `superseded-by` | list | No | `[]` | ADR IDs that replace this decision |
| `domains` | list | No | `[]` | Concern domains this ADR governs |
| `scope` | enum | No | `feature-specific` | `cross-cutting`, `domain`, `feature-specific` |
| `source-files` | list | No | — | Source files implementing this decision (for drift detection) |

### Test criterion front-matter fields

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `id` | string | Yes | — | Unique identifier, format `TC-XXX` |
| `title` | string | Yes | — | Human-readable name |
| `type` | enum | Yes | — | `scenario`, `invariant`, `chaos`, `exit-criteria`, `benchmark` |
| `status` | enum | Yes | — | `unimplemented`, `implemented`, `passing`, `failing` |
| `validates.features` | list | No | `[]` | Feature IDs this TC validates |
| `validates.adrs` | list | No | `[]` | ADR IDs this TC validates |
| `phase` | integer | No | — | Implementation phase |
| `runner` | string | No | — | Test runner: `cargo-test`, `bash`, `pytest`, `custom` |
| `runner-args` | string | No | — | Arguments passed to the runner |
| `runner-timeout` | duration | No | `30s` | Maximum execution time |

### Formal block types

| Block | Syntax | Required for | Description |
|-------|--------|--------------|-------------|
| Types | `⟦Σ:Types⟧{...}` | — | Type definitions using `≜` |
| Invariants | `⟦Γ:Invariants⟧{...}` | `invariant`, `chaos` | Universal/existential properties |
| Scenario | `⟦Λ:Scenario⟧{...}` | — | Given/when/then fields |
| Exit Criteria | `⟦Λ:ExitCriteria⟧{...}` | — | Measurable thresholds |
| Benchmark | `⟦Λ:Benchmark⟧{...}` | `benchmark` | Baseline, target, scorer config |
| Evidence | `⟦Ε⟧⟨...⟩` | — | Confidence (δ), coverage (φ), stability (τ) |

### Evidence block values

| Field | Range | Meaning |
|-------|-------|---------|
| `δ` | 0.0–1.0 | Confidence level |
| `φ` | 0–100 | Coverage percentage |
| `τ` | `◊⁺` (stable), `◊⁻` (unstable), `◊?` (unknown) | Stability indicator |

### ID assignment rules

- IDs are assigned sequentially: `FT-001`, `FT-002`, `FT-003`.
- Next ID is always `max(existing) + 1` — gaps are never filled.
- IDs are permanent. Retired artifacts use `status: abandoned`, never deletion or renumbering.
- The prefix (`FT`, `ADR`, `TC`) is configurable in `product.toml`.

### Schema version in `product.toml`

```toml
schema-version = "1"
```

- Integer, incremented only on breaking changes.
- Adding an optional field with a default does not increment the version.

### Relevant error and warning codes

| Code | Meaning |
|------|---------|
| E001 | Parse error — malformed front-matter or formal block, with line-level precision |
| E008 | Schema version mismatch — binary too old for repository schema |
| E009 | Atomic write failure — temp file could not be written or renamed |
| E010 | Repository locked — another Product process holds `.product.lock` |
| W004 | Empty formal block body — syntactically valid but semantically meaningless |
| W007 | Schema upgrade available — repository on older version than binary supports |

## Explanation

### Why YAML front-matter instead of a separate graph file?

Product uses YAML front-matter as the sole source of truth for the knowledge graph (ADR-002). Every artifact file is self-describing — open any file and you immediately see its ID, status, and all outgoing edges. The graph is recomputed from front-matter on every CLI invocation; there is no persistent graph store. This eliminates the synchronisation problem that plagues systems with separate graph files: contributors update the document but forget the graph file, and the two diverge silently.

### Why numeric IDs instead of slugs or UUIDs?

ADR-005 chose prefixed zero-padded numeric IDs (`FT-001`, `ADR-002`) over alternatives. Slugs like `cluster-foundation` break if the title changes. UUIDs are collision-free but unreadable — `FT-001` in a commit message carries meaning, a UUID does not. Sequential numbering follows established convention (JIRA, RFC numbering) and ensures correct alphabetical sort in file listings.

### How schema migration works

Schema versioning (ADR-014) uses a single integer in `product.toml`. Breaking changes increment the version. `product migrate schema` applies migration steps sequentially (1 to 2, 2 to 3, etc.) using atomic writes (ADR-015) so that a crash mid-migration cannot corrupt files. Unknown front-matter fields are preserved through migration — Product never strips fields it does not understand, which allows external tooling to add custom fields without fear of data loss.

Forward incompatibility (binary too old for schema) is a hard error because running old Product against a new schema could produce silently wrong graph output. Backward incompatibility (binary newer than schema) is a warning because the old schema is still fully readable.

### How formal blocks are parsed

ADR-016 specifies a hand-written recursive descent parser for the AISP-influenced formal block notation. The parser produces a typed AST for validation while storing the original text verbatim for context bundle output. This dual representation ensures that `product context` reproduces exactly what the author wrote, without round-trip formatting changes. The grammar is intentionally permissive on expressions — structural validation is Product's job, not full semantic verification.

### File write safety

All file mutations use atomic temp-file-plus-rename (ADR-015). An advisory lock on `.product.lock` serialises concurrent Product invocations with a 3-second timeout. Stale locks from crashed processes are detected and automatically acquired. This protects against torn writes (partial file content from interrupted commands) and concurrent write conflicts (two Product processes silently overwriting each other).
