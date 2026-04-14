## Overview

Migration Path provides a safe, two-phase process for importing existing PRD and ADR documents into Product's knowledge graph. It parses monolithic markdown files, extracts structured artifacts (features, ADRs, and test criteria) using heading-based heuristics, and writes them as individual files with proper YAML front-matter. The process is split into extraction and confirmation phases so that no files are written until the developer has reviewed the plan. This feature also includes schema versioning and migration (`product migrate schema`) to handle front-matter evolution across Product releases. See ADR-017 for the full migration specification and ADR-014 for schema versioning.

## Tutorial

### Migrating your first PRD

Suppose you have a monolithic PRD at `picloud-prd.md` and want to bring it into Product.

1. Preview what the migration would produce:

   ```bash
   product migrate from-prd picloud-prd.md --validate
   ```

   This prints a migration plan showing which feature files would be created, their inferred phases, and any warnings — but writes nothing to disk.

2. Review the output. You will see entries like:

   ```
   Migration plan: picloud-prd.md → 12 features

   Feature files to create:
     docs/features/FT-001-cluster-bootstrap.md    (phase: 1, status: planned)
     docs/features/FT-002-raft-consensus.md        (phase: 1, status: complete)
     ...
   ```

3. If the plan looks correct, execute the migration:

   ```bash
   product migrate from-prd picloud-prd.md --execute
   ```

4. Run post-migration checks to surface link gaps:

   ```bash
   product graph check
   ```

   Expect warnings about orphaned features and missing ADR links — this is normal. The migration parser does not infer relationships.

5. Fill in the missing links manually:

   ```bash
   product feature link FT-001 --adr ADR-003
   ```

6. Generate the checklist:

   ```bash
   product checklist generate
   ```

### Migrating ADRs with test criteria

1. Preview the ADR migration:

   ```bash
   product migrate from-adrs picloud-adrs.md --validate
   ```

   The parser extracts ADR files and also detects test criteria from subsections titled `### Test coverage`, `### Test criteria`, `### Exit criteria`, or similar patterns.

2. For a large document, use interactive mode to review each artifact:

   ```bash
   product migrate from-adrs picloud-adrs.md --interactive
   ```

   For each proposed artifact, you will be prompted:

   ```
   [a]ccept / [e]dit / [s]kip / [q]uit
   ```

   Choose `e` to open the proposed content in your `$EDITOR` for corrections before writing.

3. Execute once satisfied:

   ```bash
   product migrate from-adrs picloud-adrs.md --execute
   ```

## How-to Guide

### Preview a migration without writing files

```bash
product migrate from-prd SOURCE.md --validate
product migrate from-adrs SOURCE.md --validate
```

The `--validate` flag prints the full migration plan — proposed files, inferred metadata, warnings, and conflicts — then exits without writing anything.

### Migrate a PRD and ADR document together

1. Run both migrations in sequence:

   ```bash
   product migrate from-adrs picloud-adrs.md --execute
   product migrate from-prd picloud-prd.md --execute
   ```

2. Check the graph for link gaps:

   ```bash
   product graph check
   ```

3. Add missing edges based on the graph check output:

   ```bash
   product feature link FT-001 --adr ADR-003
   ```

4. Regenerate the checklist:

   ```bash
   product checklist generate
   ```

### Re-run migration safely

Migration never modifies the source document. Re-running with `--execute` skips files that already exist and reports the skips. To overwrite existing files instead:

```bash
product migrate from-adrs picloud-adrs.md --overwrite
```

This prompts for confirmation. Add `--yes` to skip the prompt.

### Review artifacts interactively during migration

```bash
product migrate from-prd picloud-prd.md --interactive
```

For each proposed artifact, the tool displays the front-matter and the first 200 characters of the body, then prompts for action. Choose `e` to open the content in `$EDITOR` for corrections before it is written.

### Upgrade the front-matter schema

When Product is updated and a new schema version is available, you will see warning W007 on startup. To upgrade:

1. Preview what would change:

   ```bash
   product migrate schema --dry-run
   ```

2. Run the upgrade:

   ```bash
   product migrate schema
   ```

3. Verify the result — re-running is idempotent:

   ```bash
   product migrate schema
   # Reports: 0 files changed
   ```

### Handle a schema version mismatch

If `product.toml` declares a schema version newer than your binary supports, Product exits with error E008:

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

Upgrade your Product binary to resolve this.

## Reference

### Commands

| Command | Description |
|---------|-------------|
| `product migrate from-prd SOURCE.md` | Extract features from a monolithic PRD |
| `product migrate from-adrs SOURCE.md` | Extract ADRs and test criteria from a monolithic ADR document |
| `product migrate schema` | Upgrade front-matter to the current schema version |

### Flags for `from-prd` and `from-adrs`

| Flag | Description |
|------|-------------|
| `--validate` | Print migration plan without writing files |
| `--execute` | Write proposed files, skipping existing ones |
| `--overwrite` | Write proposed files, replacing existing ones (prompts for confirmation) |
| `--interactive` | Review each artifact before writing (`[a]ccept / [e]dit / [s]kip / [q]uit`) |
| `--yes` | Skip confirmation prompts (used with `--overwrite`) |

### Flags for `migrate schema`

| Flag | Description |
|------|-------------|
| `--dry-run` | Show what would change without writing files |
| `--from N` | Explicit source schema version (defaults to `product.toml` value) |

### Schema version configuration

In `product.toml`:

```toml
name = "myproject"
schema-version = "1"
schema-version-warning = false   # suppress W007 upgrade warning (optional)
```

### Extraction heuristics — PRD

- Features are detected from H2 (`##`) headings.
- Headings matching known non-feature names are excluded: `Vision`, `Goals`, `Non-Goals`, `Target Environment`, `Core Architecture`, `Open Questions`, `Resolved Decisions`, `Phase Plan`, `Overview`, `Introduction`, `Background`, `References`.
- Leading numbers and punctuation are stripped from titles (`## 5. Products and IAM` becomes `Products and IAM`).
- Phase is inferred from the nearest preceding `### Phase N` heading, defaulting to 1.
- Status defaults to `planned`. Checked checklist items (`- [x]`) set status to `complete`.
- `depends-on`, `adrs`, and `tests` fields are left empty — they require human review.

### Extraction heuristics — ADRs

- ADRs are detected from H2 headings matching `ADR-NNN:` or `## ADR-NNN`.
- Status is extracted from `**Status:**` lines. Missing status defaults to `proposed` with warning W008.
- `supersedes` and `superseded-by` are extracted from corresponding bold-prefix lines.
- Test criteria are extracted from subsections titled `### Test coverage`, `### Test criteria`, `### Exit criteria`, or similar.
- Test type is inferred from bullet keywords: `chaos` produces `type: chaos`, `invariant` produces `type: invariant`, `exit-criteria` from `### Exit criteria` headings, and all others produce `type: scenario`.
- Each extracted TC gets `validates.adrs` set to the source ADR ID.

### Warnings and errors

| Code | Meaning |
|------|---------|
| W007 | Schema upgrade available — run `product migrate schema` |
| W008 | ADR status not found in body, defaulting to `proposed` |
| W009 | ADR has no test subsection — no test criteria extracted |
| E008 | Schema version mismatch — binary too old for this repository |

### Atomicity and rollback

- All file writes are atomic (temp file + rename, per ADR-015).
- If a write fails mid-batch, already-written files remain (they are individually valid). The error is reported with the list of written and unwritten files.
- The source document is never modified.
- `product.toml` and `CHECKLIST.md` are not modified by migration — use `product checklist generate` separately.

## Explanation

### Why a two-phase process?

Migration from unstructured markdown is inherently heuristic. The parser makes educated guesses based on heading structure, but it cannot determine developer intent with certainty. The two-phase design — extraction then confirmation — prevents the most dangerous failure mode: silently writing dozens of files with incorrect metadata. The `--validate` flag lets the developer inspect the full plan before committing, and `--interactive` mode provides per-artifact review for first-time migrations of large documents (ADR-017).

### Why relationships are not inferred

The migration parser deliberately does not infer `depends-on` edges or feature-to-ADR links. These relationships require semantic understanding of content, not pattern matching on document structure. Guessing wrong would create misleading graph edges that are harder to clean up than empty links are to fill in. After migration, `product graph check` surfaces all missing links, and the developer fills them in using `product feature link` commands. This makes the human the authority on relationships, which aligns with the principle that front-matter is the source of truth (ADR-002).

### Schema versioning philosophy

Product uses integer schema versions rather than semver (ADR-014). Schema compatibility is binary — a field either exists with expected semantics or it does not — so patch and minor distinctions add no value. Forward incompatibility (binary older than schema) is a hard error because running old code against a new schema could silently produce incorrect graph output. Backward incompatibility (binary newer than schema) is a warning because the old schema is still fully readable. Unknown front-matter fields are always preserved on write, ensuring that custom tooling built on top of Product can extend front-matter without risk of data loss.

### Safe re-runs and idempotency

Migration is designed to be re-runnable. The source document is never modified, so it remains the ground truth. Running `--execute` a second time skips all existing files and reports the skips. Running `product migrate schema` twice results in zero changes on the second run. This idempotency means a failed or partial migration can always be retried without manual cleanup.
