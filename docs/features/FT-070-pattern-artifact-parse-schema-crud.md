---
id: FT-070
title: Pattern Artifact — Parse, Schema, CRUD
phase: 5
status: planned
depends-on:
- FT-003
- FT-038
- FT-041
- FT-066
adrs:
- ADR-050
- ADR-002
- ADR-038
- ADR-043
- ADR-020
- ADR-013
- ADR-015
- ADR-018
- ADR-047
- ADR-048
- ADR-051
- ADR-040
- ADR-041
- ADR-042
- ADR-049
tests:
- TC-812
- TC-813
- TC-814
- TC-815
- TC-816
- TC-817
- TC-818
- TC-819
domains:
- api
- data-model
- storage
domains-acknowledged: {}
---

## Description

Introduce **PAT-XXX** as a new artifact type, peer to FT / ADR / TC /
DEP, per ADR-050. This feature lands the foundational layer: the
front-matter schema, the parser, the file layout under
`docs/patterns/`, the CLI commands for create / show / list / status
/ link, and integration with the unified request interface
(ADR-038) so atomic batches can write patterns alongside other
artifacts.

Patterns are reusable implementation knowledge — how to build a
thing of this shape in this codebase. They sit alongside ADRs (which
say *why*) and TCs (which say *whether it works*). Patterns answer
*how*. The lifecycle is accretion (no supersession audit trail);
the only state transitions are `live ↔ deprecated`. ADR-050 is the
governing decision; this feature is the parse / write surface that
makes patterns observable in the graph.

The deliberate scope of this feature is **structural plumbing only**.
Walking patterns into the context bundle (F2), running pattern-aware
graph health checks (F2), pattern-aware authoring (F4),
pattern-aware implement (F5), and the seed catalog (F6) all land in
later features. Splitting these keeps each PR small enough to review
and each TC focused.

---

## Depends on

- **FT-003** — Front-Matter Schema. Owns the parser pipeline this
  feature extends with `PatternFrontMatter`.
- **FT-038** — Front-Matter Field Management. Owns the granular
  mutation tools (`product pattern link`, `product pattern status`)
  that this feature replicates for PAT.
- **FT-041** — Product Request — Unified Write Interface. Patterns
  must be expressible in `product_request_apply` from day one;
  ADR-038 batching is the only path that supports bidirectional
  `examples:` ↔ `feature.patterns:` materialisation atomically.
- **FT-066** — MCP Parity for Feature/TC Status Writes. The
  reciprocation pattern for `examples:` ↔ `feature.patterns:` is the
  same shape FT-066 established for `feature.tests` ↔
  `tc.validates.features`. The PAT slice copies that approach.

---

## Functional Specification

### Inputs

- **CLI:**
  - `product pattern new "Title"` — scaffold a new PAT file.
  - `product pattern show PAT-XXX` — render details.
  - `product pattern list [--status live|deprecated]` — enumerate.
  - `product pattern status PAT-XXX <live|deprecated> [--deprecated-by PAT-YYY]` — transition.
  - `product pattern link PAT-XXX [--adr ADR-NNN] [--requires PAT-YYY] [--example FT-NNN]` — add front-matter links.
- **MCP (parity with CLI; lands in this feature, not F4):**
  - `product_pattern_new { title }`.
  - `product_pattern_show { id }`.
  - `product_pattern_list { status? }`.
  - `product_pattern_status { id, status, deprecated_by? }`.
  - `product_pattern_link { id, adr?, requires?, example? }`.
- **Request interface:** `product_request_apply` accepts `type:
  pattern` records under `artifacts:` (create) and `changes:`
  (mutate) the same way it accepts `feature` / `adr` / `tc` / `dep`.
- **`product.toml`:**
  - `[paths].patterns` (default `"docs/patterns"`).
  - `[prefixes].pattern` (default `"PAT"`).
  - `[patterns].body-sections` — list of required H2 headings
    (default mirrors ADR-050's five-section list).
  - `[patterns].body-severity` — `warning | error` (default
    `warning`, mirroring W030's severity model for features).

### Outputs

- **`product pattern new`:** writes `docs/patterns/PAT-NNN-<slug>.md`
  with the scaffolded front-matter + empty H2 sections; prints the
  created path. JSON form returns
  `{ id, path }`.
- **`product pattern show`:** prints front-matter + body sections.
  JSON form returns the typed front-matter plus the body.
- **`product pattern list`:** one line per PAT (`PAT-NNN  <status>
  <title>`). JSON form returns `[{ id, status, title, domains }]`.
- **`product pattern status`:** writes the updated front-matter;
  prints the transition. JSON form returns `{ id, status,
  previous-status, deprecated-by? }`.
- **`product pattern link`:** writes the updated front-matter (and
  reciprocates on link targets — see Behaviour); JSON form returns
  `{ id, writes: [{ path, kind }], reciprocated: [{ id, field }] }`
  using the exact shape FT-066 established for `product_feature_link`.
- **Request apply:** new patterns appear in the `created` array;
  pattern changes appear in `changed`. Existing envelope; no shape
  change.

### State

- New artifact files at `docs/patterns/PAT-NNN-<slug>.md`. ID
  allocation uses the existing monotonic-allocator path
  (`src/types/id_allocator.rs` analogue), guaranteeing no collisions
  with concurrent writes.
- New `PatternFrontMatter` struct in `src/types/pattern.rs`
  alongside `FeatureFrontMatter` / `AdrFrontMatter`. Fields per
  ADR-050:
  - `id: String`
  - `title: String`
  - `status: PatternStatus` (enum: `Live`, `Deprecated`)
  - `domains: Vec<String>`
  - `adrs: Vec<String>`
  - `requires: Vec<String>`
  - `examples: Vec<String>`
  - `deprecated_by: Option<String>`
- `KnowledgeGraph` gains a `patterns: HashMap<String,
  PatternArtifact>` field, populated by `parser::load_all` from the
  configured `[paths].patterns` directory.
- New slice module `src/pattern/` following the established
  slice + adapter shape (ADR-043):
  - `plan_new` / `apply_new`
  - `plan_status_change` / `apply_status_change`
  - `plan_link` / `apply_link`
  - `render_list_text`, `render_show_text`
  - Unit tests in `src/pattern/tests.rs`.

### Behaviour

1. **Scaffold (`product pattern new`).** Allocate the next free
   PAT ID, slugify the title, write the file with all required H2
   headings as empty sections, set `status: live`, all link arrays
   empty. The body sections list comes from
   `[patterns].body-sections`; the scaffold writes exactly those
   headings.

2. **Status transition (`product pattern status`).** Validate the
   target value is `live` or `deprecated`. When transitioning to
   `deprecated`, accept an optional `--deprecated-by PAT-YYY` arg
   and validate the target exists in the graph. When transitioning
   to `live`, clear any existing `deprecated_by` value. Write
   through `plan_status_change` → `apply_status_change`.

3. **Link (`product pattern link`).** For each provided link target:
   - `--adr ADR-N`: add to `pattern.adrs`; **no reciprocation on
     ADR side** (ADRs do not carry a `patterns:` back-reference in
     this feature; this is parity with how ADRs handle `features:`
     today — F2 may add the back-reference if surfacing patterns
     in `product impact ADR-N` requires it).
   - `--requires PAT-Y`: add to `pattern.requires`; cycle-check via
     the same algorithm the feature `depends-on` slice uses
     (FT-062). Reject with the existing E003 code on cycle.
   - `--example FT-N`: add to `pattern.examples` **and** reciprocate
     by adding the linking PAT id to `FT-N.patterns` in the same
     atomic batch. This is the bidirectional materialisation
     ADR-050 specifies. The feature schema gains a `patterns:
     Vec<String>` field (default empty) to support the back-link.

4. **Parse.** `parser::load_all` walks `[paths].patterns` and
   produces `PatternArtifact { front, body }` values keyed by id.
   Unknown front-matter keys are rejected with the existing E001
   formal-block / parse-error machinery extended for the new
   schema.

5. **Body section validation.** A new warning code (allocate W031;
   confirm in implementation) fires when a `live` pattern is
   missing a required H2 from `[patterns].body-sections`. Mirrors
   W030 exactly. The error variant escalates when
   `[patterns].body-severity = "error"`. F2 wires this into
   `product graph check`; this feature only adds the parser hook
   and the config keys.

6. **Request interface.** `product_request_apply` accepts:
   - `artifacts: [{ type: pattern, title, status?, domains?, adrs?,
     requires?, examples?, body? }]` for create.
   - `changes: [{ id: PAT-NNN, op: add|remove|set, field: <one of
     status, domains, adrs, requires, examples, deprecated-by,
     body>, value: ... }]` for mutate.
   - Bidirectional `examples:` ↔ `feature.patterns:` writes are
     emitted automatically by the apply pipeline, not requested
     explicitly — matching the FT-066 model for `tests:` ↔
     `validates.features`.

7. **`feature.patterns` field.** Add to `FeatureFrontMatter` with
   default empty. Parser accepts. Schema doc gains the new field.
   No `product feature link --pattern` flag in this feature — F4
   adds the symmetric authoring surface.

### Invariants

- Every PAT file in `[paths].patterns` has a unique `id` matching
  its filename slug prefix.
- Every value in `pattern.requires` resolves to a real PAT id; no
  pattern requires itself directly or transitively (cycle-checked
  via the existing depends-on cycle detector, reporting E003).
- Every value in `pattern.examples` resolves to a real feature id;
  the reciprocal `feature.patterns` membership is materialised in
  the same atomic batch as the `pattern.examples` write
  (request-apply invariant; FT-066 parity).
- Every value in `pattern.adrs` resolves to a real ADR id (broken
  link reported as E002, the existing code).
- `pattern.deprecated_by` is non-empty only when `pattern.status =
  deprecated`; transitioning to `live` clears it.
- Concurrent writes to two different patterns succeed under the
  existing repo-lock model (`shared::acquire_write_lock_typed`).
- The on-disk shape produced by an MCP request for the same input
  is byte-identical to the CLI shape (MCP / CLI parity; FT-066
  invariant generalised).

### Error handling

- **E001** — Pattern front-matter parse error (extends the existing
  formal-block / YAML parser machinery to the new schema).
- **E002** — Broken link (unknown ADR / PAT / FT referenced in
  link arrays).
- **E003** — `requires:` cycle (reuses the existing
  depends-on cycle detector and code).
- **E026** — Unknown field name in a `changes:` mutation against a
  pattern (reuses the existing FT-064 strict-validation code).
- **NotFound** — `product pattern show` / `link` / `status` against
  an unknown PAT id (reuses the existing ProductError variant).
- **W031** (new) — `live` pattern missing a required H2 body
  section. Escalates to error when
  `[patterns].body-severity = "error"`.
- MCP errors propagate via `format!("{}", e)` per the FT-066 model;
  the JSON-RPC envelope carries the error text.

### Boundaries

- **In scope:** schema, parser, CLI commands, MCP tools (CRUD only
  — authoring sessions are F4), request-apply integration,
  bidirectional `examples:` ↔ `feature.patterns:` materialisation,
  `feature.patterns` field addition with parser support, body
  section validation hook (config + parser; F2 wires it into
  `graph check`), W031 code allocation, the `--no-fail-fast` test
  suite passing.
- **Out of scope:** authoring sessions for patterns (F4); pattern
  participation in `product context` bundles (F2); pattern
  participation in `product impact` (F2); pattern participation in
  Brandes centrality (F2); pattern-aware preflight (F4 or F5);
  pattern-aware `product implement` (F5); the seed catalog itself
  (F6); deprecation linting (warn when a live feature cites a
  deprecated pattern — F2).
- **Out of scope:** content-hash immutability for patterns. ADR-050
  explicitly excludes it.
- **Out of scope:** schema migration of the `feature.patterns`
  field for existing features. The field defaults to empty; older
  features simply have no patterns until something cites them
  through F4 / F6.

---

## Out of scope

- All items listed under "Boundaries → Out of scope" above.
- A `product feature link --pattern` CLI flag (added by F4 as the
  symmetric authoring surface; F1 only adds the field).
- A `product_feature_link` MCP arg accepting `pattern: PAT-NNN`
  (added by F4 for the same reason).
- Schema migration tooling for retrofitting `patterns: []` onto
  existing feature files — the default is empty, so no migration
  is required.

---

## Implementation notes

- **New module `src/pattern/`** following ADR-043:
  - `mod.rs` re-exports the slice surface.
  - `plan.rs` — pure `plan_*` functions.
  - `apply.rs` — `apply_*` functions writing via
    `fileops::write_file_atomic` / `write_batch_atomic`.
  - `link.rs` — the link plan that emits the bidirectional
    `examples:` ↔ `feature.patterns:` writes (mirrors
    `src/feature/link.rs` from FT-066).
  - `render.rs` — `render_list_text`, `render_show_text`.
  - `tests.rs` — unit tests against in-memory graphs.
- **New module `src/types/pattern.rs`** defines `PatternFrontMatter`,
  `PatternStatus`, `PatternArtifact`. Add `patterns: HashMap<String,
  PatternArtifact>` to `KnowledgeGraph` in `src/types/mod.rs`.
- **Add `patterns: Vec<String>` to `FeatureFrontMatter`** with
  default empty. Update the schema render in
  `src/commands/schema.rs` to surface the new field.
- **`src/parser/`** — extend the per-type loader registry to
  include patterns. The existing `[paths]` walk picks up the new
  directory once `[paths].patterns` is configured. New TC type
  pattern parsing is not affected.
- **`src/config.rs`** — add `PatternsSection` with `body_sections`
  and `body_severity` (mirrors `FeaturesSection` from FT-055).
  Default `body_sections` is the five-item list from ADR-050.
- **`src/commands/pattern.rs`** — thin clap adapter, dispatches to
  the slice. New CRUD subcommands wired into the `Commands` enum in
  `commands/mod.rs`.
- **`src/mcp/registry.rs`** + `src/mcp/tools/write.rs` — register
  the five new MCP tools. All write tools declare `requires_write:
  true`. Each handler calls into `pattern::plan_*` +
  `apply_*` directly. No envelope-only stubs (FT-066 invariant).
- **`src/request/`** — extend the validate / apply pipeline to
  accept `type: pattern`. The bidirectional materialisation is
  added to `apply.rs` alongside the existing `tests` ↔
  `validates.features` writes.
- **`src/types/id_allocator.rs`** — reuse the existing allocator;
  add a path for the `PAT` prefix.
- **`src/error.rs`** — no new error codes beyond W031 (warning,
  not in the error enum). W031 lives in the existing
  `GraphCheckFinding` family.
- **File-length budget:** each new file targets ≤ 200 lines to
  leave headroom under the 400-line cap; the slice splits
  naturally across `plan.rs` / `apply.rs` / `link.rs` / `render.rs`
  to keep modules small.
- **Concurrency:** all writes hold the existing repo write-lock;
  no new locking primitives.

---

## Acceptance criteria

A developer can:

1. Run `product pattern new "Slice + Adapter module structure"`
   and observe a new file at
   `docs/patterns/PAT-001-slice-adapter-module-structure.md` with
   the configured H2 sections present and front-matter populated
   with `id`, `title`, `status: live`, empty link arrays.
2. Run `product pattern list` and observe the new PAT in the table
   (text and JSON forms parity-tested).
3. Run `product pattern link PAT-001 --requires PAT-002` after
   creating PAT-002, then run `product pattern link PAT-002
   --requires PAT-001` and observe an E003 cycle error.
4. Run `product pattern link PAT-001 --example FT-066` against a
   repo where FT-066 exists, then `product feature show FT-066` and
   observe `patterns: [PAT-001]` in the front-matter (bidirectional
   materialisation).
5. Run `product pattern status PAT-001 deprecated --deprecated-by
   PAT-042` and observe `status: deprecated` and `deprecated-by:
   PAT-042` in the front-matter. Run `product pattern status
   PAT-001 live` and observe `deprecated-by` removed.
6. Apply a YAML request containing
   `artifacts: [{ type: pattern, title: "...", status: live, adrs:
   [ADR-050], examples: [FT-070] }]` through
   `product_request_apply` and observe the file written **and**
   `FT-070.patterns` populated in the same atomic batch.
7. Invoke `product_pattern_new` over MCP and observe a file on
   disk byte-identical to the CLI shape (parity invariant — TC
   matches FT-066's TC-778 shape, observing both `file` and
   `mcp-response`).
8. Run `cargo t`, `cargo clippy -- -D warnings -D
   clippy::unwrap_used`, and `cargo build` — all pass.

See the consolidated exit-criteria TC scaffolded by this feature.

---

## TC scaffolding plan

Every TC scaffolded for this feature carries:

- A non-empty `observes:` list per ADR-051 (this feature is the
  first to dogfood the new field — F3 adds the validator that
  enforces it; F1 simply demonstrates the discipline).
- `runner: cargo-test` and `runner-args: tc_NNN_<snake_case_title>`
  per CLAUDE.md.
- Body assertions on the named surfaces, not on `Ok(_)` shape.
- AISP block type matching TC type (`scenario` → `⟦Λ:Scenario⟧`).

Planned TCs:

| TC | Type | `observes:` | What it asserts |
|---|---|---|---|
| `pattern_new_writes_file_with_required_sections` | scenario | `[file]` | After `product pattern new`, the file at `docs/patterns/PAT-NNN-<slug>.md` exists and contains every configured H2 heading. |
| `pattern_link_requires_cycle_returns_e003` | scenario | `[exit-code, stdout]` | `product pattern link PAT-A --requires PAT-B` after `pattern link PAT-B --requires PAT-A` exits with code 3 and prints the E003 cycle text. |
| `pattern_link_example_materialises_feature_patterns` | scenario | `[file, graph]` | After `product pattern link PAT-X --example FT-Y`, both `PAT-X.examples` and `FT-Y.patterns` are updated in the same batch (assert by reading both files and reloading the graph). |
| `request_apply_pattern_creates_file_and_back_link` | scenario | `[file, graph]` | A YAML request creating a pattern with `examples: [FT-N]` produces the PAT file and the FT-N.patterns membership atomically. |
| `mcp_pattern_new_writes_to_disk` | scenario | `[file, mcp-response]` | `product_pattern_new` over MCP produces an on-disk file byte-identical to the CLI shape (FT-066 TC-778 generalisation). |
| `mcp_pattern_status_writes_status_field` | scenario | `[file, mcp-response]` | `product_pattern_status PAT-X deprecated --deprecated-by PAT-Y` over MCP writes both fields to disk. |
| `pattern_status_to_live_clears_deprecated_by` | scenario | `[file]` | After deprecation then re-promotion, `deprecated-by` is absent. |
| `ft_070_exit_criteria_pattern_crud_parity` | exit-criteria | n/a | Aggregates all the above; asserts `cargo t`, `cargo clippy`, `cargo build` all green; asserts the legacy "envelope-only" anti-pattern absent from the pattern slice (grep guard). |
