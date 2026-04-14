---
id: ADR-014
title: Schema Versioning and Migration Path
status: accepted
features: []
supersedes: []
superseded-by: []
domains: []
scope: domain
content-hash: sha256:8201ece69d1f1346782b20d7c29d3ef7cd11fa0817149be5e5af111f17a43897
---

**Status:** Accepted

**Context:** Product's front-matter schema will evolve. Fields will be added, renamed, or have their semantics clarified. A repository created with Product v0.1 may contain front-matter that Product v0.2 reads differently — silently producing wrong results — or refuses to read at all, hard-erroring on every command. Both outcomes are unacceptable for a tool that manages long-lived project artifacts.

The schema version must be machine-readable, forward-compatible by default, and upgradeable without requiring the developer to manually edit every artifact file.

**Decision:** `product.toml` carries a `schema-version` field. Product validates this on startup against its own supported schema range. Front-matter fields unknown to the current schema version are ignored with a warning (forward compatibility). Fields present in the schema but absent in a file are filled with documented defaults (backward compatibility). `product migrate schema` performs in-place upgrades when a breaking change is introduced.

---

### Schema Version in `product.toml`

```toml
name = "picloud"
schema-version = "1"          # integer, incremented on breaking changes
```

Schema version is an integer, not semver. It increments only on breaking changes — field renames, removed fields, changed semantics. Adding an optional field with a default is not a breaking change and does not increment the version.

---

### Compatibility Rules

**Forward compatibility (Product older than schema):** If `product.toml` declares `schema-version = "2"` and the running binary only supports up to version `"1"`, Product exits with error E008:

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

**Backward compatibility (Product newer than schema):** If `product.toml` declares `schema-version = "1"` and the binary supports version `"2"`, Product runs normally but emits W007 on startup:

```
warning[W007]: schema upgrade available
  schema version 1 is supported but version 2 is current
  run `product migrate schema` to upgrade (dry-run with --dry-run)
```

This warning is suppressible with `schema-version-warning = false` in `product.toml` for repositories that have made an explicit decision to stay on an older schema.

**Unknown front-matter fields:** Fields in artifact files not recognised by the current schema are silently ignored. They are preserved on write — Product never strips fields it does not understand. This ensures that tooling built on top of Product can add custom fields to front-matter without Product destroying them.

---

### `product migrate schema` Command

```
product migrate schema              # upgrade to current schema version
product migrate schema --dry-run    # show what would change without writing
product migrate schema --from 1     # explicit source version (defaults to product.toml value)
```

The migrate command:
1. Reads `product.toml` schema version
2. Applies each migration step in sequence (1→2, 2→3, etc.)
3. Writes updated artifact files atomically (temp file + rename, see ADR-015)
4. Updates `schema-version` in `product.toml` last
5. Reports a summary: N files updated, M files unchanged

If any file write fails mid-migration, the command reports the failure and leaves `schema-version` in `product.toml` unchanged. The partially migrated files remain — they are individually valid for the new schema — but the operator is told which files were updated and which were not. Re-running `product migrate schema` is idempotent.

---

### Breaking Change Protocol

When a schema change is introduced:

1. Increment `schema-version` in the Product source
2. Write a migration function `migrate_v1_to_v2()` that transforms affected front-matter fields
3. Document the change in `CHANGELOG.md` with before/after examples
4. Add a scenario test that runs a v1 repository through the migration and asserts the v2 output
5. Keep the migration function permanently — it must be possible to upgrade from any historical version to current in one command

---

**Rationale:**
- Integer schema version is simpler than semver for this use case. Schema compatibility is binary: a field either exists and has the expected semantics, or it doesn't. Patch and minor version distinctions don't apply.
- Forward incompatibility is a hard error, not a warning. Running a new schema repository through old Product would produce silently wrong graph output — missing edges, incorrect status values. Hard error is the only safe response.
- Backward incompatibility is a warning, not an error. The old schema is still readable; it's just missing new capabilities. The developer can choose when to migrate.
- Preserving unknown fields on write is critical for extensibility. If Product stripped unrecognised fields, adding a custom field would be permanently lost on the next `product feature status` invocation.

**Rejected alternatives:**
- **Semver for schema** — over-engineered. Schema evolution for a flat YAML structure does not benefit from the three-level distinction.
- **No versioning, always latest** — the path to silent data corruption. Rejected without further consideration.
- **Per-file schema version** — each artifact file declares its own schema version. Rejected because it makes migration a per-file operation with no single point of truth. `product.toml` is the correct single source of schema version truth.