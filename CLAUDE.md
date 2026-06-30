# CLAUDE.md — Product CLI

## What is this project?

Product is a Rust CLI and MCP server for the **Product Framework** — the open
What/How specification graph (`docs/product-framework-open.md`). It captures and
verifies a product's *What* (domain model, event model, Deciders, Projectors,
systems, triggers, UI/AIO model) and *How* (contracts, reification, delivery),
all under `.product/`. The reference What lives in
`.product/author-domain/product-cli/`.

> **Graph-only.** This repo was pivoted to the framework graph alone. The former
> FT/ADR/TC knowledge-graph tool (the `feature`/`adr`/`test`/`gap`/`drift`/
> `conformance`/`implement`/`verify` commands, `docs/{features,adrs,tests}`, and
> the meta-graph engine) has been removed. Don't reach for those commands or dirs.

## Build & Test

```bash
cargo build                                          # compile
cargo t                                              # full suite, runs every binary (alias in .cargo/config.toml)
cargo clippy -- -D warnings -D clippy::unwrap_used   # lint (zero unwrap policy)
```

**Always run `cargo t`, never plain `cargo test`.** Plain `cargo test` stops at
the first failing binary and skips the rest. The `t` alias is `test
--no-fail-fast` (`.cargo/config.toml`): it runs every test binary and reports
the complete result set at the end.

Suite composition (~420 tests):

| Binary | What it covers |
|---|---|
| `cargo test -p product-core --lib` | pf unit tests (the framework graph, in `#[cfg(test)] mod tests`) |
| `cargo test -p product-mcp` | MCP registry + framework tool handlers |
| `--test framework` | `assert_cmd`-driven framework CLI scenarios (`tests/framework.rs`) |
| `--test code_quality_tests` | fitness gates: file length ≤ 400, function length, SRP, module structure |
| `product-core/tests/property` | `proptest` over fileops/init |

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

This repo is a Cargo workspace with three publishable members and one
internal `xtask` helper (FT-107).

```
Cargo.toml           # Workspace manifest — [workspace.members] + clippy lints
product-core/        # Pure library: the `pf/` framework graph (domain model,
  Cargo.toml         #   event model, Deciders, Projectors, systems, triggers,
  src/lib.rs         #   UI/AIO model, How contract, validate/turtle/seed/rules),
  src/pf/            #   plus guide, demo, error, fileops, parse, io, config.
  src/pf/<slice>/    #   NO clap / axum / tower-http. `[lib] name = "product_core"`
  tests/property/    #   proptest over fileops/init
product-mcp/         # MCP server (stdio + HTTP via axum). Depends on
  Cargo.toml         #   product-core. Re-exports `ToolRegistry`,
  src/lib.rs         #   `run_stdio`, `run_http`, `serve_http_blocking`.
  src/...            #   framework tool handlers (domain/decider/projector/…)
product-cli/         # The `product` binary. Depends on product-core +
  src/main.rs        #   product-mcp. Owns clap, the commands/ adapter layer,
  src/commands/      #   and the framework integration tests.
    mod.rs           #     Subcommand enum + dispatch
    domain.rs        #     What-graph CRUD + `validate [--strict]`
    decider.rs       #     §3.3 Decider derive/validate/simulate
    projector.rs     #     §3.4 Projector derive/validate
    build.rs · how.rs · slice.rs · seam.rs · preview.rs · …
    target.rs        #     §7.3 target version + computed direction/gap
    verdict.rs       #     §5.1 build-seam verdict-event validation
  tests/
    framework.rs            # assert_cmd-driven framework CLI scenarios (~44)
    code_quality_tests.rs   # fitness gates (walk every member src/)
xtask/                # Workspace convention enforcement (`cargo xtask check`)
docs/
  product-framework-open.md   # The open framework spec (What/How/Delivery)
  two-pillars-conformance.md  # The conformance clause set
  examples/ · workshop/       # Worked examples + workshop runbook
.product/
  config.toml          # Repo config
  author-domain/       # The captured What graphs (e.g. product-cli — the example What)
  deciders/ · slices/ · work-units/ · deliverables/ · archetypes/
```

**Downstream consumers** (e.g. `decision-cli`) should add only:

```toml
[dependencies]
product-core = { path = "../product-cli/product-core" }   # or a git rev
```

This buys the `pf/` framework graph and `ProductError` without
pulling in `clap`, `axum`, `tower-http`, or the `product` binary.

## Working with the framework graph

Use the `product` CLI (or MCP tools) to author and verify a What/How graph under
`.product/`:

- **Author the What** — `product domain new <kind> <id> …` captures domain nodes
  (entity, command, event, read-model, ui-step, system, trigger, **product**
  (§3.0 root owning domains+systems), **journey** (§3.0.1 cross-system flow
  composition), **quality-demand** (§3.6 runtime-bound / architectural NFR), …);
  `product domain show/list` inspects them; `product domain export` emits Turtle.
- **Validate** — `product domain validate` runs the per-node §3.1/§3.2 shapes;
  `product domain validate --strict` adds the graph-level completeness checks
  (flow ownership §3.2.5, the Command pattern §3.2.0, view consumption §3.4, the
  unreifiable seam §4.5, **journey conformance §3.0.1** — every crossing a
  Translation — and, when a How contract is present, that an architectural
  quality demand's `constrains` binds a real How element §3.6).
- **Make behaviour executable** — `product decider derive <aggregate>` derives a
  Decider's signature from the event model; `product decider validate <id>` runs
  the §3.3 drift rules + the state/Decider justification detectors;
  `product decider simulate` runs its scenarios. `product projector …` is the §3.4
  read-model peer.
- **Realise it** — `product how`, `product slice`, `product build`, `product seam`,
  `product preview` cover the How contract, delivery slices, and the screen seam.
  `product how set version|realises-version --id <v>` carries the §7.3 semantic
  versions (a How declares which What-version it realises).
- **Direction (§7.3)** — `product target new <id> --version <v> --slice <deliverable>…`
  declares a future partition of feature-slices; `product target direction <id>`
  computes the gap (the unrealised members) — a query over the graph, not prose.
- **Build seam (§5.1)** — `product build <deliverable> --emit-seam` emits the work
  units as build-seam envelopes (by value + content-hash identity, the outbound
  half); `product verdict <file>` validates an inbound verdict event against the
  pinned accepted/rejected/escalate vocabulary. Schemas: `schema/json/build-seam/`.

The reference What lives in `.product/author-domain/product-cli/`; the live web
view (`product mcp --http`, then open `/`) renders it across three connected
views — Systems (§3.0), Domain ER (§3.1) and Flows / Event-Modeling swimlanes
(§3.2) — projected from `/api/graph` (`pf::viz`) and live-refreshed over SSE.

## Key Conventions

- **No unwrap**: `#![deny(clippy::unwrap_used)]` — use `?`, `.ok_or()`, `.unwrap_or_default()`, or match
- **Error model**: All errors go through `ProductError` in `error.rs` — each variant maps to a specific exit code
- **Atomic writes**: File writes use `fileops::atomic_write()` with advisory locking
- **Graph is derived**: No persistent graph store. The What graph is held in a session and serialized to Turtle/YAML under `.product/`
- **Pure `pf/`**: every file in `product-core/src/pf/` depends only on `crate::error` — the framework graph is self-contained

## Architecture Pattern — Slice + Adapter

The codebase is organised as vertical slices, each with a pure domain module
in `product-core` and a thin CLI adapter in `product-cli/src/commands/`.
This separation keeps business logic unit-testable without tempdirs, print
capture, or `cargo run`, and lets sibling CLIs (`decision-cli`) reuse the
slice library without inheriting the CLI surface (FT-107).

**Slice modules (`product-core/src/<slice>/`)** — pure, testable:
- `plan_*` / `build_*` functions take current state + user input, return a
  struct describing the intended change. No I/O, no println, no exit.
- `apply_*` functions take a plan struct and perform the minimal I/O
  (`fileops::write_file_atomic`, `write_batch_atomic`) needed to commit it.
- `render_*` functions turn result structs into text strings. JSON rendering
  is derived from `serde::Serialize` on the plan / result types.
- Unit tests (`src/<slice>/tests.rs`) exercise the pure functions directly.

Reference slices (`product-core/src/pf/`):
- `pf/feature/`-style modules don't exist; the slices are the framework kinds —
  e.g. `pf/decider*` (derive/validate/simulate the §3.3 Decider),
  `pf/projector*` (§3.4), `pf/slice*`, `pf/how*`, `pf/seam*`, and the `domain`
  CRUD pipeline (`pf/edit.rs` → `pf/validate.rs` → `pf/turtle.rs`/`pf/seed.rs`).

**Command adapters (`product-cli/src/commands/<cmd>.rs`)** — thin:
- A read/write adapter returns `CmdResult = Result<Output, ProductError>`; it
  loads the What graph via a pf session loader, calls the slice's pure
  `derive_*`/`validate_*`/`plan_*`+`apply_*`, and wraps the result in `Output`.
  Never call `println!`.
- Wire into `dispatch()` in `commands/dispatch.rs` (PF families funnel through
  `dispatch_pf`).

**Handlers that remain on `BoxResult`** are intentional — keep them where a
handler prints continuous progress (`build`, `author`, `init`, `mcp`), has
exit-code semantics `CmdResult` can't express, or is a trivial wrapper
(`completions`, `hooks`).

## Adding a New Command

1. Add the clap subcommand in `src/commands/<cmd>.rs` and the variant in
   `commands/root_enum.rs`; declare the module in `commands/mod.rs`.
2. If the command has non-trivial logic, create a slice at `product-core/src/pf/<cmd>*`.
3. Implement the handler as a thin adapter.
4. Wire into `commands/dispatch.rs`.
5. Add unit tests on the pure slice functions (a `pf/<cmd>_tests.rs` sibling).
6. Add a framework integration test in `product-cli/tests/framework.rs` with `assert_cmd`.

## Adding a New Module

1. Create `src/foo.rs` (or `src/foo/mod.rs` for a multi-file slice)
2. Add `pub mod foo;` in `src/lib.rs`
3. Consume from command adapters via `use product_lib::foo;`
4. Keep the first `//!` doc line free of the word "and" (SRP fitness test)
5. Keep every file under 400 lines (file-length fitness test)

## Adding a New Node Kind (pf graph)

A node kind lives in **five hand-maintained parallel enumerations** — miss one
and the kind silently vanishes on a Turtle round-trip (which `finalize` and the
`from_spec` reload depend on), with no error. When adding a kind to
`DomainGraph`, wire **all** of:

1. the struct field on `DomainGraph` (`pf/model*.rs`) and its `counts()` row;
2. Turtle **emit** (`pf/turtle*.rs`);
3. seed **parse** (`pf/seed*.rs`) — emit and parse must be symmetric;
4. `seed_canon::canonicalize` — sort the new list (and any id-list fields), or
   re-export churns and the byte-stability test fails;
5. `pf/viz.rs` if the kind should render in the web view.

The guard: `pf/seed_tests.rs::maximal()` builds one node of **every** kind and
`full_graph_round_trips_losslessly` proves emit→parse→canon is lossless, while
`maximal_populates_every_node_kind` fails by name if a kind is added to
`counts()` but not to `maximal()` — so steps 1–4 cannot be silently skipped.

## Test Organization

- **Unit tests**: `#[cfg(test)] mod tests` (or a `#[path] mod tests` sibling) at the bottom of each `pf/` source file — the real framework-graph coverage.
- **Framework integration tests**: `product-cli/tests/framework.rs` using `assert_cmd` + a temp-dir `Harness` (`init --demo` → `domain`/`decider`/…).
- **Fitness gates**: `product-cli/tests/code_quality_tests.rs` (file length ≤ 400, function length, SRP, module structure).
- **Property tests**: `product-core/tests/property/` using `proptest` (fileops/init).

## Documentation System

- **Framework spec** (source of truth): `docs/product-framework-open.md` — the open standard for What/How/Delivery, §-numbered.
- **Conformance clauses**: `docs/two-pillars-conformance.md`.
- **Examples + workshop**: `docs/examples/`, `docs/workshop/`, `docs/workshop-runbook.md`.

## Dependencies

Key crates: clap (CLI), serde/serde_yaml/serde_json/toml (serialization), oxigraph (RDF/SPARQL — the `pf` rule engine + seed parser), axum/tokio (HTTP server), sha2 (hashing), fd-lock (file locking), chrono (dates).

Dev: tempfile, assert_cmd, predicates, proptest.
