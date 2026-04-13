# Product CLI — Feature Checklist

> Auto-maintained during implementation.
> Status: [ ] not started, [~] partial/stub, [x] implemented, [T] tested
>
> Last verified: 2026-04-12 — 95 unit tests, 12 integration, 9 property, 4 benchmarks

---

## Phase 1-3 (Complete — see previous checklist for full detail)

All Phase 1, 2, 3 items: [T]

---

## ADR-018: Testing Strategy

- [T] Property-based tests (proptest) — TC-P001 through TC-P011 (9 tests)
- [T] Integration test harness — Harness struct, fixtures, Output assertions
- [T] Integration tests IT-001 through IT-018 (12 tests)
- [x] LLM benchmark runner scaffold (benchmarks/runner — Phase 3 deferred)

## ADR-019: Continuous Gap Analysis

- [T] Gap types G001-G007 with severity levels
- [T] GapFinding, GapReport, GapSummary structs
- [T] Gap ID derivation (sha256-based deterministic)
- [T] `gaps.json` baseline — load, save, suppress, unsuppress, resolve
- [T] `product gap check [ADR-XXX]` — structural analysis
- [T] `product gap check --changed` — git-scoped CI mode
- [T] `product gap check --format json` — structured output
- [T] `product gap report` — human-readable
- [T] `product gap suppress GAP-ID --reason` — baseline mutation
- [T] `product gap unsuppress GAP-ID`
- [T] `product gap stats` — density and resolution metrics
- [x] G001/G002/G005 LLM checks (structural heuristic, full LLM stubbed)
- [T] G003 missing rejected alternatives (structural)
- [T] G006 feature coverage gap (structural)
- [T] G007 stale rationale references (structural)

## ADR-020: MCP Server

- [T] ToolRegistry with read + write tool sets (18 tools)
- [T] JSON-RPC protocol handler (initialize, tools/list, tools/call)
- [T] stdio transport (`product mcp`)
- [T] HTTP transport (`product mcp --http`) with axum
- [T] Bearer token authentication for HTTP
- [T] CORS configuration for claude.ai access
- [T] Write permission gating (`mcp.write` in product.toml)
- [T] `.mcp.json` scaffolding via `product install-hooks`

## ADR-021: Agent Orchestration

- [T] `product implement FT-XXX` — 5-step pipeline
- [T] Gap gate (step 1) — blocks on unsuppressed high-severity gaps
- [T] Context assembly (step 3) — depth-2 bundle with TC status table
- [T] `--dry-run` — writes context file without invoking agent
- [T] `--no-verify` — skips auto-verify
- [T] Agent invocation (step 4) — claude command with context file
- [T] `product verify FT-XXX` — TC runner protocol
- [T] TC runners: cargo-test, bash, pytest, custom
- [T] TC status update in front-matter (passing/failing)
- [T] Feature status auto-update (complete if all pass)
- [T] Checklist auto-regeneration after verify

## ADR-022: Authoring Sessions

- [T] `product author feature` — versioned system prompt
- [T] `product author adr` — reads graph before writing
- [T] `product author review` — spec gardening session
- [T] Default prompts for each session type
- [T] Prompt version loading from benchmarks/prompts/
- [T] `product adr review --staged` — pre-commit structural checks
- [T] `product install-hooks` — pre-commit hook installation

## ADR-023: Drift Detection

- [T] Drift types D001-D004 with severity
- [T] DriftBaseline — load, save, suppress, unsuppress
- [T] `product drift check [ADR-XXX]` — source file analysis
- [T] `product drift check --files` — explicit source file override
- [T] `product drift scan SRC_PATH` — reverse ADR lookup
- [T] `product drift suppress/unsuppress`
- [T] Source file resolution: pattern-based + front-matter override
- [T] `drift.json` baseline lifecycle

## ADR-024: Fitness Functions

- [T] MetricSnapshot with 9 tracked metrics
- [T] `product metrics record` — append to metrics.jsonl
- [T] `product metrics threshold` — CI gate (exit 1/2/0)
- [T] `product metrics trend` — ASCII sparkline
- [T] Threshold config in product.toml `[metrics.thresholds]`
- [T] Snapshot roundtrip (serialize, append, load)
- [T] Threshold breach detection (min/max, error/warning severity)

---

## Test Counts

| Suite | Count |
|---|---|
| Unit tests (lib) | 74 |
| Integration tests | 12 |
| Property-based tests | 9 |
| Benchmarks | 4 |
| **Total** | **99** |

## Source Files

| File | Lines | Purpose |
|---|---|---|
| main.rs | ~1700 | CLI entry point, all command handlers |
| lib.rs | 18 | Module re-exports |
| graph.rs | ~1040 | Knowledge graph, algorithms |
| migrate.rs | ~790 | PRD/ADR migration |
| mcp.rs | ~540 | MCP server (stdio + HTTP) |
| formal.rs | ~490 | AISP formal block parser |
| gap.rs | ~500 | Gap analysis |
| config.rs | ~370 | product.toml parsing |
| parser.rs | ~310 | Front-matter parser |
| error.rs | ~310 | Error model |
| metrics.rs | ~290 | Fitness functions |
| context.rs | ~270 | Context bundle assembly |
| implement.rs | ~330 | implement + verify pipeline |
| drift.rs | ~300 | Drift detection |
| author.rs | ~180 | Authoring sessions |
| rdf.rs | ~200 | TTL export + SPARQL |
| types.rs | ~280 | Core artifact types |
| checklist.rs | ~100 | Checklist generation |
| fileops.rs | ~240 | Atomic writes + locking |
