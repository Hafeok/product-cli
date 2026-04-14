## Overview

Schema Migration provides in-place upgrades for YAML front-matter when the Product schema version changes. As a knowledge graph tool evolves, its front-matter schema — the fields, their names, and their semantics — must evolve too. Without a migration path, developers would need to hand-edit every artifact file after a schema change, or risk silent data corruption from version mismatches. The `product migrate schema` command applies sequential migration functions to bring all artifact files from an older schema version to the current one, preserving unknown fields and ensuring idempotent, atomic writes.

## Tutorial

### Understanding schema versions

Product tracks schema versions as integers in `product.toml`. Open your project's config file:

```toml
name = "my-project"
schema-version = "1"
```

The `schema-version` field declares which front-matter schema your repository uses. When Product's binary supports a newer schema, it tells you.

### Step 1: See the upgrade warning

If your repository is on an older schema version than the running binary supports, every command emits a warning:

```
warning[W007]: schema upgrade available
  schema version 1 is supported but version 2 is current
  run `product migrate schema` to upgrade (dry-run with --dry-run)
```

This warning is informational — all commands continue to work normally.

### Step 2: Preview the migration

Before changing any files, run a dry run to see what would happen:

```bash
product migrate schema --dry-run
```

This scans every artifact file and reports which files would be modified and what changes would be applied. No files are written.

### Step 3: Run the migration

When you are satisfied with the dry-run output, execute the migration:

```bash
product migrate schema --execute
```

Product applies each migration step in sequence (e.g., v1 to v2, then v2 to v3), writes updated files atomically, and bumps `schema-version` in `product.toml` as the final step.

### Step 4: Verify the result

Run the migration command again to confirm idempotency:

```bash
product migrate schema --execute
```

The second run should report zero files changed. You can also run `product graph check` to verify the graph is healthy after migration.

## How-to Guide

### Preview changes before migrating

1. Run `product migrate schema --dry-run`.
2. Review the output for each file that would be modified.
3. No files are written in dry-run mode.

### Upgrade a repository to the latest schema

1. Run `product migrate schema --dry-run` to preview changes.
2. Run `product migrate schema --execute` to apply the migration.
3. Run `product migrate schema --execute` again to confirm zero files changed (idempotency).
4. Commit the updated files and `product.toml`.

### Migrate from a specific schema version

If you need to specify the source version explicitly rather than reading it from `product.toml`:

```bash
product migrate schema --from 1
```

This applies migrations starting from version 1, regardless of what `product.toml` currently declares.

### Suppress the upgrade warning

If your repository intentionally stays on an older schema, add to `product.toml`:

```toml
schema-version-warning = false
```

This suppresses the W007 warning on every command invocation.

### Recover from a partial migration failure

If a file write fails mid-migration:

1. Product leaves `schema-version` in `product.toml` unchanged.
2. The output reports which files were updated and which were not.
3. Re-run `product migrate schema --execute` — the command is idempotent, so already-migrated files are skipped.

### Preserve custom front-matter fields during migration

No action is needed. Product preserves any front-matter fields it does not recognise. If you have added custom fields like `custom-tag: foo` to your artifact files, they survive migration intact.

## Reference

### CLI syntax

```
product migrate schema [OPTIONS]
```

| Flag / Option | Description |
|---|---|
| `--dry-run` | Report what would change without writing any files. |
| `--execute` | Apply the migration and write updated files. |
| `--from <VERSION>` | Explicit source schema version (integer). Defaults to the value in `product.toml`. |

### Configuration keys (`product.toml`)

| Key | Type | Description |
|---|---|---|
| `schema-version` | String (integer) | Current schema version of the repository. Updated by `product migrate schema` on success. |
| `schema-version-warning` | Boolean | Set to `false` to suppress W007 warnings. Defaults to `true`. |

### Error and warning codes

| Code | Type | Condition |
|---|---|---|
| **E008** | Error | Forward incompatibility — the repository's schema version is newer than the binary supports. Product exits immediately. |
| **W007** | Warning | Backward compatibility — the repository's schema version is older than the binary's current version. An upgrade is available. |
| **E010** | Error | Advisory lock conflict — another `product migrate schema` process is already running. |

### E008 output format

```
error[E008]: schema version mismatch
  --> product.toml
   |
 2 | schema-version = "2"
   |                  ^^^ this repository requires schema version 2
   |                      this binary supports up to schema version 1
   |
   = hint: upgrade product with `cargo install product --force`
```

### W007 output format

```
warning[W007]: schema upgrade available
  schema version 1 is supported but version 2 is current
  run `product migrate schema` to upgrade (dry-run with --dry-run)
```

### Migration execution order

1. Read `schema-version` from `product.toml` (or `--from` flag).
2. Apply each migration function in sequence (v0 to v1, v1 to v2, etc.).
3. Write each updated artifact file atomically (temp file + rename).
4. Update `schema-version` in `product.toml` last.
5. Print summary: N files updated, M files unchanged.

### Idempotency guarantee

Running `product migrate schema` on a repository already at the current schema version reports zero files changed and exits successfully.

### Concurrency safety

Advisory file locking prevents concurrent migration commands. If a second `product migrate schema` is invoked while one is already running, the second process exits with E010.

## Explanation

### Why integer versions instead of semver

Schema compatibility for YAML front-matter is binary: a field either exists with the expected semantics, or it does not. The patch/minor/major distinction of semver does not apply. An integer that increments only on breaking changes is simpler to reason about and compare. Non-breaking changes — such as adding an optional field with a default value — do not increment the version at all (ADR-014).

### What counts as a breaking change

A schema version increment is required when:

- A front-matter field is renamed.
- A front-matter field is removed.
- The semantics of a field change (e.g., a string field becomes a list).

Adding a new optional field with a documented default is **not** a breaking change.

### Forward incompatibility is a hard error

If the repository declares a schema version newer than the binary understands, Product exits with E008 rather than attempting to continue. Running old Product against new schema would produce silently wrong graph output — missing edges, incorrect status values. A hard error is the only safe response (ADR-014).

### Unknown fields are preserved, not stripped

Product never removes front-matter fields it does not recognise. This is critical for extensibility: teams can add custom metadata fields to their artifact files without Product destroying them on the next write operation. Migration functions transform known fields and pass unknown fields through unchanged (ADR-002, ADR-014).

### Atomic writes and advisory locking

Each file is written atomically using a temp-file-plus-rename strategy. `product.toml` is updated last, so a crash mid-migration leaves the schema version unchanged. Re-running the command picks up where it left off. Advisory locking (E010) prevents two migration processes from racing on the same repository.

### Migration functions are permanent

Every migration function (e.g., `migrate_v0_to_v1`) is kept in the codebase permanently. This ensures a repository at any historical schema version can be upgraded to the current version in a single `product migrate schema` invocation, applying each step in sequence.

### Relationship to the knowledge graph

The schema version governs the structure of YAML front-matter, which is the sole source of truth for the knowledge graph (ADR-002). A schema migration therefore updates the raw material from which the graph is derived. After migration, running `product graph check` validates that the upgraded front-matter produces a healthy graph.
