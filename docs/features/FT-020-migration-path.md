---
id: FT-020
title: Migration Path
phase: 1
status: complete
depends-on: []
adrs:
- ADR-014
- ADR-017
tests:
- TC-060
- TC-061
- TC-062
- TC-063
- TC-064
- TC-065
- TC-080
- TC-081
- TC-082
- TC-083
- TC-084
- TC-085
- TC-162
- TC-275
domains:
- api
- data-model
domains-acknowledged:
  ADR-048: Predates ADR-048; this feature does not author the canonical .product/ folder layout. FT-057 owns the migration command and discovery fallback.
  ADR-047: Predates ADR-047; this feature does not author the functional-spec body convention. FT-055 owns the structural validator and W030 mechanics.
  ADR-041: Pre-dates ADR-041; this feature does not author absence TCs or set removes/deprecates on ADRs. FT-047 owns the mechanics.
  ADR-043: Predates ADR-043; feature command adapters were written before the slice+adapter pattern was formalised and are not retroactively refactored.
  ADR-042: Pre-dates ADR-042; this feature does not define TC types or validate the type vocabulary. FT-048 owns the mechanics.
  ADR-040: predates ADR-040; feature ships independently of the unified verify / LLM-boundary pipeline
  ADR-018: Predates the 2026-04-22 scope promotion of ADR-018 to cross-cutting. Test coverage reflects the property/session/benchmark strategy as it existed when this feature shipped; not retroactively reclassified.
---

Migration is a two-phase extract-then-confirm process. See ADR-017 for full heuristic specification.

```bash
# Dry run — see what would be created
product migrate from-adrs picloud-adrs.md --validate
product migrate from-prd picloud-prd.md --validate

# Execute — write files, skip existing
product migrate from-adrs picloud-adrs.md --execute
product migrate from-prd picloud-prd.md --execute

# Interactive — review each artifact before writing
product migrate from-prd picloud-prd.md --interactive

# Post-migration: fill in link gaps and generate checklist
product graph check
product checklist generate
```

The migration parser uses heading structure to detect artifact boundaries and extracts phase references, status markers, and test criteria from subsections. It does not infer `depends-on` edges or feature→ADR links — those require human review and are filled in via `product feature link` commands after migration.

The source document is never modified. Migration can be re-run safely.

---

---

## Description

FT-020 covers the full migration path: importing legacy monolithic PRD and ADR documents into the Product artifact structure (`product migrate from-prd`, `product migrate from-adrs`), and upgrading the front-matter schema between Product binary versions (`product migrate schema`). Migration is a two-phase extract-then-confirm process per ADR-017 — no files are written until the developer explicitly passes `--execute` or `--interactive`. Schema versioning is governed by ADR-014.

## Functional Specification

### Inputs

- `product migrate from-prd SOURCE.md [--validate|--execute|--interactive|--overwrite] [--yes]`
- `product migrate from-adrs SOURCE.md [--validate|--execute|--interactive|--overwrite] [--yes]`
- `product migrate schema [--dry-run] [--from N]`
- Source PRD or ADR document: unstructured markdown with heading-delimited sections
- `product.toml` `schema-version` field (integer)

### Outputs

- `--validate` (default safe mode): prints a migration plan (artifact count, filenames, extracted types, warnings, conflicts) to stdout; writes nothing
- `--execute`: writes all proposed artifact files atomically, skipping files that already exist; reports skipped files
- `--overwrite`: as `--execute` but overwrites existing files (requires explicit confirmation unless `--yes`)
- `--interactive`: for each proposed artifact, prints proposed front-matter + first 200 chars of body, prompts `[a]ccept / [e]dit / [s]kip / [q]uit`; `edit` opens `$EDITOR`
- `product migrate schema`: updates front-matter fields in-place across all artifact files; writes `schema-version` in `product.toml` last; reports N files updated / M unchanged
- W008 warning: ADR `status` field not found, defaulted to `proposed`
- W009 warning: no test subsection found in ADR, no TC files extracted
- E008 error (startup): `schema-version` in `product.toml` exceeds the binary's supported version (TC-060); W007 warning when the repo is behind current schema (TC-061)

### State

- Migration is stateless between runs: re-running `product migrate` is idempotent — existing files are skipped (not overwritten unless `--overwrite`).
- `product migrate schema` is also idempotent: applying the same migration step twice leaves files byte-identical to the first run.
- The source PRD/ADR document is never modified — it is a read-only input.

### Behaviour

**PRD → Features (`from-prd`):**
1. Scan for H2 headings; exclude known non-feature headings (`Vision`, `Goals`, `Non-Goals`, `Overview`, etc.).
2. For each candidate feature: extract title (strip leading numbers/punctuation), infer `phase` from nearest preceding `### Phase N` heading (default 1), set `status: planned`.
3. Checklist inference: checked items (`- [x]`) in a checklist section → `status: complete`.
4. Feature body = section content until the next H2.
5. `depends-on`, `adrs`, and `tests` are left empty — these require human review.

**ADRs → ADR files + TCs (`from-adrs`):**
1. Scan for H2 headings matching `ADR-NNN:` or `## ADR-NNN`.
2. Extract `id`, `title`, `status` (from `**Status:**` line), `supersedes`/`superseded-by`.
3. Within each ADR body, extract test criteria from subsections matching `### Test coverage`, `### Test criteria`, `### Tests`, `### Exit criteria`, `### Scenarios`; produce one TC file per bullet; infer type from bullet content keywords (`chaos`, `invariant`, `exit-criteria`, else `scenario`).

**Schema migration (`migrate schema`):**
1. Read `schema-version` from `product.toml`.
2. Apply each migration step in sequence (1→2, 2→3, etc.) using the stored migration functions.
3. Write updated artifact files atomically (ADR-015).
4. Update `schema-version` in `product.toml` last.
5. If any write fails mid-migration, stop and report; leave `schema-version` unchanged; re-running is safe.

### Invariants

- The source document is never modified — `product migrate` is always a read-only operation on the source (TC-162 asserts this).
- `--validate` never writes any files — zero new files after a validate run (ADR-017).
- Migration is idempotent: re-running `product migrate from-prd --execute` on the same repo skips all existing files.
- `product migrate schema` never rolls back partially-written files — each file is written atomically; re-running is the recovery path.
- Unknown front-matter fields are preserved on write; migration never strips fields it does not recognise (ADR-014).

### Error handling

- E008: `schema-version` in `product.toml` exceeds supported version → hard error on startup; all commands blocked until binary is upgraded (TC-060).
- W007: repo `schema-version` is older than current → advisory warning on startup; commands still execute (TC-061).
- Conflict on `--execute`: existing file → skip and report skip; no content is overwritten without `--overwrite`.
- Write failure mid-migration: error is reported, remaining files not written, `product.toml` schema version unchanged.
- W008/W009: ADR missing status or test subsection → emit warning, continue migration.

### Boundaries

- `product migrate` does not infer `depends-on` edges or feature→ADR links from prose content; those require human review via `product feature link` after migration.
- `product migrate` does not modify `product.toml` (other than `schema-version` in `migrate schema`) or `CHECKLIST.md`.
- LLM-assisted migration (`--ai`) is not implemented in this feature (possible future extension per ADR-017).

## Out of scope

- Inferring `depends-on` edges or feature→ADR links automatically during migration.
- Modifying the source PRD or ADR document.
- Providing an undo command (`--validate` → `--execute` is the safety model; no undo log is maintained).
- LLM-assisted migration (reserved for a future `product migrate --ai` variant).
