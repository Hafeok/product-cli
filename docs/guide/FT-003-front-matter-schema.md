The write permission was denied twice. Here is the complete documentation for FT-003 — Front-Matter Schema. You can save it to `docs/guide/FT-003-front-matter-schema.md`:

---

## Overview

Front-Matter Schema defines the YAML front-matter structure that every artifact file (feature, ADR, test criterion) must carry. This front-matter is the sole source of truth for the knowledge graph — Product rebuilds the graph from front-matter on every invocation rather than maintaining a persistent store (ADR-002). The schema is versioned through `product.toml`, validated on parse, and migrateable across versions (ADR-014). Artifact identifiers follow a prefixed numeric scheme (`FT-XXX`, `ADR-XXX`, `TC-XXX`) that is human-readable, sortable, and permanent (ADR-005).

## Tutorial

### Writing your first feature file

1. Create a feature file using the CLI:

   ```bash
   product feature new --title "My First Feature"
   ```

   Product assigns the next sequential ID (e.g., `FT-001`) and writes the file to your configured features directory.

2. Open the generated file. It contains front-matter like this:

   ```yaml
   ---
   id: FT-001
   title: My First Feature
   phase: 1
   status: planned
   depends-on: []
   adrs: []
   tests: []
   domains: []
   domains-acknowledged: {}
   ---
   ```

3. Link the feature to an ADR and a test criterion by editing the front-matter:

   ```yaml
   adrs: [ADR-001]
   tests: [TC-001]
   ```

4. Validate that the front-matter is well-formed and all links resolve:

   ```bash
   product graph check
   ```

   If `ADR-001` or `TC-001` does not exist, the command reports the broken link and exits with code 1.

### Writing your first ADR file

1. Create an ADR:

   ```bash
   product adr new --title "Use PostgreSQL for Storage"
   ```

2. The generated front-matter looks like:

   ```yaml
   ---
   id: ADR-001
   title: Use PostgreSQL for Storage
   status: proposed
   features: [FT-001]
   supersedes: []
   superseded-by: []
   domains: []
   scope: feature-specific
   ---
   ```

3. Set the status to `accepted` once the decision is ratified, and add the concern domains it governs:

   ```yaml
   status: accepted
   domains: [storage]
   scope: domain
   ```

### Writing your first test criterion

1. Create a test criterion:

   ```bash
   product test new --title "Database Connection" --type scenario
   ```

2. The generated front-matter:

   ```yaml
   ---
   id: TC-001
   title: Database Connection
   type: scenario
   status: unimplemented
   validates:
     features: [FT-001]
     adrs: [ADR-001]
   phase: 1
   ---
   ```

3. Once you write the integration test, add runner configuration so `product verify` can execute it:

   ```yaml
   runner: cargo-test
   runner-args: "tc_001_database_connection"
   ```

4. Run verification:

   ```bash
   product verify FT-001
   ```

## How-to Guide

### Link a feature to ADRs and tests

1. Open the feature file and add IDs to the `adrs` and `tests` arrays:

   ```yaml
   adrs: [ADR-001, ADR-002]
   tests: [TC-001, TC-002]
   ```

2. Run `product graph check` to validate all links resolve.

### Declare feature dependencies

1. Add the prerequisite feature IDs to `depends-on`:

   ```yaml
   depends-on: [FT-001, FT-002]
   ```

2. Product validates that dependency edges form a DAG. Cycles produce a hard error on `product graph check`.

3. Use `product feature next` to see the correct implementation order based on topological sort over the dependency graph.

### Supersede an ADR

1. In the new ADR's front-matter, reference the old decision:

   ```yaml
   supersedes: [ADR-001]
   ```

2. In the old ADR's front-matter, add the back-reference:

   ```yaml
   superseded-by: [ADR-002]
   ```

3. Set the old ADR's status to `superseded`.

### Check and upgrade the schema version

1. Check the current schema version in `product.toml`:

   ```toml
   schema-version = "1"
   ```

2. Preview what a migration would change:

   ```bash
   product migrate schema --dry-run
   ```

3. Run the migration:

   ```bash
   product migrate schema
   ```

4. Re-running the migration is safe — it is idempotent and reports zero files changed on the second run.

### Add custom fields without breaking Product

1. Add any field to an artifact's front-matter:

   ```yaml
   custom-tag: my-value
   ```

2. Product ignores fields it does not recognise and preserves them on write. Custom fields survive migrations and CLI-driven edits.

## Reference

### Feature front-matter fields

| Field | Type | Required | Default | Description |
|---|---|---|---|---|
| `id` | `string` | Yes | — | Unique identifier (`FT-NNN`) |
| `title` | `string` | Yes | — | Human-readable name |
| `phase` | `integer` | No | `1` | Implementation phase |
| `status` | `string` | No | `planned` | One of: `planned`, `in-progress`, `complete`, `abandoned` |
| `depends-on` | `string[]` | No | `[]` | Feature IDs that must be complete first |
| `adrs` | `string[]` | No | `[]` | Linked ADR IDs |
| `tests` | `string[]` | No | `[]` | Linked TC IDs |
| `domains` | `string[]` | No | `[]` | Concern domains this feature touches |
| `domains-acknowledged` | `map<string, string>` | No | `{}` | Domains with no linked ADR, with reasoning |

### ADR front-matter fields

| Field | Type | Required | Default | Description |
|---|---|---|---|---|
| `id` | `string` | Yes | — | Unique identifier (`ADR-NNN`) |
| `title` | `string` | Yes | — | Human-readable name |
| `status` | `string` | No | `proposed` | One of: `proposed`, `accepted`, `superseded`, `abandoned` |
| `features` | `string[]` | No | `[]` | Features this ADR implements |
| `supersedes` | `string[]` | No | `[]` | ADR IDs this decision replaces |
| `superseded-by` | `string[]` | No | `[]` | ADR IDs that replace this decision |
| `domains` | `string[]` | No | `[]` | Concern domains this ADR governs |
| `scope` | `string` | No | `feature-specific` | One of: `cross-cutting`, `domain`, `feature-specific` |

### Test criterion front-matter fields

| Field | Type | Required | Default | Description |
|---|---|---|---|---|
| `id` | `string` | Yes | — | Unique identifier (`TC-NNN`) |
| `title` | `string` | Yes | — | Human-readable name |
| `type` | `string` | No | `scenario` | One of: `scenario`, `invariant`, `chaos`, `exit-criteria` |
| `status` | `string` | No | `unimplemented` | One of: `unimplemented`, `implemented`, `passing`, `failing` |
| `validates.features` | `string[]` | No | `[]` | Feature IDs this test validates |
| `validates.adrs` | `string[]` | No | `[]` | ADR IDs this test validates |
| `phase` | `integer` | No | `1` | Implementation phase |
| `runner` | `string` | No | — | Test runner: `cargo-test`, `bash`, `pytest`, `custom` |
| `runner-args` | `string` | No | — | Arguments passed to the runner |
| `runner-timeout` | `string` | No | `30s` | Execution timeout (e.g., `60s`, `5min`) |

### ID format

IDs follow the pattern `PREFIX-NNN`:
- Prefix is configurable in `product.toml` under `[prefixes]` (defaults: `FT`, `ADR`, `TC`)
- Numeric part is zero-padded to at least 3 digits
- IDs are assigned sequentially — `max(existing) + 1`; gaps are never filled
- IDs are permanent; retired artifacts are marked `status: abandoned`, never renumbered

### Schema version in `product.toml`

```toml
schema-version = "1"
schema-version-warning = true   # set to false to suppress W007
```

- Integer, incremented only on breaking changes (field renames, removed fields, changed semantics)
- Adding an optional field with a default is not a breaking change

### Error codes

| Code | Condition |
|---|---|
| E001 | Parse error: malformed YAML, invalid field value, formal block syntax error |
| E008 | Schema version mismatch: repository requires a newer schema than the binary supports |
| E010 | Repository locked: another Product process holds the advisory lock |
| W004 | Empty formal block body (syntactically valid, semantically meaningless) |
| W007 | Schema upgrade available but not applied |

### `product migrate schema` command

```
product migrate schema                # upgrade to current schema version
product migrate schema --dry-run      # show what would change without writing
product migrate schema --from 1       # explicit source version
```

Migrations are idempotent. Files are written atomically (ADR-015). `schema-version` in `product.toml` is updated last. If a write fails mid-migration, re-running the command completes the upgrade.

## Explanation

### Why YAML front-matter instead of a separate graph file

Product uses YAML front-matter as the sole source of truth for graph relationships (ADR-002). Each file is self-describing — open any artifact and you immediately see its identity, status, and connections. This eliminates the synchronisation problem where a separate graph file drifts from the documents it describes. Git diffs on front-matter changes are clean one-line edits, making code review straightforward.

The trade-off is that the graph must be recomputed on every invocation. For the expected scale (hundreds of artifacts, not millions), this is negligible — the parser reads all `.md` files in the configured directories, deserialises the YAML, and builds the in-memory graph in milliseconds.

### Why prefixed numeric IDs

The `FT-001` / `ADR-001` / `TC-001` scheme (ADR-005) is chosen for readability and stability. Sequential numbers are a convention engineers arrive with from JIRA, RFC numbering, and ADR numbering. The prefix makes artifact type visible in any context — commit messages, Slack threads, code comments. Zero-padding ensures correct alphabetical sort in file listings.

IDs are permanent. Once `FT-007` is assigned, that number is never reused, even if the feature is abandoned. This guarantees external references (code comments, commit messages) remain valid indefinitely. The `next_id` function always assigns `max(existing) + 1`, never filling gaps.

### Schema versioning strategy

Schema evolution is managed through a single integer version in `product.toml` (ADR-014). The version increments only on breaking changes. Product enforces strict forward incompatibility — running an old binary against a newer schema is a hard error (E008), because silently misinterpreting fields would corrupt the graph. Backward incompatibility is a warning (W007), since the old schema remains readable.

Unknown front-matter fields are preserved on write. This is critical for extensibility: teams can add custom fields without Product stripping them on the next CLI invocation.

### File write safety

All front-matter writes use atomic temp-file-plus-rename (ADR-015). An advisory lock on `.product.lock` serialises concurrent Product invocations with a 3-second timeout. This prevents torn writes from interrupted processes and silent data loss from concurrent modifications. Read-only commands never acquire the lock.

### Formal blocks in test criteria

Test criterion files can include AISP-influenced formal blocks after the prose body (ADR-016). The parser produces a typed AST for validation while preserving the raw text byte-for-byte for context bundle output. This dual representation means Product can validate evidence block ranges and detect empty blocks (W004) without altering the author's original notation.
