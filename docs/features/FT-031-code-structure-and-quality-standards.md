---
id: FT-031
title: Code Structure and Quality Standards
phase: 3
status: complete
depends-on: []
adrs:
- ADR-001
- ADR-029
- ADR-043
tests:
- TC-369
- TC-370
- TC-371
- TC-372
- TC-373
- TC-374
- TC-375
- TC-376
- TC-377
- TC-378
- TC-379
- TC-380
- TC-402
domains: []
domains-acknowledged:
  ADR-040: predates ADR-040; feature ships independently of the unified verify / LLM-boundary pipeline
  ADR-042: Pre-dates ADR-042; this feature does not define TC types or validate the type vocabulary. FT-048 owns the mechanics.
  ADR-041: Pre-dates ADR-041; this feature does not author absence TCs or set removes/deprecates on ADRs. FT-047 owns the mechanics.
  ADR-018: Predates the 2026-04-22 scope promotion of ADR-018 to cross-cutting. Test coverage reflects the property/session/benchmark strategy as it existed when this feature shipped; not retroactively reclassified.
  ADR-048: Predates ADR-048; this feature does not author the canonical .product/ folder layout. FT-057 owns the migration command and discovery fallback.
  ADR-047: Predates ADR-047; this feature does not author the functional-spec body convention. FT-055 owns the structural validator and W030 mechanics.
---

## Description

Enforce structural quality rules with measurable thresholds (ADR-029): file size limits (400 lines hard, 300 warning), function length limits (40 statement lines), mandatory module decomposition, and single-responsibility doc comments on every source file. Checked by shell scripts in `scripts/checks/` and run via `product verify --platform`.

---

## Functional Specification

### Inputs

- Source files in `src/**/*.rs` (the 400-line and 300-line file length checks and single-responsibility doc comment rule apply to this tree; `tests/` and `benches/` are exempt).
- Environment variables `FILE_LENGTH_HARD` (default 400) and `FILE_LENGTH_WARN` (default 300) and `FN_LENGTH_HARD` (default 40) and `FN_LENGTH_WARN` (default 30) may override defaults.
- Invocation of `product verify --platform`, which triggers the bash enforcement scripts.

### Outputs

- **Exit 0** from each script when all files pass their respective check.
- **Exit 2** (warning) when files exceed the warning threshold but not the hard limit (file-length only).
- **Exit 1** (error) when any file violates the hard limit, when a function exceeds 40 statement lines, or when a file lacks a valid single-responsibility doc comment.
- Human-readable diagnostic lines on stdout identifying the offending file, line count, and limit that was breached.

### State

Stateless. The scripts re-scan `src/` on every invocation; no results are cached or persisted between runs.

### Behaviour

Four quality rules are checked by separate shell scripts in `scripts/checks/`:

1. **File length (`scripts/checks/file-length.sh`)** — `find src -name "*.rs" | xargs wc -l` compares line counts against `FILE_LENGTH_HARD` (400) and `FILE_LENGTH_WARN` (300). Hard violations exit 1; warning-only violations exit 2; clean pass exits 0.

2. **Function length (`scripts/checks/function-length.sh`)** — uses `awk` to track brace depth and count non-blank statement lines within each `fn` block in every `.rs` file. Functions exceeding `FN_LENGTH_HARD` (40) are errors; those exceeding `FN_LENGTH_WARN` (30) are warnings.

3. **Module structure (`scripts/checks/module-structure.sh`)** — verifies that the required top-level module directories exist under `src/` (e.g. `graph/`, `context/`, `commands/`, `verify/`, `mcp/`, `io/`, `parse/`) and that `src/main.rs` does not exceed 80 lines.

4. **Single-responsibility doc comments (`scripts/checks/single-responsibility.sh`)** — every `.rs` file except `mod.rs` and `main.rs` must begin with a `//!` line that does not contain the word "and". Violation exits 1.

All four scripts are invoked by `product verify --platform` through TCs with `runner: bash`. The file-length warning threshold is tested separately (TC-370) by setting `FILE_LENGTH_HARD=99999` to disable the hard limit while keeping `FILE_LENGTH_WARN=300` active, exploiting the three-tier exit code model.

### Invariants

- No Rust source file in `src/` may exceed 400 lines (including blank lines and comments).
- No function body in `src/` may exceed 40 statement lines (blank lines excluded from count).
- Every `src/**/*.rs` file except `mod.rs` and `main.rs` must have a `//!` doc comment as its first line, and that comment must not contain the word "and".
- `src/main.rs` must not exceed 80 lines.
- Required top-level module directories must exist.

### Error handling

- Each script writes a human-readable error or warning line to stdout naming the offending file, the actual value, and the limit.
- Exit codes follow the ADR-009 three-tier model: 0 (pass), 2 (warning), 1 (error/hard violation).
- When `FILE_LENGTH_HARD` or `FILE_LENGTH_WARN` environment variables are absent, the scripts use their built-in defaults — they never fail due to missing environment variables.

### Boundaries

- Applies only to `src/**/*.rs`. Integration test files (`tests/`), benchmarks (`benches/`), and non-Rust files are not checked.
- Does not check logical correctness, API design, or Rust idiom adherence — those are covered by `cargo clippy` (ADR-001).
- Does not modify source files; read-only analysis only.

## Out of scope

- Rust compilation quality and `clippy::unwrap_used` enforcement — governed by ADR-001, enforced by `cargo clippy`.
- Test file length limits — `tests/` and `benches/` are explicitly exempt.
- Automatic refactoring or code splitting — the scripts report violations; fixing them is the developer's responsibility.
- Runtime or dynamic code quality checks — all checks are static analysis of source text.
