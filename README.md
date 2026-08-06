# Product

**A CLI and MCP server for the Product Framework — specify software as a verifiable What/How graph.**

The [Product Framework](docs/product-framework-open.md) is an open standard for
describing a software product as one connected, machine-readable graph: the
**What** (domain model + event model — entities, commands, events, read models,
UI steps typed against Abstract Interaction Objects, *systems*, *triggers*,
*Deciders*, *Projectors*), the **How** (contracts, the screen-composition /
reification model, delivery features), and the typed links between them. The graph
can drive generation, gate verification, and explain itself — so "describe this
system" is a query, not a stale document.

This repo is the reference tooling: a single Rust binary (`product`) plus an MCP
server that lets an agent author and verify the graph directly. No database, no
service — the graph lives as YAML/Turtle under `.product/`.

```
$ product init --demo                 # scaffold + seed the bookstore What model
$ product domain new system sys-shop --system-kind application \
      --purpose "consumer e-commerce" --target-classes gui
$ product domain validate --strict    # per-node shapes + graph-level completeness
$ product decider derive Order        # derive an aggregate's executable signature
$ product decider validate Order-decider
$ product mcp --http                  # MCP server + a live Event-Modeling web view at /
```

---

## Install

```bash
# from source
cargo install --path product-cli
```

The binary ships with the What→How→Build **Claude Code skills** baked in.
`product init` writes them into `.claude/skills/` of the new repo (pass
`--no-skills` to opt out); `product skills install` (re)installs them, and
`product skills install --global` puts them in `~/.claude/skills/` for every
project. Start a fresh Claude Code session to pick them up, then `/product-session`.

## Choosing the agent CLI

`product session start` (and `product author domain`) host the What→How→Build
session in an agent CLI — **Claude Code** or **GitHub Copilot CLI**. The CLI is
resolved in this order:

1. the `--cli claude|copilot` flag, else
2. the repo's `[author].cli` in `.product/config.toml`, else
3. the global user default in `$XDG_CONFIG_HOME/product/config.toml`
   (or `~/.config/product/config.toml`), else
4. `claude`.

```toml
# .product/config.toml — make this repo default to Copilot CLI
[author]
cli = "copilot"
```

Scaffold it on a new repo with `product init --cli copilot`, or set a personal
default for every repo by putting the same `[author]` block in
`~/.config/product/config.toml`. With a default configured, `product session
start` needs no `--cli` flag.

## 60-second tour

```bash
product init --demo                   # a worked What model to explore
product domain list                   # the captured nodes, by kind
product domain show Order             # one node and its links
product domain export                 # the graph as RDF/Turtle
product domain validate               # §3.1/§3.2 per-node conformance shapes
product domain validate --strict      # + §3.2.0/§3.2.5/§3.4/§4.5 completeness checks
product decider derive Order          # §3.3 — derive decide/evolve signature
product decider simulate Order-decider  # run its flow-derived scenarios
product guide                         # where you are + the next step
```

## The model

- **What** — `product domain …` captures the domain + event model; `product
  decider …` (§3.3) and `product projector …` (§3.4) make behaviour and read
  models executable; `product primitive …` (§3.5) names irreducible algorithms.
- **How** — `product how`, `product feature`, `product build`, `product seam`,
  `product preview` cover the How contract, delivery features, the screen seam, and
  the §11/§12 design-system / content-store preview profiles.
- Everything is validated against the framework's SHACL shapes + SPARQL rules;
  the captured What serializes to Turtle (`product domain export`).

## MCP + the web view

`product mcp --http` starts the MCP server (framework tools: `product_domain_*`,
`product_decider_*`, `product_projector_*`, …) and serves a live web view at `/`
that renders the active What graph across three connected views — **Systems**
(the product → systems & journeys map, §3.0), **Domain** (one bounded context as
an ER graph, §3.1), and **Flows** (a system's event-model as Event-Modeling
swimlanes — triggers / commands / views over per-aggregate event streams, §3.2).
A node detail panel, the What→How→Build phase stepper, dark/light theme and live
SSE refresh round it out.

## ddd — Decision-Driven Design governance

The workspace also ships `ddd` (crates `ddd-core`, `ddd-lsp`, `ddd-mcp`,
`ddd-cli`), a separate tool over a separate store: a repo-local `.ddd/` graph
of predicates, closure claims, decisions, analyzer/linter manifests, pattern
instances, seam declarations, and interception event rows
([PRD](docs/ddd-cli-prd.md), formats: [migrations](docs/ddd-format-migrations.md)).

```bash
cargo run -p ddd-cli -- init        # scaffold .ddd/
ddd validate                        # schema + ontology rules (CI gate)
ddd diff --sarif build.sarif        # declared vs. detected rules (CI gate)
ddd report escapes                  # diff + cadence + basis-loss report
ddd why CA2007                      # rule -> decision -> principal -> claims
ddd render                          # static self-contained HTML projection of the graph
ddd serve                           # the ddd_* MCP surface (stdio)
ddd warmup                          # pre-load the LSP hosts (Roslyn solution load)
```

### MCP surface (M3/M4)

`ddd serve` exposes the `ddd_*` tool namespace over stdio: LSP-backed
language intelligence for C# (`roslyn-language-server --stdio
--autoLoadProjects`, the official prerelease .NET global tool) and Bicep
(`bicep-ls` from `Azure.Bicep.LangServer`) — `find_symbol`, `references`,
`hover`, `diagnostics` (joined to the manifests by rule id, the same join the
SARIF path uses), `signature`, `rename` (computes; application funnels
through the interceptor) — plus the governance tools `why`, `graph_query`,
`declare_seam`, `declare_pattern`, `accept_risk`.

`ddd_apply_edit` runs every edit through the per-language contract-surface
classifier (policy tables, PRD §9): non-surface edits apply; a surface edit
applies only with a matching same-session declaration; otherwise it is
rejected with a structured demand whose *facts* (symbol, kind, signature,
visibility, reference count) are pre-filled and whose judgment fields are
blank (`dec/ddd/rejection-facts-prefilled`). Modes: `intercept: enforce |
warn | off`, per artifact class via `intercept_by_class` (config format 3);
`adapter.csharp.internal_is_surface` flips the library-repo posture
(`dec/ddd/internal-not-surface`). Every classified surface outcome lands as a
row under `.ddd/seams/events/` — the correspondence dataset. Hosts are
spawned lazily, health-checked, and respawned on crash; while Roslyn loads
the solution, tools return an explicit `{"status": "loading"}` rather than
hanging. CI runs against a fixture-grade mock host
(`dec/ddd/fixtures-not-sdk`); set `DDD_LSP_E2E=1` with both tools on PATH to
run the gated real-host suites.

`ddd diff` compares the manifests under `.ddd/manifest/` against two detected
sources per language and reports `UNGOVERNED` (detected, no manifest entry),
`STALE` (manifest entry, absent from config and emissions), and
`UNCITED_SUPPRESSION` (a config or in-source suppression with no
risk-acceptance record):

- **configured** — parsed from `.editorconfig` (`dotnet_diagnostic.<ID>.severity`
  lines only; section globs are recorded, never evaluated — deliberately not an
  editorconfig engine), `bicepconfig.json` (`analyzers.core.rules` levels), and
  the root `Cargo.toml` (`[workspace.lints.clippy]`, for this repo's own gates).
  Rules enabled by analyzer-package defaults have no config line and surface
  via the emitted source only — `diff` says so when one source covers a rule.
- **emitted** — SARIF 2.1 files from real builds, ingested by one shared
  module. Produce them with (verified against current tool docs at M2):
  - **C#**: `dotnet build -p:ErrorLog=diag.sarif%2Cversion=2.1` — the MSBuild
    `ErrorLog` property; `version=2.1` is required (the default is SARIF 1.0),
    and `%2C` escapes the comma on the CLI (or set `<ErrorLog>` in the project).
  - **Bicep**: `bicep lint main.bicep --diagnostics-format sarif > bicep.sarif`
    (also available as `az bicep lint`).

  In-source suppressions differ per toolchain: the C# compiler still logs a
  `#pragma`-suppressed diagnostic into SARIF (marked suppressed), which is how
  `UNCITED_SUPPRESSION` sees it; Bicep's `#disable-next-line` removes the
  diagnostic from the output entirely, so Bicep source suppressions are
  invisible to detection — only `level: off` config suppressions are covered.

Point `ddd diff` at the files with `--sarif` or the `detect.sarif` list in
`.ddd/config.yaml`. This repo governs itself: `.ddd/manifest/clippy.yaml` maps
`clippy::unwrap_used` to its decision, and `ddd diff` verifies it against the
workspace lints table.

## Build & test

```bash
cargo build
cargo t                                          # full suite (alias: test --no-fail-fast)
cargo clippy -- -D warnings -D clippy::unwrap_used
```

See [CLAUDE.md](CLAUDE.md) for the architecture and contributor workflow, and
[docs/product-framework-open.md](docs/product-framework-open.md) for the spec.

## License

See [`LICENSE`](LICENSE).
