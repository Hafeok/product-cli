# CLAUDE.md — Product CLI

## What is this project?

Product is a Rust CLI and MCP server for the **Product Framework** — the open
What/How specification graph (`docs/product-framework-open.md`, currently
v1.9.1). It captures and verifies a product's *What* (domain model, event model,
Deciders, Projectors, systems, triggers, UI/AIO model) and *How* (contracts,
reification, delivery), all under `.product/`. The reference What lives in
`.product/author-domain/product-cli/`.

> **Graph-only.** This repo was pivoted to the framework graph alone. The former
> FT/ADR/TC knowledge-graph tool (the `adr`/`test`/`gap`/`drift`/`conformance`/
> `implement`/`verify` commands, `docs/{features,adrs,tests}`, and the meta-graph
> engine) has been removed. Don't reach for those commands or dirs. Note the
> current `product feature` command is unrelated — it is the §7.1 **delivery
> feature** (see *Vocabulary* below), not the old FT-XXX artifact.

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

Suite composition (~500 tests):

| Binary | What it covers |
|---|---|
| `cargo test -p product-core --lib` | pf unit tests (the framework graph, in `#[cfg(test)] mod tests`) |
| `cargo test -p product-mcp` | MCP registry + framework tool handlers (stateless + phase-gated session) |
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
  src/pf/<mod>/      #   NO clap / axum / tower-http. `[lib] name = "product_core"`
  src/author/        #   Session launch: domain-capture + the phase-gated workflow
  tests/property/    #   proptest over fileops/init
product-mcp/         # MCP server (stdio + HTTP via axum). Depends on
  Cargo.toml         #   product-core. Re-exports `ToolRegistry`,
  src/lib.rs         #   `run_stdio`, `run_http`, `serve_http_blocking`.
  src/registry.rs    #   stateless tool dispatch (domain/decider/projector/…)
  src/workflow.rs    #   phase-gated session transport (What→How→Build gating)
product-cli/         # The `product` binary. Depends on product-core +
  src/main.rs        #   product-mcp. Owns clap, the commands/ adapter layer,
  src/commands/      #   and the framework integration tests.
    mod.rs           #     Subcommand enum + dispatch
    domain.rs        #     What-graph CRUD + `validate [--strict]`
    decider.rs       #     §3.3 Decider derive/validate/simulate
    projector.rs     #     §3.4 Projector derive/validate
    build.rs · how.rs · feature.rs · seam.rs · preview.rs · session.rs · …
    target.rs        #     §7.3 target version + computed direction/gap
    verdict.rs       #     §5.1 build-seam verdict-event validation
  tests/
    framework.rs            # assert_cmd-driven framework CLI scenarios (~44)
    code_quality_tests.rs   # fitness gates (walk every member src/)
xtask/                # Workspace convention enforcement (`cargo xtask check`)
docs/
  product-framework-open.md   # The open framework spec (What/How/Delivery), a
                              #   mirror of ../product-framework (on re-sync, patch
                              #   preview/{build-seam,codegen,conformance}/ links
                              #   back to schema/json/... )
  two-pillars-conformance.md  # The conformance clause set
  examples/ · workshop/       # Worked examples + workshop runbook
.product/
  config.toml          # Repo config (`[author].cli` sets the default session CLI: claude|copilot)
  how-contract.yaml    # The canonical §4 How (hand-editable source); the self-hosted blueprint refs it
  author-domain/       # The captured What graphs (e.g. product-cli — the example What)
  deciders/ · features/ · work-units/ · deliverables/ · blueprints/ ·
  deployable-units/ · sessions/
```

A blueprint's `how-contract.yaml` may be an inline contract **or** a one-line
`ref: <relative path>` stub pointing at a shared one — resolved by
`HowContract::load_opt` (one hop, relative to the stub). The self-hosted
`blueprints/product-cli/` uses `ref: ../../how-contract.yaml` so the repo has a
single canonical How; `product blueprint init` still scaffolds a full inline
contract for a genuinely standalone blueprint.

**Downstream consumers** (e.g. `decision-cli`) should add only:

```toml
[dependencies]
product-core = { path = "../product-cli/product-core" }   # or a git rev
```

This buys the `pf/` framework graph and `ProductError` without
pulling in `clap`, `axum`, `tower-http`, or the `product` binary.

## Vocabulary — three senses of "slice" (don't conflate them)

The framework (§7.1) fixes the delivery containment **feature ⊇ flow ⊇ slice**.
The word "slice" is overloaded across the repo — keep the three apart:

- **§7.1 feature** — a *reference to a subgraph* of one or more flows: the
  `Feature` struct (`pf/feature.rs`), the `product feature` command, the
  `product_feature_*` MCP tools, stored under `.product/features/*.yaml`. This is
  the delivery unit that was formerly (mis)named "slice". A **deliverable** wraps
  a feature and adds shippable acceptance/runner; a **release** partitions
  deliverables.
- **atomic slice = work unit (§5, §3.2.0)** — a single pattern instance
  (Trigger → Command → Decider → Events, or a View + the events it reads): the
  `product_work_unit_*` tools, stored under `.product/work-units/`. A flow is a
  connected chain of these.
- **vertical slice (architecture)** — the *module-organization* pattern below
  ("Slice + Adapter"): a pure `pf/` module + a thin CLI adapter. Nothing to do
  with delivery; it is how the code is laid out.

Back-compat: the old `slice` on-disk key and CLI/MCP argument still load —
`Deliverable.feature` carries `#[serde(alias = "slice")]`, CLI `--feature` has
`alias = "slice"`, and the delivery MCP handlers fall back to the `slice`/`slices`
keys.

## Working with the framework graph

Use the `product` CLI (or MCP tools) to author and verify a What/How graph under
`.product/`:

- **Author the What** — `product domain new <kind> <id> …` captures domain nodes
  (entity, command, event, read-model, ui-step, **system**, trigger, **product**
  (§3.0 root owning domains+systems), **journey** (§3.0.1 cross-system flow
  composition), **quality-demand** (§3.6 runtime-bound / architectural NFR), …);
  `product domain show/list` inspects them; `product domain export` emits Turtle.
  A **system** must also declare its §3.2.5 sub-kind: `product domain new system
  <id> --system-kind service|application|website|cli --purpose … ` (the top-level
  `<kind>` selects the node *type*; `--system-kind` sets the system's own `kind`).
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
- **Realise it** — `product how`, `product feature`, `product build`, `product seam`,
  `product preview` cover the How contract, delivery features, and the screen seam.
  `product feature new <id> --anchor <node>…` saves a §7.1 feature (a subgraph
  pointer; its build-context is *assembled from the model*, never restated).
  `product how set version|realises-version --id <v>` carries the §7.3 semantic
  versions (a How declares which What-version it realises).
- **Design system (§11)** — `product design-system add <manifest>` vendors a §11.3
  manifest (declaration + implementation bundle: per-target component sources, token
  values per theme, templates) under `.product/design-systems/<id>/`; `validate` /
  `couple` are the wholeness + §11.2 coverage checks; `bind <id>` records the choice
  on the How contract (§4.5). Once bound, every `product codegen` backend gates on the
  coupling at plan time and emits `design-system.g.json` + `tokens.g.css` (hash-pinned;
  `reify check` catches design-system drift), and `product codegen web` renders one
  on-system HTML page per UI step, styled exclusively via tokens.
- **DeployableUnit (§4/§4.2)** — `product deployable-unit new <id> --built-from
  <blueprint> --system <sys>… [--environment … --domain-name/--bundle-id/--runtime]`
  declares the concrete artifact a **blueprint** (v1.7.0's rename of *archetype*)
  is instantiated as for a system, carrying its per-environment deployment
  identity. `validate` resolves `built_from` against `.product/blueprints/` and
  each `deploys_system` against the What. Stored under `.product/deployable-units/`;
  a How-phase concept (edges `instantiated_as`/`deploys_system`/`built_from`/
  `carries_identity`). *`archetype` remains a back-compat alias everywhere.*
- **Direction (§7.3)** — `product target new <id> --version <v> --feature <deliverable>…`
  declares a future partition of features; `product target direction <id>`
  computes the gap (the unrealised members) — a query over the graph, not prose.
- **Build seam (§5.1)** — `product build <deliverable> --emit-seam` emits the work
  units as build-seam envelopes (by value + content-hash identity, the outbound
  half); `product verdict <file>` validates an inbound verdict event against the
  pinned accepted/rejected/escalate vocabulary. Schemas: `schema/json/build-seam/`,
  with siblings `schema/json/codegen/` (the code-generation seam §5.2 — codegen
  manifest + file plan) and `schema/json/conformance/` (the §6.3.1 behavioural-
  conformance wire protocol — decision/projection request+response).

The reference What lives in `.product/author-domain/product-cli/`. `product mcp
--http` serves two web views:

- **`/`** — the **1.7.0 explorer** (a React app embedded from
  `product-mcp/src/assets/ui/`, served via a `rust_embed` fallback in
  `http_ui.rs`; React + Babel vendored under `vendor/` so it needs no CDN or
  build step, transpiled in-browser). Six sections — The Graph (§2·§9), The What
  (§3), UI (§3.2), The How (§4, incl. **blueprints + DeployableUnits**), Build
  (§5–6), Delivery (§7, incl. **versions**). *Currently driven by the bundled
  `window.PF` demo data (`assets/ui/data*.js`), not live-wired to `/api/graph`
  yet — that is the follow-up pass.*
- **`/legacy`** — the original self-contained **live** 3-view page (Systems §3.0,
  Domain ER §3.1, Flows / Event-Modeling swimlanes §3.2), projected from
  `/api/graph` (`pf::viz`, gaining a §4 How lane in 1.7.0) and live-refreshed
  over SSE. This is the graph-connected view until the explorer is wired.

## Phase-gated session (What → How → Build)

`product session start <product>` (or `product mcp --workflow --session <id>`)
launches a **phase-gated** authoring session: the agent CLI (`[author].cli` —
claude or copilot) drives the graph *only through MCP tools*, writing the
**canonical `.product` graph directly** (no workspace copy — the session record
is just `workflow.json` under `.product/sessions/<id>/`). The transport
(`product-mcp/src/workflow.rs`) gates the tool surface by phase — `tools/list`
shows only the current phase's family, and out-of-phase calls are rejected:

- **What** — `product_domain_*`, `product_decider_*`, `product_projector_*`,
  `product_primitive_*`.
- **How** — `product_how_*`, `product_blueprint_*` (alias `product_archetype_*`),
  `product_deployable_unit_*`, `product_cell_*`,
  `product_work_unit_*` (the atomic slice), `product_worker_*`.
- **Build** — `product_feature_*`, `product_deliverable_*`, `product_release_*`,
  `product_target_*`, `product_build_run`.

Read-only tools from an earlier phase stay callable; writes lock to their home
phase (`phase_of` in `workflow.rs` is the single source of truth). Three control
tools are visible in every phase: `product_workflow_status`,
`product_workflow_advance`, and `product_session_finalize` — which validates the
What and, if conformant, stamps provenance and closes the session. Writes land
in canonical as they happen (`RepoLock` serializes them); there is no draft
rollback — an abandoned session's edits stay in the graph.

## Key Conventions

- **No unwrap**: `#![deny(clippy::unwrap_used)]` — use `?`, `.ok_or()`, `.unwrap_or_default()`, or match
- **Error model**: All errors go through `ProductError` in `error.rs` — each variant maps to a specific exit code
- **Atomic writes**: File writes use `fileops::atomic_write()` with advisory locking. MCP write-tool handlers hold a `RepoLock`; a validation failure rolls back the whole node (no partial writes — supply every shape-required field in one call)
- **Graph is derived**: No persistent graph store. The What graph is held in a session and serialized to Turtle/YAML under `.product/`
- **Pure `pf/`**: every file in `product-core/src/pf/` depends only on `crate::error` — the framework graph is self-contained

### CLI ↔ MCP parity

Every `product_*` MCP tool mirrors a `product` subcommand and must accept the
same fields. Two gotchas the current code encodes — preserve them:

- **The `kind` overload.** `product_domain_new` uses the top-level `kind` arg to
  *route the node type* and drops it from the field map, but `System` (§3.2.5) and
  `ContextMapping` (§3.1) each carry a real `kind` *struct field*. The router
  shadows the field, so those sub-kinds arrive via the aliases `system_kind` /
  `mapping_kind` (mirroring the CLI's `--system-kind` / `--mapping-kind`),
  normalized back to `kind` in `domain_handlers::normalize_kind_aliases`. Any new
  field whose name collides with a routing key needs the same alias treatment.
- **Singletons via `product_how_set`.** `target` is one of
  `app-contract | infra-contract | version | realises-version`; for the two §7.3
  versions the `id` arg carries the version string (CLI `--id`).

## Architecture Pattern — Slice + Adapter

The codebase is organised as vertical slices (module organization — *not* the
§7.1 delivery feature), each with a pure domain module in `product-core` and a
thin CLI adapter in `product-cli/src/commands/`. This separation keeps business
logic unit-testable without tempdirs, print capture, or `cargo run`, and lets
sibling CLIs (`decision-cli`) reuse the slice library without inheriting the CLI
surface (FT-107).

**Slice modules (`product-core/src/<mod>/`)** — pure, testable:
- `plan_*` / `build_*` functions take current state + user input, return a
  struct describing the intended change. No I/O, no println, no exit.
- `apply_*` functions take a plan struct and perform the minimal I/O
  (`fileops::write_file_atomic`, `write_batch_atomic`) needed to commit it.
- `render_*` functions turn result structs into text strings. JSON rendering
  is derived from `serde::Serialize` on the plan / result types.
- Unit tests (`src/<mod>/tests.rs`) exercise the pure functions directly.

Reference modules (`product-core/src/pf/`) — the vertical slices are the
framework kinds:
- e.g. `pf/decider*` (derive/validate/simulate the §3.3 Decider),
  `pf/projector*` (§3.4), `pf/feature*` (§7.1 delivery feature), `pf/how*`,
  `pf/seam*`, and the `domain` CRUD pipeline (`pf/edit.rs` → `pf/validate.rs` →
  `pf/turtle.rs`/`pf/seed.rs`).

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
   `commands/root_enum.rs` (keep the variant list sorted — the
   `cli_subcommands_are_sorted` fitness gate enforces it); declare the module in
   `commands/mod.rs`.
2. If the command has non-trivial logic, create a slice at `product-core/src/pf/<cmd>*`.
3. Implement the handler as a thin adapter.
4. Wire into `commands/dispatch.rs`.
5. Add unit tests on the pure slice functions (a `pf/<cmd>_tests.rs` sibling).
6. Add a framework integration test in `product-cli/tests/framework.rs` with `assert_cmd`.
7. If the command mutates the graph, add the mirror `product_<cmd>_*` MCP tool
   (schema in `product-mcp/src/tools/`, handler in `product-mcp/src/`, dispatch in
   `registry.rs`) and place it in the right session phase via `workflow::phase_of`.

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

Delivery artifacts (features, deliverables, releases, targets, work units) are
**not** graph node kinds — they are standalone YAML under `.product/`, so they do
not touch these five enumerations.

## Test Organization

- **Unit tests**: `#[cfg(test)] mod tests` (or a `#[path] mod tests` sibling) at the bottom of each `pf/` source file — the real framework-graph coverage.
- **Framework integration tests**: `product-cli/tests/framework.rs` using `assert_cmd` + a temp-dir `Harness` (`init --demo` → `domain`/`decider`/…).
- **Fitness gates**: `product-cli/tests/code_quality_tests.rs` (file length ≤ 400, function length, SRP, module structure, sorted subcommands).
- **Property tests**: `product-core/tests/property/` using `proptest` (fileops/init).

## Documentation System

- **Framework spec** (source of truth): `docs/product-framework-open.md` — the open standard for What/How/Delivery, §-numbered. It mirrors the canonical `../product-framework`; on re-sync, patch the `preview/{build-seam,codegen,conformance}/` links back to `schema/json/...` (the framework repo keeps them under `preview/`, product-cli vendors them under `schema/json/`). `schema/json/build-seam/` (§5.1) has siblings `schema/json/codegen/` (the code-generation seam §5.2) and `schema/json/conformance/` (the §6.3.1 wire protocol).
- **Conformance clauses**: `docs/two-pillars-conformance.md`.
- **Examples + workshop**: `docs/examples/`, `docs/workshop/`, `docs/workshop-runbook.md`.

## Dependencies

Key crates: clap (CLI), serde/serde_yaml/serde_json/toml (serialization), oxigraph (RDF/SPARQL — the `pf` rule engine + seed parser), axum/tokio (HTTP server), sha2 (hashing), fd-lock (file locking), chrono (dates).

Dev: tempfile, assert_cmd, predicates, proptest.
