---
id: ADR-017
title: Migration Command Specification
status: accepted
features: []
supersedes: []
superseded-by: []
domains: []
scope: domain
---

**Status:** Accepted

**Context:** `product migrate from-prd` and `product migrate from-adrs` were listed in the phase plan without specification. These are the highest-risk commands in Product: they read freeform markdown prose and write many new files based on heuristic parsing. Unlike all other Product commands, they produce artifacts that require human review — the parser cannot determine intent with certainty from unstructured input.

The migration command must be specified completely before implementation: what heuristics it uses, what output it produces, what rollback story exists, and how the developer confirms and corrects the output.

**Decision:** Migration is a two-phase process: **extraction** (parse the source document, propose a set of artifacts) then **confirmation** (developer reviews and commits). No files are written until the developer explicitly confirms. Extraction is deterministic given a document; there is no ambiguous state. All extracted artifacts are written atomically as a batch.

---

### Supported Source Documents

**`product migrate from-prd SOURCE.md`** — parses a monolithic PRD document. Detects features from heading structure.

**`product migrate from-adrs SOURCE.md`** — parses a monolithic ADR document. Detects individual ADRs and extracts test criteria from ADR subsections.

Both commands accept `--validate` for dry-run output without writing files.

---

### Extraction Heuristics: PRD → Features

The parser scans for H2 (`##`) headings that match feature-like patterns. A heading is treated as a feature boundary if it:
- Is at H2 level
- Does not match a set of known non-feature headings: `Vision`, `Goals`, `Non-Goals`, `Target Environment`, `Core Architecture`, `Open Questions`, `Resolved Decisions`, `Phase Plan`, `Overview`, `Introduction`, `Background`, `References`

For each candidate feature heading:
- `title` is the heading text, stripped of leading numbers and punctuation (`## 5. Products and IAM` → `Products and IAM`)
- `phase` is inferred from the nearest preceding `### Phase N` heading, or 1 if none found
- `status` is `planned` by default
- `depends-on` is empty — not inferred (requires human judgment)
- `adrs` and `tests` are empty — not linked (requires `product graph check` to identify gaps)

The body of the section (all content until the next H2) becomes the feature file body.

**Checklist inference:** If the source PRD contains a checklist section (lines matching `- [ ]` or `- [x]`), checked items set the corresponding feature `status` to `complete`, unchecked items remain `planned`. This handles migration from an existing `checklist.md`.

---

### Extraction Heuristics: ADRs → ADR Files + Test Criteria

The parser scans for H2 (`##`) headings matching the pattern `ADR-NNN:` or `## ADR-NNN`.

For each ADR:
- `id` is extracted from the heading prefix
- `title` is the heading text after the prefix
- `status` is extracted from a `**Status:**` line in the body (`Accepted`, `Proposed`, etc.)
- `supersedes` and `superseded-by` are extracted from `**Supersedes:**` / `**Superseded By:**` lines
- `features` is empty — not inferred

**Test criteria extraction:** Within each ADR body, the parser looks for subsections matching these heading patterns:
- `
### Output Format (Dry-Run and Confirmation)

`product migrate from-adrs picloud-adrs.md --validate` produces:

```
Migration plan: picloud-adrs.md → 9 ADRs, 34 test criteria

ADR files to create:
  docs/adrs/ADR-001-rust-language.md                (status: accepted)
  docs/adrs/ADR-002-openraft-consensus.md            (status: accepted)
  ... (7 more)

Test criteria files to create:
  docs/tests/TC-001-binary-compiles.md               (type: exit-criteria, adr: ADR-001)
  docs/tests/TC-002-raft-leader-election.md           (type: scenario, adr: ADR-002)
  ... (32 more)

Warnings:
  [W008] ADR-003: status not found, defaulting to "proposed"
  [W009] ADR-007: no test subsection found — no test criteria extracted

Conflicts:
  docs/adrs/ADR-001-rust-language.md already exists — will skip (use --overwrite to replace)

Run without --validate to create these files.
Run with --interactive for per-artifact confirmation.
```

---

### Execution Modes

**`--validate`** (default safe mode) — prints the migration plan and exits. No files written.

**`--execute`** — writes all proposed files. Skips files that already exist. Reports skipped files.

**`--overwrite`** — writes all proposed files. Overwrites files that already exist. Requires explicit confirmation prompt unless `--yes` is also passed.

**`--interactive`** — for each proposed artifact, prints the proposed front-matter and first 200 characters of body, then prompts: `[a]ccept / [e]dit / [s]kip / [q]uit`. `edit` opens the proposed content in `$EDITOR`. This mode is recommended for first migration of a large document.

---

### Rollback

Migration writes files atomically (ADR-015). If any write fails mid-batch, the error is reported and the remaining files are not written. Already-written files are not rolled back — they are valid artifact files. The developer can delete them manually or run migration again with `--overwrite`.

`product migrate` never modifies the source document. The source PRD and ADR files are read-only inputs.

`product migrate` never modifies `product.toml` or `checklist.md`. These are updated by `product checklist generate` after migration.

---

### Post-Migration Workflow

After migration, the recommended workflow is:

```bash
product migrate from-adrs picloud-adrs.md --execute
product migrate from-prd picloud-prd.md --execute
product graph check          # surfaces all broken links (features with no ADRs, etc.)
# manually add depends-on edges and feature→ADR links based on graph check output
product graph check          # should now exit 0 or 2 (warnings only)
product checklist generate
```

`product graph check` after migration will always produce warnings (W001 orphaned ADRs, W002 features with no tests, etc.) because the link edges are not inferred. This is expected and documented. The developer fills in edges using `product feature link` commands.

---

**Rationale:**
- Two-phase extraction → confirmation prevents the most dangerous failure mode: writing 40 files and discovering the heuristics got 10 of them wrong. With `--validate`, the developer sees the full plan before committing.
- `--interactive` mode is the recommended path for a first migration. It forces a review of each artifact, which is valuable because the developer catches heuristic errors and also re-familiarises themselves with the content as it is being structured.
- Not inferring `depends-on` edges or feature→ADR links is correct. These relationships require semantic understanding of the content, not pattern matching on structure. Guessing wrong would be worse than leaving them empty.
- Preserving the source document unchanged means migration can be re-run safely if the first attempt was wrong. The source is always the ground truth.

**Rejected alternatives:**
- **Infer feature→ADR links from ADR body mentions of feature names** — too fragile. ADR prose mentions feature concepts by name but not by ID. Mismatches would require more cleanup than just linking manually.
- **Write all files immediately, provide `product migrate undo`** — rollback is complex in a file system context. The `--validate` → `--execute` two-phase approach achieves the same safety without requiring an undo log.
- **LLM-assisted migration** — use an LLM to interpret the PRD and generate structured artifacts. Would produce higher-quality extraction for ambiguous documents. Rejected for v1: Product must work without network access or API keys. Can be added as `product migrate --ai` in a future version.

**Test coverage:**

Scenario tests:
- `migrate_prd_heading_detection.rs` — parse a PRD with 5 H2 sections, 2 of which are `Goals` and `Non-Goals` (excluded). Assert exactly 3 feature files are proposed.
- `migrate_prd_phase_inference.rs` — parse a PRD with `### Phase 1` and `### Phase 2` headings. Assert features under each phase heading get the correct `phase` value.
- `migrate_prd_checklist_status.rs` — parse a PRD with a checklist section where 3 items are checked. Assert the 3 corresponding feature files have `status: complete`.
- `migrate_adrs_id_extraction.rs` — parse an ADR file with `## ADR-001: Rust` and `## ADR-002: openraft`. Assert exactly 2 ADR files are proposed with IDs `ADR-001` and `ADR-002`.
- `migrate_adrs_test_extraction.rs` — parse an ADR with a `### Test coverage` subsection containing 4 bullet points. Assert 4 test criterion files are proposed with `validates.adrs: [ADR-XXX]`.
- `migrate_adrs_test_type_inference.rs` — assert bullets containing "chaos" produce `type: chaos`, bullets containing "invariant" produce `type: invariant`, and others produce `type: scenario`.
- `migrate_validate_no_write.rs` — run `product migrate from-prd --validate`. Assert zero files are created in the repository.
- `migrate_execute_skips_existing.rs` — create `ADR-001-rust-language.md` before running migration. Assert the file is skipped and the skip is reported. Assert the original file content is unchanged.
- `migrate_interactive_skip.rs` — run `product migrate --interactive`, respond `s` (skip) to all prompts. Assert zero files are created.
- `migrate_source_unchanged.rs` — run `product migrate from-prd PRD.md --execute`. Assert the source `PRD.md` is byte-for-byte identical before and after the command.
- `migrate_picloud_prd.rs` — integration test: run migration against the actual PiCloud PRD. Assert at least 10 feature files are created. Assert `product graph check` exits with 2 (warnings) not 1 (errors) after migration (no broken links, only coverage gaps).


---