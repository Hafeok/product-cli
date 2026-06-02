---
id: FT-107
title: Split product-cli into product-core, product-mcp, product-cli workspace
phase: 6
status: complete
depends-on: []
adrs:
- ADR-001
- ADR-013
- ADR-018
- ADR-020
- ADR-029
- ADR-043
- ADR-048
tests:
- TC-885
- TC-886
- TC-887
- TC-888
- TC-889
- TC-890
domains:
- api
- testing
domains-acknowledged:
  ADR-051: All FT-107 TCs declare `observes:` and assert on the named surface.
  ADR-042: TCs use existing reserved types (scenario, invariant); no new TC type is introduced.
  ADR-041: FT-107 is additive — no CLI surface, MCP tool, or schema is removed or deprecated.
  ADR-047: Functional specification lives in the body of this feature, not in a separate artifact.
  ADR-050: PAT-001 is already linked via `patterns:` — slice + adapter is the enabling pattern.
  ADR-040: FT-107 is a pure code reorganisation; the verify pipeline is unchanged.
  ADR-049: Context bundle assembly is unchanged; no per-model template touched.
patterns:
- PAT-001
---

## Description

Convert this repo from a single `product` crate into a Cargo workspace
with three published members so the domain logic can be reused by
sibling CLIs (notably `decision-cli`) without inheriting `clap`,
`clap_complete`, `axum`, or `tower-http` as dead weight.

- **`product-core`** — every slice (`feature/`, `adr/`, `tc/`,
  `status/`, `gap/`, `drift/`, `request/`, `graph/`, `agent_context/`,
  `implement/`, `verify/`, `parser`, `types`, `config*`, `fileops`,
  `error`, `rdf`, etc.). No `clap`, no `axum`, no `tower-http`. The
  reusable surface.
- **`product-mcp`** — `src/mcp/` plus its `axum` / `tower-http` /
  `tokio` runtime. Depends on `product-core`. Independently
  consumable by future tools that want the MCP server without the
  Clap CLI.
- **`product-cli`** — `src/commands/`, `src/main.rs`, `src/root.rs`,
  the `[[bin]] name = "product"`. Depends on `product-core` and
  `product-mcp`. The Cargo-dist release pipeline continues to ship
  this binary.

The pre-work for this feature is already done: the codebase is built
on PAT-001 (slice + adapter) so the layering is enforced rather than
aspirational. Grep verification on the current `main` (4bfd6db) found
zero reverse-coupling: nothing outside `src/commands/` imports from
`src/commands/`, `src/mcp/` does not import from `src/commands/`,
and no slice imports from `src/mcp/`. The only `tokio` use outside
`src/mcp/` is one `Runtime::new()` call inside `commands/mcp_cmd.rs`,
which moves with the CLI adapter.

## Why now

`decision-cli` is being built as a sibling project that needs the
full graph/spec engine but not the `product` subcommand tree.
Without this split, the only options are:

- Vendor the source into `decision-cli` — duplicates 30k LoC and
  forks bug fixes immediately.
- Depend on the current `product` crate — inherits `clap`,
  `clap_complete`, `axum`, `tower-http`, and a `[[bin]]` target
  `decision-cli` does not want. Bloats compile time and the
  dependency surface for a downstream that only needs the library.
- Wait — but every new feature that lands in `product-cli` until then
  is one more file that has to be reassigned to a crate later. The
  cost only grows with time.

The split was implicit in ADR-043 (slice + adapter) from the day it
landed. This feature makes it explicit at the crate boundary so the
compiler enforces it.

## Functional Specification

### Inputs

- The current single-crate `Cargo.toml` at the repo root, including
  its `[workspace]` block that already lists `xtask`.
- The current `src/` tree (every file moves; nothing is deleted).
- The current `tests/` tree: `code_quality_tests.rs`,
  `integration_tests.rs`, `integration/`, `property_tests.rs`,
  `property/`, `sessions.rs`, `sessions/`, `fixtures/`.
- The current `benches/graph_bench.rs`.
- `rust-toolchain.toml` (workspace-wide; no change needed).
- `.cargo/config.toml` (the `t = "test --no-fail-fast"` alias; no
  change needed — alias runs at the workspace root).

### Outputs

- A workspace `Cargo.toml` at the repo root declaring members
  `["product-core", "product-mcp", "product-cli", "xtask"]`.
- Three new crate directories at the repo root:
  `product-core/`, `product-mcp/`, `product-cli/`, each with its
  own `Cargo.toml` and `src/` subtree.
- The `product` binary still built from `product-cli` —
  `cargo build` at the workspace root produces
  `target/debug/product` at the same path cargo-dist expects.
- All six existing test binaries still discoverable by `cargo t`:
  `--lib` (per crate), `--doc`, `--test code_quality_tests`,
  `--test integration_tests`, `--test property_tests`,
  `--test sessions`.

### State

- No new on-disk state. `.product/`, `docs/`, and `CHECKLIST.md`
  are untouched.
- No new files are created in `docs/` for this feature beyond
  the feature spec and TCs.
- The current `[workspace.lints.clippy] unwrap_used = "deny"` block
  remains at the workspace root and continues to apply to every
  member via `[lints] workspace = true`.

### Behaviour

- F1 — **Crate split.** Three workspace members are created and
  the existing `src/` tree is moved into them by their architectural
  layer (see "Crate boundaries" below). No code is rewritten; this
  is a `git mv` plus `use crate::` → `use product_core::` /
  `use product_mcp::` rewrite where imports cross a crate
  boundary.
- F2 — **Per-crate dependency lists.** `product-core/Cargo.toml`
  drops `clap`, `clap_complete`, `axum`, `tower-http`. It keeps
  `serde`, `serde_yaml`, `serde_json`, `toml`, `oxigraph`, `uuid`,
  `chrono`, `regex`, `sha2`, `fd-lock`, `libc`, `notify`.
  `product-mcp/Cargo.toml` adds `axum`, `tower-http`, `tokio` and
  depends on `product-core`. `product-cli/Cargo.toml` adds `clap`,
  `clap_complete` and depends on both `product-core` and
  `product-mcp`. `tokio` stays in `product-mcp`; the one
  `Runtime::new()` call in `commands/mcp_cmd.rs` calls through a
  re-exported helper (`product_mcp::serve_blocking`).
- F3 — **Import rewrite.** Every `use crate::X` that now crosses a
  crate boundary becomes `use product_core::X` or
  `use product_mcp::X`. Imports that stay within the same crate
  remain `use crate::X`. The integration tests in `tests/integration/`
  and `tests/sessions/` keep their `use product_lib::X` lines but
  the lib alias points at `product-core` (renamed from `product_lib`
  to `product_core`, with `[lib] name = "product_core"`). The
  `product_lib` name is dropped — there is no published `product_lib`
  alias after this feature.
- F4 — **Test reorganisation.** Pure unit tests (`#[cfg(test)] mod
  tests`) move with their slice into `product-core`. Doc tests move
  with their host crate. `tests/property_tests.rs` and
  `tests/property/` move to `product-core/tests/`. `benches/` moves
  to `product-core/benches/`. `tests/code_quality_tests.rs` moves
  to `product-cli/tests/` and is generalised to walk every member
  crate's `src/` directory (not just `src/`). `tests/integration_tests.rs`,
  `tests/integration/`, `tests/sessions.rs`, `tests/sessions/`, and
  `tests/fixtures/` stay in `product-cli/tests/` (they drive the
  `product` binary via `assert_cmd`).
- F5 — **Build and test parity.** After the split, at the
  workspace root: `cargo build` succeeds; `cargo t` runs all six
  existing test binaries (≈820 tests) with zero new failures;
  `cargo clippy --workspace -- -D warnings -D clippy::unwrap_used`
  passes; `cargo bench --workspace --no-run` builds the graph
  benchmark.
- F6 — **External-consumer smoke test.** A small fixture crate
  under `tests/fixtures/external-core-consumer/` depends on
  `product-core` via `path = "../../../product-core"` and exercises
  the public API (`KnowledgeGraph::load`, `Feature` access,
  `fileops::write_file_atomic`). TC-889 builds and runs this
  fixture to prove the lib is consumable without the CLI or MCP
  crates pulled in.
- F7 — **CLAUDE.md project structure section** is updated to
  describe the new workspace layout: the three member crates, what
  lives where, and a one-line consumer recipe
  (`product-core = { path = "..." }` for sibling repos).

### Invariants

- The `product` binary's CLI surface is byte-identical
  (`product --help` output, every subcommand's flags and exit
  codes, every error message verbatim). This is the strongest
  invariant; integration tests in `tests/integration/` are the
  oracle.
- The MCP tool surface is byte-identical (tool list, response
  envelopes, error codes). The `tests/sessions/` MCP tests are the
  oracle.
- Front-matter schemas, `.product/` layout, `CHECKLIST.md`
  generation, request-log format, and all on-disk artifacts are
  byte-identical before and after the split.
- `product-core` has zero dependency on `clap`, `clap_complete`,
  `axum`, or `tower-http`. Verified by parsing
  `cargo metadata --filter-platform <host> --format-version 1
  --no-deps` for the `product-core` package and asserting these
  names do not appear in `dependencies[]`. TC-885 is the oracle.
- `product-mcp` has zero dependency on `clap` or `clap_complete`.
  Verified the same way. TC-886 is the oracle.
- Workspace-level clippy lint `unwrap_used = "deny"` continues to
  fail any new `.unwrap()` in any member crate.
- Code-quality fitness tests (400-line file limit, SRP first-line
  doc check) run against every member crate's `src/`, not just the
  old `src/`. TC-890 is the oracle.

### Error handling

- **Pre-existing graph error (E001 in TC-768) is not blocked by
  this feature.** That error predates FT-107 and lives in an
  unrelated TC body; it is acknowledged here and explicitly out of
  scope (see "Out of scope").
- A test failure during the split is treated as a stop-the-line
  signal: revert the offending move, restore the previous import,
  and re-run `cargo t` before continuing. The split is a single
  logical unit of work and lands as a single commit (or an explicit
  multi-commit series where each commit compiles and tests
  green).
- A clippy violation introduced by the move (e.g. a `pub use` that
  was previously crate-private and now exposes a non-`pub` type)
  is fixed in place by promoting the affected type to `pub`. No
  `#[allow(...)]` escapes.

### Boundaries

- Crate names are exactly `product-core`, `product-mcp`,
  `product-cli`. Library aliases are `product_core` and
  `product_mcp`. The binary target stays `name = "product"`.
- Path-based workspace dependencies only. No version pinning, no
  `crates.io` publishing in this feature.
- `xtask` stays exactly as it is (already a workspace member).
- The cargo-dist release pipeline is **not** modified in this
  feature beyond whatever path adjustment is needed for the
  binary's manifest location. The shipped artifact is still
  `product` (a single binary).

## Crate boundaries — file-level move plan

| Current path                        | New crate     | New path                                |
|-------------------------------------|---------------|------------------------------------------|
| `src/main.rs`                       | product-cli   | `product-cli/src/main.rs`                |
| `src/root.rs`                       | product-cli   | `product-cli/src/root.rs`                |
| `src/commands/**`                   | product-cli   | `product-cli/src/commands/**`            |
| `src/mcp/**`                        | product-mcp   | `product-mcp/src/**` (re-rooted)         |
| `src/lib.rs`                        | product-core  | `product-core/src/lib.rs` (trimmed)      |
| every other `src/*.rs` / `src/*/`   | product-core  | `product-core/src/...`                   |
| `tests/integration_tests.rs`        | product-cli   | `product-cli/tests/integration_tests.rs` |
| `tests/integration/**`              | product-cli   | `product-cli/tests/integration/**`       |
| `tests/sessions.rs`                 | product-cli   | `product-cli/tests/sessions.rs`          |
| `tests/sessions/**`                 | product-cli   | `product-cli/tests/sessions/**`          |
| `tests/fixtures/**`                 | product-cli   | `product-cli/tests/fixtures/**`          |
| `tests/code_quality_tests.rs`       | product-cli   | `product-cli/tests/code_quality_tests.rs` (generalised) |
| `tests/property_tests.rs`           | product-core  | `product-core/tests/property_tests.rs`   |
| `tests/property/**`                 | product-core  | `product-core/tests/property/**`         |
| `benches/graph_bench.rs`            | product-core  | `product-core/benches/graph_bench.rs`    |

## Out of scope

- **Renaming the binary.** `product` stays `product`. A future
  feature can rename it if a sibling decision-cli causes a
  collision.
- **Renaming types.** `KnowledgeGraph`, `Feature`, `Adr`,
  `TestCriterion`, `ProductError`, etc. keep their names. A
  downstream `decision-cli` reuses these as-is.
- **Publishing to crates.io.** Path dependencies only. The
  cargo-dist binary release continues; library publication is a
  separate decision.
- **Feature-gating `oxigraph` or `notify` behind cargo features.**
  Worth doing for compile time once a real downstream demands it,
  but premature here — pin the split first, optimise later.
- **Fixing the pre-existing E001 graph error in TC-768.** Unrelated
  to this feature.
- **Updating `decision-cli`.** The sibling repo is out of this
  feature's tree. This feature only makes its consumption possible;
  the consumer-side wiring lands in `decision-cli`.

## Exit criteria

- `cargo build` at the workspace root succeeds; the `product`
  binary appears at `target/debug/product`.
- `cargo t` at the workspace root runs the full suite
  (≈820 tests across all binaries) with zero new failures.
- `cargo clippy --workspace -- -D warnings -D clippy::unwrap_used`
  is clean.
- `cargo metadata` for `product-core` shows no `clap`,
  `clap_complete`, `axum`, or `tower-http` in `dependencies[]`
  (TC-885).
- `cargo metadata` for `product-mcp` shows no `clap` or
  `clap_complete` in `dependencies[]` (TC-886).
- `product --help` output is byte-identical to the pre-split
  output captured in the TC-887 fixture.
- The external-consumer fixture at
  `tests/fixtures/external-core-consumer/` builds and its smoke
  test executes (TC-889).
- The 400-line and SRP fitness gates run over every member crate's
  `src/` directory and produce the same pass/fail set as today
  (TC-890).
- CLAUDE.md `## Project Structure` section reflects the workspace
  layout.
