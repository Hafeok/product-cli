# CLAUDE.md — Product CLI

## What is this project?

Product is a Rust CLI and MCP server that manages a file-based knowledge graph of features (FT-XXX), architectural decisions (ADR-XXX), and test criteria (TC-XXX). It assembles precise LLM context bundles from the graph and orchestrates the full spec-to-implementation loop.

## Build & Test

```bash
cargo build                                          # compile
cargo test                                           # all tests (83 unit + 32 integration + 9 property)
cargo clippy -- -D warnings -D clippy::unwrap_used   # lint (zero unwrap policy)
cargo bench                                          # 4 benchmarks
```

All three (build, test, clippy) must pass before any commit.

## Project Structure

```
src/
  main.rs        # CLI entry point + all command handlers (~1700 lines)
  lib.rs         # Module re-exports
  types.rs       # Core artifact types (Feature, Adr, TestCriterion)
  parser.rs      # YAML front-matter parser
  config.rs      # product.toml parsing
  graph.rs       # Knowledge graph + algorithms (centrality, BFS, topo sort)
  context.rs     # Context bundle assembly
  rdf.rs         # TTL export + SPARQL queries (Oxigraph)
  formal.rs      # AISP formal block parser
  gap.rs         # Specification gap analysis
  drift.rs       # Spec-vs-code drift detection
  metrics.rs     # Architectural fitness functions
  implement.rs   # implement + verify pipeline (agent orchestration)
  author.rs      # Authoring sessions
  mcp.rs         # MCP server (stdio + HTTP via axum)
  migrate.rs     # PRD/ADR document migration
  fileops.rs     # Atomic writes + advisory locking
  checklist.rs   # Checklist generation
  domains.rs     # ADR concern domain classification
  error.rs       # Error model (ProductError enum, exit codes)
docs/
  product-prd.md     # Full PRD
  product-adrs.md    # All ADRs in one file
  adrs/              # Individual ADR files (26 ADRs)
  tests/             # Individual TC files (100+)
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

## Adding a New Command

1. Add the clap subcommand in `main.rs` (Commands enum or sub-enum)
2. Add the handler function in `main.rs` (or a new module if >200 lines)
3. Wire up in the match block in `main()`
4. Add unit tests in the same module
5. Add integration tests in `tests/integration.rs`
6. Create TC-XXX doc in `docs/tests/` if the feature has a formal test criterion
7. **Add runner config to every TC** — see "TC Runner Configuration" below

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

1. Create `src/foo.rs`
2. Add `pub mod foo;` in `src/lib.rs`
3. Add `use product_lib::foo;` in `src/main.rs` as needed

## Test Organization

- **Unit tests**: `#[cfg(test)] mod tests` at bottom of each source file
- **Integration tests**: `tests/integration.rs` using `assert_cmd` + temp fixtures
- **Property tests**: `tests/property.rs` using `proptest`
- **Benchmarks**: `benches/graph_bench.rs`

## Documentation System

- **PRD**: `docs/product-prd.md` — the source of truth for what to build
- **ADRs**: `docs/adrs/ADR-XXX-*.md` — one file per decision, with YAML front-matter
- **Test Criteria**: `docs/tests/TC-XXX-*.md` — one file per test criterion
- **ADR index**: `docs/product-adrs.md` — all ADRs collected in one file for reference

## Dependencies

Key crates: clap (CLI), serde/serde_yaml/serde_json/toml (serialization), oxigraph (SPARQL), axum/tokio (HTTP server), sha2 (hashing), fd-lock (file locking), chrono (dates), regex, uuid.

Dev: tempfile, assert_cmd, predicates, proptest.
