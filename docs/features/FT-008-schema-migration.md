---
id: FT-008
title: Schema Migration
phase: 2
status: complete
depends-on:
- FT-003
adrs:
- ADR-002
- ADR-014
- ADR-016
tests:
- TC-060
- TC-061
- TC-062
- TC-063
- TC-064
- TC-065
- TC-179
domains:
- data-model
domains-acknowledged:
  ADR-043: Predates ADR-043; feature command adapters were written before the slice+adapter pattern was formalised and are not retroactively refactored.
  ADR-041: Pre-dates ADR-041; this feature does not author absence TCs or set removes/deprecates on ADRs. FT-047 owns the mechanics.
  ADR-040: predates ADR-040; feature ships independently of the unified verify / LLM-boundary pipeline
  ADR-018: Predates the 2026-04-22 scope promotion of ADR-018 to cross-cutting. Test coverage reflects the property/session/benchmark strategy as it existed when this feature shipped; not retroactively reclassified.
  ADR-042: Pre-dates ADR-042; this feature does not define TC types or validate the type vocabulary. FT-048 owns the mechanics.
  ADR-048: Predates ADR-048; this feature does not author the canonical .product/ folder layout. FT-057 owns the migration command and discovery fallback.
  ADR-047: Predates ADR-047; this feature does not author the functional-spec body convention. FT-055 owns the structural validator and W030 mechanics.
---

In-place schema upgrades for front-matter when the schema version changes.

```
product migrate schema --dry-run    # report what would change without writing
product migrate schema --execute    # update all files in place
```

The `schema-version` field in `product.toml` declares the current schema version. On startup, Product validates:
- E008 — forward incompatibility (file schema version > binary schema version)
- W007 — upgrade available (file schema version < binary schema version)

Migration functions are registered per version transition (e.g., v0→v1). Each migration function transforms front-matter in place while preserving unknown fields. Concurrent `product migrate schema` commands are prevented by advisory locking (E010).

### Exit Criteria

Run `product migrate schema` on a v0 repository — all files updated, `schema-version` bumped. Run two concurrent commands — one succeeds, one exits E010. No data corruption.

---

## Description

FT-008 provides in-place schema upgrade for all artifact front-matter when the Product binary's supported schema version advances past the version recorded in `product.toml` (ADR-014). The `product migrate schema` command applies registered migration functions in sequence (e.g. `migrate_v0_to_v1`) to every artifact file in the configured directories, writing updated files atomically (ADR-015) and updating `schema-version` in `product.toml` only after all files have been successfully migrated. Forward incompatibility (a repository whose `schema-version` exceeds the binary's supported maximum) is a hard error (E008). Backward compatibility (binary newer than repository) is a recoverable warning (W007) that allows all commands to continue while encouraging migration. The migration command is idempotent: running it twice produces zero changes on the second run (TC-063). Concurrent invocations are serialised by the advisory lock (E010, TC-179).

## Functional Specification

### Inputs

- `product migrate schema` CLI invocation with optional flags:
  - `--dry-run` — describe what would change without writing any files (TC-062).
  - `--execute` — perform the migration (same as the default when no flag is given).
  - `--from N` — override the source version (defaults to the `schema-version` in `product.toml`).
- `product.toml` declaring the current repository `schema-version` (integer string).
- All `.md` artifact files in the configured `features`, `adrs`, and `tests` directories.

### Outputs

- **`--dry-run`**: a summary to stdout listing each file that would be changed and the field-level transformations that would be applied. No files are modified.
- **`--execute` (or default)**: each artifact file that requires migration is rewritten atomically with the updated front-matter. A summary reporting the count of updated and unchanged files is printed to stdout. `schema-version` in `product.toml` is updated last, atomically.
- **Error/warning output**: E008 on startup if the file schema version exceeds binary support; W007 on startup if the file schema version is below the binary's current version (printed to stderr; commands continue).

### State

The `schema-version` integer in `product.toml` is the only persisted schema state. The registered migration functions (one per version step, e.g. `v0→v1`) are compiled into the binary. No migration history file is maintained; idempotency is achieved by re-running the same migration function, which produces no change if the target fields already have the post-migration values.

### Behaviour

1. On every Product invocation, `product.toml` is read and the `schema-version` is compared against the binary's supported range.
   - If the file version exceeds the binary's maximum: E008 hard error, exit code 1; no command executes.
   - If the file version is below the binary's current version: W007 printed to stderr; the command continues using backward-compatible defaults for missing fields.
2. `product migrate schema` acquires the advisory write lock (ADR-015) before touching any file.
3. For each version step from the current file version to the binary's current version, the corresponding migration function is applied to each artifact file in sequence:
   - The file's front-matter is deserialized, the migration function transforms the in-memory struct, and the file is rewritten atomically (ADR-015).
   - Unknown fields in the front-matter are preserved verbatim (ADR-014).
4. If any file write fails, the migration command reports the error with the file path and stops. `schema-version` in `product.toml` is not updated. Re-running `product migrate schema` is safe — the partially migrated files are individually valid for the new schema; the migration function is idempotent on already-migrated files.
5. After all files are successfully migrated, `schema-version` in `product.toml` is updated atomically as the final step.
6. `--dry-run` mode executes steps 3 and 4 in memory and prints the would-be changes without calling `write_file_atomic` or updating `product.toml`.

### Invariants

- `schema-version` in `product.toml` is updated only after all artifact files have been successfully migrated; it is never updated if any file write fails.
- Migration is idempotent: running `product migrate schema` on a repository already at the current schema version produces zero file changes and exits 0 (TC-063).
- Unknown front-matter fields are preserved through migration unchanged (ADR-014).
- All file writes during migration use `fileops::write_file_atomic` — no partial file states are observable (ADR-015).
- Concurrent `product migrate schema` invocations are serialised by the advisory lock; the second invocation exits with E010 (TC-179).

### Error handling

- `schema-version` in `product.toml` exceeds binary support → E008 on startup, exit code 1, no migration or other command runs.
- `schema-version` in `product.toml` below binary's current → W007 on stderr; commands continue.
- File write failure during migration → error reported with file path; `product.toml` schema version not updated; command exits with code 1.
- Advisory lock not acquired → E010 with PID and start time of lock holder; command exits without modifying any file.
- `--from N` specifying a version not supported by the binary → E008.

### Boundaries

- Migration functions are registered per integer version step; the binary always contains every migration function from version 0 to the current version, enabling upgrade from any historical version in one command.
- Migration applies to artifact front-matter only (Feature, ADR, TC files). `product.toml` itself has its `schema-version` field updated but its other fields are not migrated by this command.
- The migration command does not reformat or reorder front-matter fields beyond the specific field transformations defined by each registered migration function.

## Out of scope

- Per-file schema versioning — `product.toml` is the single source of schema version truth; per-file version fields are not supported (ADR-014).
- Downgrade (migration to an older schema version) — not supported. Schema versions increment monotonically.
- Migration of `product.toml` config fields other than `schema-version` — configuration evolution is handled by backward-compatible defaults, not a migration command.
- The discovery fallback for repositories using the `.product/` layout — covered by FT-057 (ADR-048).
