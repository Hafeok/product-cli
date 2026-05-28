---
id: FT-010
title: CLI Commands
phase: 1
status: complete
depends-on: []
adrs:
- ADR-009
- ADR-013
tests:
- TC-027
- TC-028
- TC-029
- TC-030
- TC-055
- TC-056
- TC-057
- TC-058
- TC-059
domains:
- api
- error-handling
domains-acknowledged:
  ADR-041: Pre-dates ADR-041; this feature does not author absence TCs or set removes/deprecates on ADRs. FT-047 owns the mechanics.
  ADR-018: Predates the 2026-04-22 scope promotion of ADR-018 to cross-cutting. Test coverage reflects the property/session/benchmark strategy as it existed when this feature shipped; not retroactively reclassified.
  ADR-047: Predates ADR-047; this feature does not author the functional-spec body convention. FT-055 owns the structural validator and W030 mechanics.
  ADR-043: Predates ADR-043; feature command adapters were written before the slice+adapter pattern was formalised and are not retroactively refactored.
  ADR-048: Predates ADR-048; this feature does not author the canonical .product/ folder layout. FT-057 owns the migration command and discovery fallback.
  ADR-040: predates ADR-040; feature ships independently of the unified verify / LLM-boundary pipeline
  ADR-042: Pre-dates ADR-042; this feature does not define TC types or validate the type vocabulary. FT-048 owns the mechanics.
---

### Navigation

```
product feature list [--phase N] [--status STATUS]
product feature show FT-001
product feature adrs FT-001          # all ADRs linked to this feature
product feature tests FT-001         # all test criteria for this feature
product feature deps FT-001          # full transitive dependency tree
product feature next                 # next feature by topological order (not phase label)

product adr list [--status STATUS]
product adr show ADR-002
product adr features ADR-002         # which features reference this ADR
product adr tests ADR-002            # which tests validate this decision

product test list [--phase N] [--type TYPE] [--status STATUS]
product test show TC-002
product test untested                # features with no linked tests
```

### Context Assembly

```
product context FT-001               # feature + direct ADRs + direct tests (depth 1)
product context FT-001 --depth 2     # transitive context: deps, shared ADRs, their tests
product context --phase 1            # all features in phase 1, with full context
product context --phase 1 --adrs-only  # phase 1 features + ADRs, no tests
product context ADR-002              # ADR + all linked features + all linked tests
product context --order id           # override default centrality ordering of ADRs
```

ADRs within a bundle are ordered by betweenness centrality descending by default — the most structurally important decisions appear first. Pass `--order id` for the previous ID-ascending behaviour.

### Status and Checklist

```
product status                       # summary: features by phase and status
product status --phase 1             # phase 1 detail with test coverage
product status --untested            # features with no linked test criteria
product status --failing             # features with one or more failing tests
product checklist generate           # regenerate checklist.md from feature files
```

### Graph Operations

```
product graph check                  # validate all links, DAG cycles, phase/dep mismatches
product graph rebuild                # regenerate index.ttl from all front-matter
product graph query "SELECT ..."     # SPARQL over the generated graph
product graph stats                  # artifact counts, link density, centrality summary,
                                     # φ (formal block coverage) across test criteria
product graph central                # top-10 ADRs by betweenness centrality
product graph central --top N        # configurable N
product graph central --all          # full ranked list
product graph coverage               # feature × domain coverage matrix
product graph coverage --domain security   # filter to one domain column
product graph coverage --format json       # machine-readable for CI
product impact ADR-002               # full affected set if this decision changes
product impact FT-001                # what depends on this feature completing
product impact TC-003                # what depends on this test criterion
```

`product graph check` also validates:
- No cycles in the `depends-on` feature DAG (exit code 1)
- Phase label / dependency order disagreements (exit code 2)
- Acknowledgements without reasoning — E011 (exit code 1)
- Domains declared in front-matter not present in `product.toml` vocabulary — E012 (exit code 1)

### Pre-flight and Domain Coverage

```
product preflight FT-001             # domain coverage check — run before authoring
product preflight FT-001 --format json

product feature acknowledge FT-009 --domain security \
  --reason "no trust boundaries introduced"
product feature acknowledge FT-009 --adr ADR-040 \
  --reason "standard output conventions apply"
```

Pre-flight must be clean before `product implement` proceeds (Step 0 in the pipeline). Pre-flight gaps are resolved by linking an ADR (`product feature link`) or acknowledging a domain/ADR with explicit reasoning. Acknowledgements without reasoning are E011 hard errors.

### Authoring

```
product feature new "Cluster Foundation"   # scaffold FT-XXX file with next ID
product adr new "Use openraft for consensus"
product test new "Raft leader election" --type scenario

product feature link FT-001 --adr ADR-002  # add edge (mutates front-matter)
product feature link FT-001 --test TC-002

product adr status ADR-002 accepted        # set ADR status
product test status TC-002 passing         # set test status
product feature status FT-001 complete     # set feature status
```

### Migration

```
product migrate from-prd PRD.md           # parse monolithic PRD → feature files
product migrate from-adrs ADRS.md         # parse monolithic ADR file → adr files + test files
product migrate validate                  # report what would be created without writing
```

### Gap Analysis

```
product gap check                         # analyse all ADRs for specification gaps
product gap check ADR-002                 # analyse a single ADR
product gap check --changed               # CI mode: only ADRs changed since HEAD~1
                                          # plus 1-hop graph neighbours
product gap check --format json           # structured JSON to stdout for CI annotation
product gap check --severity high         # filter to high-severity findings only

product gap report                        # human-readable gap summary across all ADRs
product gap stats                         # gap density by ADR, resolution rate over time

product gap suppress GAP-ADR002-G001-a3f9 --reason "deferred to phase 2"
product gap unsuppress GAP-ADR002-G001-a3f9
```

Gap findings go to stdout (they are results). Analysis errors (network failure, model error) go to stderr. Exit code 0 = no new gaps. Exit code 1 = new unsuppressed gaps found. Exit code 2 = analysis warnings (partial results, model errors on some ADRs).

### Drift Detection

```
product drift check ADR-002               # check one ADR against codebase
product drift check --changed             # only ADRs changed in current PR
product drift check --phase 1             # all phase-1 ADRs
product drift scan src/consensus/         # what ADRs govern this code?
product drift report                      # full drift report across all ADRs
product drift suppress DRIFT-ADR002-D001-a3f9 --reason "..."
```

### Metrics and Fitness Functions

```
product metrics record                    # snapshot current metrics to metrics.jsonl
product metrics trend                     # graph over last N snapshots
product metrics trend --metric phi        # single metric over time
product metrics threshold                 # check metrics against declared thresholds
product metrics stats                     # current values for all tracked metrics
```

### MCP Server

```
product mcp                               # stdio transport (default, for Claude Code)
product mcp --http                        # HTTP transport on default port 7777
product mcp --http --port 8080            # HTTP transport on custom port
product mcp --http --bind 0.0.0.0        # bind to all interfaces (remote access)
product mcp --http --token $SECRET        # bearer token auth (required for remote)
product mcp tools                         # list all available MCP tools
```

### Authoring Sessions

```
product author feature                    # graph-aware feature authoring session
product author adr                        # graph-aware ADR authoring session
product author review                     # spec gardening — find gaps and improve coverage

product install-hooks                     # install pre-commit hook in .git/hooks/
product adr review --staged               # review staged ADR files (used by pre-commit hook)
product adr review ADR-XXX                # review a specific ADR
```

### Agent Orchestration

```
product implement FT-001                  # gap-check → assemble context → invoke agent
product implement FT-001 --agent cursor   # override configured agent
product implement FT-001 --dry-run        # show what would be sent to agent, don't invoke
product verify FT-001                     # run linked TCs, update status, regenerate checklist
product verify FT-001 --tc TC-002         # run a single TC only
```

---

---

## Description

FT-010 defines the complete CLI surface of the Product tool: the full set of subcommands, their flags, their output formats, and the exit code contract used for CI integration (ADR-009, ADR-013). The commands documented in the prose above span navigation (`feature list/show`, `adr list/show`, `test list/show`), context assembly (`product context`), graph operations (`product graph check/rebuild/query/stats/central/coverage`), impact analysis (`product impact`), status and checklist (`product status`, `product checklist generate`), pre-flight and domain coverage (`product preflight`, `product feature acknowledge`), authoring (`product feature/adr/test new`, `product feature link`, status updates), migration (`product migrate`), gap analysis (`product gap`), drift detection (`product drift`), metrics and fitness functions (`product metrics`), MCP server (`product mcp`), authoring sessions (`product author`), and agent orchestration (`product implement`, `product verify`). All commands support `--format json` for machine-readable output. Graph health commands use a three-tier exit code scheme (0 = clean, 1 = errors, 2 = warnings).

## Functional Specification

### Inputs

- CLI invocation: subcommand name, positional arguments (artifact IDs, titles), and flags (`--phase N`, `--status STATUS`, `--format json`, `--depth N`, `--order id`, `--top N`, `--changed`, `--severity LEVEL`, `--dry-run`, `--reason REASON`, etc.).
- The in-memory knowledge graph rebuilt from all front-matter on every invocation.
- `product.toml` for path, phase, prefix, domain, agent, and threshold configuration.
- Optional `--config PATH` to override the default `product.toml` location.

### Outputs

- **Stdout**: command results — lists, show output, context bundles, graph stats, impact reports, gap findings, drift reports, metric snapshots. All results go to stdout so that output can be piped or redirected cleanly (e.g. `product context FT-001 > bundle.md`).
- **Stderr**: all errors and warnings formatted as rustc-style diagnostics (error code, file path, line, context, hint) for interactive use, or as JSON when `--format json` is set (ADR-013). Errors and warnings never appear on stdout.
- **Exit codes** (ADR-009):
  - `0` — clean result; no errors or warnings for graph health commands.
  - `1` — errors (broken links, cycles, malformed front-matter, E-codes).
  - `2` — warnings only (orphaned artifacts, missing exit criteria, W-codes) for `product graph check`, `product dep check`, `product preflight`.
  - `3` — internal Product error (Tier 4, bug in Product itself).
- **Files**: write commands produce modified artifact files (atomic writes); generation commands produce `checklist.md` or `index.ttl`.

### State

The CLI layer is stateless between invocations. Each invocation rebuilds the in-memory graph from scratch. The only persistent state is the artifact files themselves and `product.toml`. MCP server mode (`product mcp`) is the one long-lived process; it holds the in-memory graph and rebuilds it on each tool call.

### Behaviour

1. **Navigation commands** (`feature list`, `adr list`, `test list`, `feature show`, `adr show`, `test show`, `feature adrs`, `feature tests`, `feature deps`, `adr features`, `adr tests`, `test untested`): query the in-memory graph and print results to stdout. Accept `--phase N`, `--status STATUS`, `--type TYPE` filters. `feature next` applies topological sort plus phase gate (ADR-012, Capability 1).
2. **Context assembly** (`product context`): accepts a Feature, ADR, or phase filter as seed; assembles a context bundle via BFS to the specified depth (default 1); orders ADRs by betweenness centrality (overridable with `--order id`). Prints the bundle to stdout (ADR-012, Capabilities 2 and 3).
3. **Graph operations** (`product graph check`, `rebuild`, `query`, `stats`, `central`, `coverage`): `check` validates all links, DAG cycles, phase/dep ordering, acknowledgements, and domain vocabulary — exits 0/1/2 per the three-tier scheme. `rebuild` writes `index.ttl`. `query` runs SPARQL over the graph. `central` lists ADRs by betweenness centrality. `coverage` shows the feature × domain matrix.
4. **Impact analysis** (`product impact`): reverse-graph BFS from the target artifact; see FT-006 for full behaviour. Auto-triggered during `product adr status ADR-XXX superseded`.
5. **Error model** (ADR-013): all errors go to stderr using the four-tier taxonomy (Parse, Graph, Validation, Internal). `--format json` switches all error and warning output to structured JSON on stderr. No user action produces a Rust panic (enforced by `#![deny(clippy::unwrap_used)]`).
6. **Gap, drift, metrics, MCP, author, implement, verify**: these subcommand families are documented in the prose above; each delegates to its domain slice module. Long-running commands (`implement`, `author`, `mcp`) remain on `BoxResult` and may print continuous progress; other commands return `CmdResult` through the `render()` adapter.
7. **Pre-flight** (`product preflight FT-XXX`): checks domain coverage for a feature before implementation — must be clean before `product implement` proceeds. Gaps are resolved by linking an ADR or adding an acknowledged entry with explicit reasoning (E011 for empty reasoning).

### Invariants

- All results go to stdout; all errors and warnings go to stderr. This separation is enforced at the `render()` layer and verified by TC-059 (`product context FT-001 > bundle.md` produces a clean file even with warnings present).
- `product graph check` exits 0 on a fully consistent repository (TC-027), exits 1 on a broken link (TC-028), and exits 2 on warnings-only (TC-029, e.g. an orphaned artifact).
- The `--format json` flag is global to all commands; when set, errors, warnings, and results are all serialised as JSON (TC-056).
- No command produces a Rust panic or exposes a Rust backtrace to the user for an input-caused failure (TC-057).
- Broken-link error messages include file path, line number, offending content, and a remediation hint (TC-055).

### Error handling

- Unknown subcommand or missing required argument → clap error printed to stderr with usage hint; exit code 1.
- Artifact ID not found → E002 on stderr; command exits with code 1 (not 2, because a missing ID is an error, not a warning).
- Malformed front-matter in any file → E001 on stderr with file path and line; the file is skipped; the command continues with remaining valid files (ADR-013 graceful recovery).
- Internal error (unexpected None, assertion failure) → Tier 4 diagnostic with source location and Product version on stderr; exit code 3.
- All errors are structured: error code, tier, message, file path, line, context, detail, hint (ADR-013).

### Boundaries

- The CLI surface described in the prose above is the complete command set for this feature's scope. Commands added by later features (FT-019 gap analysis, FT-023 drift detection, FT-024 metrics, FT-030 implement/verify, FT-021 MCP) are wired into the same `dispatch()` match in `commands/mod.rs` but are specified by those features.
- `product graph check` validates structural graph health only; it does not execute TC runners. TC execution is `product verify`.
- The `--format json` flag controls output format for the current invocation only; it does not change the format of files written to disk.

## Out of scope

- Implementation of the agent orchestration pipeline (`product implement`, `product verify`) beyond the CLI surface definition — covered by FT-030 and FT-021.
- MCP tool definitions and server behaviour — covered by FT-021.
- Gap analysis LLM integration (`product gap check`) — covered by FT-019.
- Drift detection source scanning (`product drift check`) — covered by FT-023.
- Authoring session prompts and pre-commit hook integration (`product author`, `product install-hooks`) — covered by FT-022.
- Shell completion generation (`product completions`) — a thin wrapper with no domain logic.
