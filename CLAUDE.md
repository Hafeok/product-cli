# CLAUDE.md — Product CLI

## What is this project?

Product is a Rust CLI and MCP server that manages a file-based knowledge graph of features (FT-XXX), architectural decisions (ADR-XXX), and test criteria (TC-XXX). It assembles precise LLM context bundles from the graph and orchestrates the full spec-to-implementation loop.

## Build & Test

```bash
cargo build                                          # compile
cargo t                                              # full suite, runs every binary (alias in .cargo/config.toml)
cargo clippy -- -D warnings -D clippy::unwrap_used   # lint (zero unwrap policy)
cargo bench                                          # 4 benchmarks
```

**Always run `cargo t`, never plain `cargo test`.** Plain `cargo test` stops at
the first failing binary and silently skips subsequent suites — so a failure
in the code-quality fitness tests hides regressions in integration, sessions,
and property tests. The `t` alias is defined as `test --no-fail-fast` in
`.cargo/config.toml`: it runs every test binary and reports the complete
result set at the end.

Suite composition (six binaries, ~820 tests, ~14s wall-clock):

| Binary | Tests | What it covers |
|---|---|---|
| `cargo test --lib` | 253 | Unit tests on pure functions (in `#[cfg(test)] mod tests`) |
| `--doc` | 6 | Doc tests in `///` examples |
| `--test code_quality_tests` | 13 | File length ≤ 400, SRP in doc comments |
| `--test integration_tests` | 448 | `assert_cmd`-driven CLI scenarios |
| `--test property_tests` | 13 | `proptest`, 1000 cases each |
| `--test sessions` | 94 | Session-based integration (ADR-018 Design 2) |

All three gates (build, `cargo t`, clippy) must pass before any commit.
Code-quality fitness tests (`tests/code_quality_tests.rs`) enforce a
400-line-per-file hard limit and a single-responsibility check on module doc
comments (the first `//!` line must not contain the word "and").

### Rust toolchain

The toolchain is pinned in `rust-toolchain.toml` at the repo root. `rustup`
reads this automatically, so `cargo` / `cargo clippy` always run on the
pinned version locally. CI (`dtolnay/rust-toolchain@master`) reads the same
file, so local and CI stay in lockstep. To upgrade, bump the `channel`
value in `rust-toolchain.toml` — no workflow change needed.

## Project Structure

```
src/
  main.rs             # Clap entry point — 42 lines, delegates to commands::run
  lib.rs              # Module re-exports for tests and library consumers
  commands/           # CLI adapter layer — one file per subcommand family
    mod.rs            # Subcommand enum + dispatch match
    shared.rs         # load_graph/acquire_write_lock helpers (typed + boxed)
    output.rs         # Output enum, CmdResult, render_result bridge
    feature.rs        # Feature navigation (list, show, next)
    feature_write.rs  # Feature write adapters → call product_lib::feature
    status.rs         # Status/impact adapters → call product_lib::status
    ...
  feature/            # Feature domain slice — pure plan_* + thin apply_*
  status/             # Status domain slice — pure build_* + render_*
  gap/                # Gap analysis
  drift/              # Drift detection
  request/            # Unified atomic write interface
  implement/          # implement + verify pipeline orchestration
  graph/              # Knowledge graph + algorithms (centrality, BFS, topo)
  mcp/                # MCP server (stdio + HTTP via axum)
  types.rs            # Core artifact types (Feature, Adr, TestCriterion)
  parser.rs           # YAML front-matter parser
  config.rs           # product.toml parsing
  fileops.rs          # Atomic writes + advisory locking
  error.rs            # Error model (ProductError enum, exit codes)
docs/
  product-prd.md     # Full PRD
  product-adrs.md    # All ADRs in one file
  adrs/              # Individual ADR files (26 ADRs)
  features/          # Individual feature files (FT-XXX-*.md)
  tests/             # Individual TC files (100+)
  guide/             # Generated Diátaxis docs per feature (FT-XXX-*.md)
scripts/
  generate-docs.sh   # Spawns claude -p per feature to generate docs/guide/ files
product.toml         # Repo config (paths, prefixes, thresholds)
CHECKLIST.md         # Auto-generated feature checklist (tracks [x]/[T]/[ ] status)
```

## Implementation Workflow

Use the `product` CLI (or MCP tools) to stay in sync with the knowledge graph.

**If using `product implement FT-XXX`** — the pipeline assembles the context bundle and passes it to the spawned agent automatically. Do not also run `product context` — that would duplicate the context.

**If implementing manually** (without `product implement`):

1. **Get context** — run `product context FT-XXX --depth 2` to get the full bundle (linked ADRs + test criteria)
2. **Check decisions** — run `product impact ADR-XXX` to understand what a change affects before modifying behavior

**Always, regardless of path:**

- **Configure TC runners** — before verifying, ensure every TC linked to the feature has `runner: cargo-test` and `runner-args: "tc_XXX_snake_case_name"` in its front-matter (see "TC Runner Configuration" below). Without these fields, `product verify` silently skips the TC.
- **Verify work** — run `product verify FT-XXX` after implementation to execute TC runners and update test status in front-matter
- **Mark done** — when all TCs pass, `product verify` auto-updates feature status to complete and regenerates `CHECKLIST.md`
- **Check health** — run `product gap check` and `product drift check` to catch specification issues before committing

Do not manually edit feature status or CHECKLIST.md — let the CLI manage that through `verify` and `checklist generate`.

## Key Conventions

- **No unwrap**: `#![deny(clippy::unwrap_used)]` — use `?`, `.ok_or()`, `.unwrap_or_default()`, or match
- **Error model**: All errors go through `ProductError` in `error.rs` — each variant maps to a specific exit code
- **Atomic writes**: File writes use `fileops::atomic_write()` with advisory locking
- **Graph is derived**: No persistent graph store. Graph is rebuilt from YAML front-matter on every invocation (ADR-003)
- **CHECKLIST.md is generated**: Never hand-edit. Run `product checklist generate` or it regenerates after `product verify`
- **Front-matter is source of truth**: All artifact identity and relationships declared in YAML front-matter (ADR-002)
- **IDs**: Features=FT-XXX, ADRs=ADR-XXX, Tests=TC-XXX (ADR-005)
- **Test types**: scenario, invariant, chaos, exit-criteria (ADR-011)

## Architecture Pattern — Slice + Adapter

The codebase is organised as vertical slices, each with a pure domain module
in the library and a thin CLI adapter in `src/commands/`. This separation
keeps business logic unit-testable without tempdirs, print capture, or
`cargo run`.

**Slice modules (`src/<slice>/`)** — pure, testable:
- `plan_*` / `build_*` functions take current state + user input, return a
  struct describing the intended change. No I/O, no println, no exit.
- `apply_*` functions take a plan struct and perform the minimal I/O
  (`fileops::write_file_atomic`, `write_batch_atomic`) needed to commit it.
- `render_*` functions turn result structs into text strings. JSON rendering
  is derived from `serde::Serialize` on the plan / result types.
- Unit tests (`src/<slice>/tests.rs`) exercise the pure functions directly.

Reference slices:
- `src/feature/` — create, status change (with ADR-010 cascade), domain edit
- `src/adr/` — create, status change, domain/scope/source-files edits,
  supersession (bidirectional + cycle detection), amend, seal, conflicts
- `src/tc/` — create, status change, runner config
- `src/status/` — project summary, untested/failing feature lists
- `src/request/` — the unified atomic-write pipeline (pre-existing)

**Command adapters (`src/commands/<cmd>.rs`)** — thin:
- Return `CmdResult = Result<Output, ProductError>` (not `BoxResult`).
- Load graph via `shared::load_graph_typed()`, acquire lock via
  `shared::acquire_write_lock_typed()` when writing.
- Call the slice's `plan_*` + `apply_*` (for writes) or `build_*` +
  `render_*_text` (for reads), then wrap in `Output::text(...)` /
  `Output::both { text, json }`. Never call `println!`.
- Wire into `dispatch()` in `commands/mod.rs` via `render(...)` which
  handles the format flag and error conversion.

**Handlers that remain on `BoxResult`** are not legacy — they're intentional.
Keep them that way when:
- The handler prints continuous progress during a long operation
  (`implement`, `author`, `init`, `onboard`, `migrate`, `mcp`).
- The handler has exit-code semantics that `CmdResult` cannot express, such
  as exit 2 for "warning-only" states (`dep check`, `preflight`).
- The handler is an interactive flow that reads stdin mid-computation
  (`feature link` with TC-inference prompts, `feature acknowledge`).
- The handler is a trivial wrapper (`completions`, `hooks`, `schema`) where
  `Output::Empty` wrapping is pure churn.

Migrate a `BoxResult` handler only when you have a reason: adding JSON
parity, extracting a pure function for unit testing, or fixing a bug where
the pure/I/O split makes the fix cleaner.

## Adding a New Command

1. Add the clap subcommand in `src/commands/<cmd>.rs` and re-export from
   `commands/mod.rs`.
2. If the command has non-trivial logic, create a slice at `src/<cmd>/`
   following the Slice + Adapter pattern above.
3. Implement the handler as a thin adapter returning `CmdResult`.
4. Wire into the match block in `dispatch()` via `render(...)`.
5. Add unit tests on the pure slice functions (in `src/<cmd>/tests.rs`).
6. Add integration tests in `tests/integration/` with `assert_cmd`.
7. Create TC-XXX doc in `docs/tests/` if the feature has a formal test
   criterion. **Add runner config to every TC** — see section below.

## TC Runner Configuration

Every TC that has an integration test **must** include `runner` and `runner-args` in its YAML front-matter, otherwise `product verify` will skip it. When writing a new TC or implementing a feature with existing TCs, always add these fields:

```yaml
---
id: TC-054
title: product impact ADR-001
type: scenario
status: passing
validates:
  features:
  - FT-011
  adrs:
  - ADR-012
phase: 1
runner: cargo-test
runner-args: "tc_054_product_impact_adr_001"
---
```

Rules:
- `runner: cargo-test` — use this for all integration tests
- `runner-args` — the integration test function name, formatted as `tc_XXX_snake_case_title` (derived from the TC id and title)
- The `runner-args` value must match the `#[test] fn` name in `tests/integration.rs` exactly
- Add runner fields **at the same time** you write the integration test — never leave a TC without runner config if it has a test

## Adding a New Module

1. Create `src/foo.rs` (or `src/foo/mod.rs` for a multi-file slice)
2. Add `pub mod foo;` in `src/lib.rs`
3. Consume from command adapters via `use product_lib::foo;`
4. Keep the first `//!` doc line free of the word "and" (SRP fitness test)
5. Keep every file under 400 lines (file-length fitness test)

## Test Organization

- **Unit tests**: `#[cfg(test)] mod tests` at bottom of each source file
- **Integration tests**: `tests/integration.rs` using `assert_cmd` + temp fixtures
- **Property tests**: `tests/property.rs` using `proptest`
- **Benchmarks**: `benches/graph_bench.rs`

## Documentation System

### Specification docs (source of truth)

- **PRD**: `docs/product-prd.md` — the source of truth for what to build
- **ADRs**: `docs/adrs/ADR-XXX-*.md` — one file per decision, with YAML front-matter
- **Features**: `docs/features/FT-XXX-*.md` — one file per feature, with YAML front-matter
- **Test Criteria**: `docs/tests/TC-XXX-*.md` — one file per test criterion
- **ADR index**: `docs/product-adrs.md` — all ADRs collected in one file for reference

### User-facing docs — Diátaxis framework (https://diataxis.fr/)

Generated per-feature guides live in `docs/guide/FT-XXX-*.md`. Each guide follows the Diátaxis framework, which organises documentation into four modes along two axes (action vs. knowledge, learning vs. working):

| Mode | Serves | Section heading | What it contains |
|------|--------|-----------------|------------------|
| **Tutorial** | Learning + action | `## Tutorial` | Step-by-step lessons that take a newcomer through a concrete experience. Learning-oriented. |
| **How-to guide** | Working + action | `## How-to Guide` | Task-oriented recipes that solve a specific problem. Goal-oriented. |
| **Reference** | Working + knowledge | `## Reference` | Exact CLI syntax, flags, output formats, configuration. Information-oriented. |
| **Explanation** | Learning + knowledge | `## Explanation` | Design decisions, trade-offs, architecture context. Understanding-oriented. |

Each guide also starts with `## Overview` (one paragraph on what the feature is and why it exists).

Guide files must **not** contain YAML front-matter (`---` blocks). The knowledge graph parser only scans `docs/features/`, `docs/adrs/`, and `docs/tests/` (configured in `product.toml`), but omitting front-matter from guides avoids accidental collisions if scan paths change.

Regenerate guides with `scripts/generate-docs.sh`. The script assembles a context bundle per feature via the product CLI and spawns `claude -p` to write each file. Files with ≥20 lines are skipped on re-runs.

## Dependencies

Key crates: clap (CLI), serde/serde_yaml/serde_json/toml (serialization), oxigraph (SPARQL), axum/tokio (HTTP server), sha2 (hashing), fd-lock (file locking), chrono (dates), regex, uuid.

Dev: tempfile, assert_cmd, predicates, proptest.
