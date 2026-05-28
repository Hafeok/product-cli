---
id: FT-002
title: Repository Layout
phase: 1
status: complete
depends-on: []
adrs:
- ADR-002
- ADR-004
tests:
- TC-005
- TC-006
- TC-007
- TC-008
- TC-011
- TC-012
- TC-154
domains:
- data-model
domains-acknowledged:
  ADR-018: Predates the 2026-04-22 scope promotion of ADR-018 to cross-cutting. Test coverage reflects the property/session/benchmark strategy as it existed when this feature shipped; not retroactively reclassified.
  ADR-043: Predates ADR-043; feature command adapters were written before the slice+adapter pattern was formalised and are not retroactively refactored.
  ADR-042: Pre-dates ADR-042; this feature does not define TC types or validate the type vocabulary. FT-048 owns the mechanics.
  ADR-040: predates ADR-040; feature ships independently of the unified verify / LLM-boundary pipeline
  ADR-048: Predates ADR-048; this feature does not author the canonical .product/ folder layout. FT-057 owns the migration command and discovery fallback.
  ADR-041: Pre-dates ADR-041; this feature does not author absence TCs or set removes/deprecates on ADRs. FT-047 owns the mechanics.
  ADR-047: Predates ADR-047; this feature does not author the functional-spec body convention. FT-055 owns the structural validator and W030 mechanics.
---

```
/docs
  product.toml              ← repository config (name, prefix, phases)
  /features
    FT-001-cluster-foundation.md
    FT-002-products-iam.md
    FT-003-rdf-event-store.md
  /adrs
    ADR-001-rust-language.md
    ADR-002-openraft-consensus.md
  /tests
    TC-001-binary-compiles.md
    TC-002-raft-leader-election.md
    TC-003-raft-leader-failover.md
  /graph
    index.ttl               ← generated, never hand-edited
  checklist.md              ← generated, never hand-edited
```

Subdirectory names and file prefixes are configurable in `product.toml`. The layout above is the default.

---

---

## Description

FT-002 defines the canonical on-disk directory structure that Product expects for a repository. Artifact files are grouped into three subdirectories (`features/`, `adrs/`, `tests/`) under a common documentation root. The paths to these directories are declared in `product.toml` and are fully configurable; the layout shown in the prose above is the default. Two additional generated files live in this structure: `index.ttl` (an RDF/Turtle snapshot written by `product graph rebuild`, never read by Product itself) and `checklist.md` (regenerated after every `product verify` run). The layout is the physical contract between a repository and the Product CLI — every command scans these directories to rebuild the in-memory knowledge graph (ADR-002, ADR-003).

## Functional Specification

### Inputs

- A `product.toml` file at the repository root declaring the `[paths]` section with keys `features`, `adrs`, `tests`, `graph`, and `checklist`. All paths are relative to the repository root. If `product.toml` is absent, Product exits with a configuration error before scanning any directories.
- The filesystem directories identified by those paths, each containing zero or more `.md` files with YAML front-matter.

### Outputs

- An in-memory collection of all parsed `Feature`, `Adr`, and `TestCriterion` values produced by scanning the configured directories. This collection is handed to every subsequent command within the same invocation.
- On `product graph rebuild`: `docs/graph/index.ttl` written as an RDF/Turtle snapshot.
- On `product checklist generate` or after `product verify`: `checklist.md` regenerated from current feature statuses.

### State

Stateless between invocations. The configured directory paths are read from `product.toml` on every invocation. No path cache is maintained; no persistent index is consulted (ADR-003). The generated files (`index.ttl`, `checklist.md`) are write-only outputs of specific commands; they are never read back by Product.

### Behaviour

1. Product reads `product.toml` from the working directory (or the path given by `--config`) and extracts the `[paths]` section. If any required path key is absent, the default value is used (`docs/features`, `docs/adrs`, `docs/tests`, `docs/graph`, `docs/checklist.md`).
2. Product scans each configured directory for `*.md` files using a non-recursive glob. Files that do not match the glob are ignored.
3. Each `.md` file is parsed by splitting on the `---` front-matter delimiters. The artifact type is inferred from the `id` prefix in the front-matter (`FT-` → Feature, `ADR-` → Architectural Decision Record, `TC-` → Test Criterion). Files in a directory whose `id` prefix does not match the expected type for that directory are accepted but may produce a W001 warning.
4. All successfully parsed artifacts are loaded into the in-memory graph. Files that fail to parse (missing front-matter, malformed YAML) are skipped with an E001 error on stderr; scanning of remaining files continues.
5. `product graph rebuild` serialises the in-memory graph to `index.ttl` in the configured `graph` directory using Turtle syntax. If the directory does not exist, Product creates it. The file is written atomically (ADR-015).
6. `checklist.md` is written by `product checklist generate` and auto-regenerated after `product verify`. It is never hand-edited (ADR-007).

### Invariants

- Each configured directory must be readable by the current process; a permission error on any directory is a hard error (exit code 1), not a warning.
- Subdirectory names and file prefixes are configurable in `product.toml`; the default values (`docs/features`, `docs/adrs`, `docs/tests`) are not hardcoded in the binary.
- `index.ttl` and `checklist.md` are the only generated files; all other files in the layout are human-authored artifacts under version control.
- Two artifact files in the same prefix directory must not share an `id` value; duplicate IDs are reported as E005.

### Error handling

- `product.toml` absent or unreadable → hard error on startup with a hint to run `product init` or check the working directory.
- Configured directory does not exist → E002-style error reporting the missing path; the remaining directories are still scanned.
- `.md` file has no front-matter delimiters → E001, file skipped, scanning continues (ADR-013).
- Malformed YAML front-matter → E001 with file path and line number, file skipped (ADR-013).
- Duplicate `id` within a prefix type → E005, both files reported; neither is loaded into the graph.
- All errors go to stderr; stdout is reserved for command output (ADR-013).

### Boundaries

- Product scans only the top level of each configured directory — it does not recurse into subdirectories. Nested directory structures are not supported.
- The file naming convention (`FT-001-slug.md`) is a human convention; Product reads the `id` from front-matter, not from the filename. Filenames may diverge from the `id` without error, though this is discouraged.
- `index.ttl` and `checklist.md` are outputs of specific commands; they are never read back by any other Product command.

## Out of scope

- Recursive directory scanning or multi-level nesting of artifact directories.
- The `.product/` discovery fallback and migration command introduced by ADR-048 — covered by FT-057.
- File watching or incremental graph updates; the graph is always fully rebuilt from scratch on each invocation (ADR-003).
- Validation that filenames match the `id` field in front-matter — this is a convention, not an enforced constraint.
