---
id: FT-059
title: MCP Parity for Health-Check Tools — drift check and preflight
phase: 5
status: planned
depends-on:
- FT-018
- FT-019
- FT-021
- FT-029
- FT-037
adrs:
- ADR-013
- ADR-020
- ADR-023
- ADR-026
tests:
- TC-717
- TC-718
- TC-719
- TC-720
- TC-721
- TC-722
- TC-723
- TC-724
domains:
- api
- error-handling
domains-acknowledged: {}
---

## Description

Bring the MCP tool surface to parity with the CLI for the read-only **health-check** commands an agent needs in a typical authoring or implementation session.

The agent context already advertises a "Key MCP Tools" table that lists `product_preflight` alongside `product_gap_check` and `product_graph_central`, and the CLAUDE.md / agent prompts repeatedly tell agents to "run `product gap check` and `product drift check` before committing." But two of the five health checks the working protocol relies on are simply **not exposed** over MCP today:

| Command | CLI | MCP today |
|---|---|---|
| `product graph check` | yes | `product_graph_check` ✓ |
| `product gap check` | yes | `product_gap_check` ✓ |
| `product impact ADR-XXX` | yes | `product_impact` ✓ |
| `product drift check` | yes | **missing** |
| `product preflight FT-XXX` | yes | **missing** |

Effect on agents: a session connected over MCP cannot finish the standard "before commit / before implement" loop without dropping back to the local CLI. That defeats the point of the MCP server for any caller that is not co-located with the repo (claude.ai, phone, remote agent). It also makes `AGENTS.md` factually incorrect — it lists `product_preflight` as a key tool but the registry returns *Tool not found*.

This is the same class of fix as FT-046 (which closed the ADR write-side gap). FT-059 closes the read-side gap for health checks.

---

## Depends on

- **FT-018** — Validation and Graph Health (`graph check`). Defines the existing health-check shape MCP already exposes.
- **FT-021** — MCP Server. Owns the tool surface this feature extends.
- **FT-029** — Gap Analysis. Provides `product_gap_check` as the parity reference point.
- **FT-019** — Domain Coverage Matrix. Owns `domains::preflight` — the function the new MCP tool wraps.
- **FT-037** — Tag-Based Drift Detection. Owns `drift::structural_for_feature` and `drift::check_adr` — the functions the new MCP tool wraps.

---

## Scope of this feature

### In

1. **`product_drift_check` MCP tool.** Wraps the same code path the CLI calls in `commands::drift::handle_drift` for the `Check` subcommand. Optional `id` parameter (ADR-NNN or FT-NNN). When omitted, runs across every ADR in the graph, identical to `product drift check` with no argument.
2. **`product_preflight` MCP tool.** Wraps `domains::preflight` plus the dependency-availability section from `commands::preflight::handle_preflight`. Required `id: FT-NNN` parameter.
3. **JSON-only output for both tools.** Health checks in the MCP layer return structured JSON; text rendering stays a CLI concern. The JSON shape mirrors the existing `--format=json` flag on the CLI commands so agents that already parse CLI output don't need a second parser.
4. **Read-only classification.** Both new tools set `requires_write: false` in `ToolDef`. They never mutate the graph, never write `drift.json`, never touch the baseline. Suppress / unsuppress stay CLI-only (out of scope here, see follow-on).
5. **Error signalling via the JSON envelope, not exit codes.** The CLI uses `process::exit(1)` for high-severity drift findings and exit `1` / `2` for preflight gaps and dep warnings respectively. Over MCP there is no exit code — instead, every response carries a top-level `summary` block (`high`, `medium`, `low`, `clean: bool`) and a `status` enum (`clean | warnings | findings`). Callers decide how to react.
6. **Registry + tool-list registration.** New entries in `tools.rs::read_graph_tools` (or a new `read_health_tools` group) and a new branch in `registry.rs::dispatch_tool`. Agent context (`agent_context.rs`) updated so `product_preflight` no longer lies — it actually exists.
7. **Tool-surface drift fitness test.** A new test (TC-723) that asserts every health-check command listed in `AGENTS.md` "Key MCP Tools" maps to a real entry in `tools::build_tool_list()`. Fails the build if the docs and the registry diverge again.
8. **Session tests in `tests/sessions/`.** Each scenario TC builds a temp repo via `product request apply`, drives the new MCP tool through the compiled binary's stdio transport, and asserts on the JSON envelope. Same pattern as FT-046's TC-577 through TC-584.

### Out

- **Drift `scan`, `diff`, `suppress`, `unsuppress`.** Suppress / unsuppress are write operations on `drift.json` and need their own ADR-038 request shape, not a one-off MCP tool. `scan` and `diff` produce LLM-ready bundles already covered by `product_context` semantics — exposing them is a separate feature.
- **Gap `report`, `stats`, `bundle`, `suppress`, `unsuppress`.** Same reasoning as above. `gap_check` already covers the "is there a problem?" question. The richer reports are a follow-on.
- **`product status`, `product metrics threshold`, `product cycle-times`, `product forecast`, `product feature next`, `product dep bom`.** Each is a candidate for its own MCP tool, but the user's stated need is the five-command health-check loop. Those expansions belong in a follow-on parity-audit feature.
- **Changes to CLI behaviour.** The CLI commands stay byte-identical. This feature only adds an MCP entry point that calls the same library functions.
- **Authentication / authorisation policy.** Both tools are read-only and inherit whatever `mcp.write` setting the server is launched with. No new auth knobs.

---

## Tool surface

### `product_drift_check` (new)

| Parameter | Type | Required | Notes |
|---|---|---|---|
| `id` | string | no | ADR-NNN or FT-NNN. Omit to scan every ADR. |
| `files` | array of string | no | Restrict to specific source files (mirrors CLI `--files`). |
| `all_complete` | boolean | no | Mirrors CLI `--all-complete`. Mutually exclusive with `id`. |

**Success response (no findings):**

```json
{
  "status": "clean",
  "checked": { "scope": "all", "adrs": 47, "features_with_tags": 12 },
  "findings": [],
  "summary": { "high": 0, "medium": 0, "low": 0, "suppressed": 0 }
}
```

**Success response (with findings):**

```json
{
  "status": "findings",
  "checked": { "scope": "FT-021" },
  "findings": [
    {
      "id": "DRIFT-FT-021-TAG-drift",
      "code": "D003",
      "severity": "medium",
      "description": "Implementation files changed since FT-021 was completed (product/FT-021)",
      "adr_id": "FT-021",
      "source_files": ["src/mcp/registry.rs", "src/mcp/tools.rs"],
      "suggested_action": "Review changes to ensure they don't contradict governing ADRs",
      "suppressed": false
    }
  ],
  "summary": { "high": 0, "medium": 1, "low": 0, "suppressed": 0 }
}
```

**Error cases:**

- `E022 health-check-id-not-found` — supplied `id` is neither a known ADR nor a known feature.
- `E023 health-check-conflicting-args` — `id` and `all_complete` both present.

### `product_preflight` (new)

| Parameter | Type | Required | Notes |
|---|---|---|---|
| `id` | string | yes | Feature ID — `FT-NNN`. |

**Success response:**

```json
{
  "status": "clean",
  "feature": "FT-021",
  "feature_domains": ["api"],
  "cross_cutting_gaps": [
    { "adr_id": "ADR-013", "adr_title": "Error Model …", "adr_domains": ["error-handling"], "status": "linked" },
    { "adr_id": "ADR-029", "adr_title": "Code Structure …", "adr_domains": [], "status": "linked" }
  ],
  "domain_gaps": [],
  "dep_availability": [
    { "id": "DEP-003", "title": "axum", "type": "library", "available": true, "deprecated": false }
  ],
  "summary": { "cross_cutting_gaps": 0, "domain_gaps": 0, "dep_warnings": 0 }
}
```

**Error cases:**

- `E022 health-check-id-not-found` — `id` is not a known feature.
- `E024 health-check-tc-runner-missing` — feature is in a status that requires runners but TCs are missing them (mirrors the CLI `TcRunnerMissing` error). The response carries the same `tc_ids` and `tc_paths` arrays the CLI would print.

### `product_graph_check` (existing — clarification only)

No behavioural change. Documented here only to make explicit that the new tools follow its envelope conventions: `status`, `summary`, `findings` keys at the top level.

---

## Implementation notes

- **`src/mcp/read_handlers.rs`** — add `handle_drift_check` and `handle_preflight`. Both functions take `&KnowledgeGraph`, `&Path` (repo root), and the `&Value` argument map; both return `Result<Value, String>` matching the existing handler signature.
- **`handle_drift_check`** — load `drift.json` baseline; resolve `source_roots` and `ignore` from `product.toml` (same defaults as the CLI: `["src", "crates"]` and `["target", ".git", "node_modules"]`); branch on `id` / `all_complete`; for the no-arg case iterate every ADR via `drift::check_adr`; for the feature case call `drift::structural_for_feature` and adapt its tag-vs-no-tag report into the unified findings envelope.
- **`handle_preflight`** — call `runner_required::find_offenders` first (returning `E024` when non-empty, same as the CLI gate). Then call `domains::preflight`. Then walk `graph.dependencies.values()` filtered by feature membership and run each `availability_check` shell command via `std::process::Command` with stdout/stderr discarded — identical to the CLI block. Preserve the deprecated/migrating warning bit.
- **`src/mcp/tools.rs`** — extend `read_graph_tools` (or split out `read_health_tools` for grouping) with the two new `ToolDef` entries. JSON schemas mirror the parameter tables above.
- **`src/mcp/registry.rs::dispatch_tool`** — two new branches, both calling `read_handlers::*` directly. No write-lock needed; both are read-only.
- **`src/agent_context.rs`** — the "Key MCP Tools" table currently advertises `product_preflight`. After this feature lands, that line stops being aspirational. No new entries needed (the table can be expanded in a follow-on) — but `product_drift_check` should be added so the table is honest about what's available.
- **Error codes.** New `E022 health-check-id-not-found`, `E023 health-check-conflicting-args`, `E024 health-check-tc-runner-missing` registered in `src/error.rs`. They follow the ADR-013 format and are documented in the generated guide.
- **Fitness test (TC-723).** New unit test in `src/agent_context_tests.rs` (or wherever the AGENTS.md generator lives) that parses the generated "Key MCP Tools" table, extracts every backticked tool name, and asserts membership in `tools::build_tool_list().iter().map(|t| &t.name)`. Fails the build the next time someone advertises a tool that doesn't exist.
- **Runner config.** Every TC in this feature gets `runner: cargo-test` and `runner-args: tc_XXX_snake_case` at the moment the test is written, per CLAUDE.md.

---

## Acceptance criteria

A spec-authoring agent connected over MCP can:

1. Call `product_drift_check` with no arguments and receive a structured JSON envelope summarising drift across every ADR — same scope as `product drift check` (TC-717).
2. Call `product_drift_check` with `id: FT-XXX` for a complete feature with a tag and receive the same `changed_files` set the CLI produces, in the unified envelope shape (TC-718).
3. Call `product_drift_check` with an unknown ID and receive `E022 health-check-id-not-found` (TC-719).
4. Call `product_preflight` with `id: FT-XXX` and receive cross-cutting coverage, domain coverage, and dependency-availability data — the JSON equivalent of the CLI's three-section text output (TC-720).
5. Call `product_preflight` on a feature whose status requires runners but whose TCs lack `runner-args` and receive `E024 health-check-tc-runner-missing` with the offending `tc_ids` and `tc_paths` (TC-721).
6. Call `product_preflight` with an unknown feature ID and receive `E022 health-check-id-not-found` (TC-722).
7. The fitness test that scans `AGENTS.md` "Key MCP Tools" passes — every advertised tool exists in the registry (TC-723).
8. `product graph check` exits 0 after the new tools are added to the registry; `product gap check` and `product drift check` from the CLI still produce identical output to the pre-FT-059 baseline (TC-724, exit-criteria).
9. `cargo t`, `cargo clippy -- -D warnings -D clippy::unwrap_used`, and `cargo build` all pass.
10. Every TC in the feature (TC-717 through TC-724) has `runner: cargo-test` and `runner-args` matching the Rust test function name.

---

## Follow-on work

- **Bundle / report parity.** `product gap bundle`, `product gap report`, `product gap stats`, `product drift diff`, `product drift scan` — each is independently useful as an MCP tool but each carries either output-format complexity or write semantics that deserve their own scoping pass.
- **Suppression parity.** `product gap suppress` / `product drift suppress` write to `gaps.json` / `drift.json`. They should be modelled as ADR-038 request shapes (`type: change`) so they participate in the audit log, not as one-off MCP tools.
- **Status / metrics / cycle-times parity.** `product_status`, `product_metrics_threshold`, `product_cycle_times`, `product_forecast`, `product_feature_next`, `product_dep_bom` — six more inspection tools the AGENTS.md table either advertises (and lies about) or could usefully advertise. Each is a small follow-on once the pattern from this feature is in place.
- **Tool-surface drift dashboard.** Once TC-723's fitness test exists, generalise it into a `product graph check`-level diagnostic that flags any documented tool, prompt, or schema reference that no longer exists in the registry.

---

## Functional Specification

### Inputs

- **`product_drift_check`** — optional `id: string` (ADR-NNN or FT-NNN), optional `files: array<string>`, optional `all_complete: boolean`. Mutually exclusive: `id` and `all_complete` cannot both be set.
- **`product_preflight`** — required `id: string` (FT-NNN).
- Both tools read repository state from the configured paths in `product.toml`. Neither tool reads from stdin.

### Outputs

- JSON object on success, error object on failure. Both follow the shape documented in the "Tool surface" section.
- The top-level keys `status`, `summary`, and one of `findings` / `cross_cutting_gaps` are stable; new keys may be added in additive updates without breaking callers.

### State

- Read-only. No artifact files mutated. `drift.json` and `gaps.json` baselines are read but not written. The `last-run` field on TCs is **not** touched (preflight is not a verify operation).
- Tool registration is in-memory state computed once per `ToolRegistry::new` call.

### Behaviour

- **`product_drift_check` no-arg**: iterate every ADR in `graph.adrs`, call `drift::check_adr` for each, aggregate findings, emit summary.
- **`product_drift_check` with `id: ADR-NNN`**: call `drift::check_adr(id, …)` once.
- **`product_drift_check` with `id: FT-NNN`**: call `drift::structural_for_feature`. If the feature has a completion tag, return changed-files + tag metadata in the envelope. Otherwise emit a `W020`-equivalent warning entry and best-effort fall back to ADR drift for any linked ADRs (same fallback the CLI uses).
- **`product_drift_check` with `all_complete: true`**: iterate every feature where `status == complete && completion_tag.is_some()`, run the structural drift check on each, aggregate.
- **`product_preflight`**: TC-runner gate first (per FT-058). Then domain coverage. Then dep availability. Aggregate into one JSON envelope — never short-circuits unless the runner gate fails (which returns E024).

### Invariants

- Both tools are read-only: `requires_write == false`.
- The JSON envelope of every health-check tool carries `status: "clean" | "warnings" | "findings"` and a `summary` object.
- The set of MCP tools is a strict superset of the AGENTS.md "Key MCP Tools" table — TC-723 enforces this.
- No tool removed by this feature: existing `product_graph_check`, `product_gap_check`, `product_impact` keep their current shape and behaviour.

### Error handling

- `E022 health-check-id-not-found` — supplied artifact ID does not exist in the graph.
- `E023 health-check-conflicting-args` — `id` and `all_complete` both supplied to `product_drift_check`.
- `E024 health-check-tc-runner-missing` — `product_preflight` invoked on a feature whose status requires runners and whose TCs lack runner config; payload mirrors the CLI's `TcRunnerMissing` error.
- All errors flow through `ProductError` and surface as JSON-RPC error responses; the MCP server does not panic and does not exit on health-check errors.

### Boundaries

- **In**: read access to the knowledge graph, `drift.json`, `dependency.availability_check` shell commands.
- **Out**: writing to any spec file, writing to baseline files, network egress beyond what `availability_check` shells already do, anything that needs a write lock.
- **Caller responsibilities**: agents must inspect `summary.high` / `summary.medium` themselves — the MCP layer never `process::exit`s on the agent's behalf.

## Out of scope

- Bundle-style output (`gap bundle`, `drift diff`) — separate follow-on.
- Suppress / unsuppress — write semantics, modelled via ADR-038 in a follow-on.
- Status / metrics / cycle-times / forecast / dep-bom MCP tools — out of scope for the *health-check* parity ask; addressed by a separate parity-audit feature.
- Changes to existing CLI behaviour — none.
- New ADRs — this feature is implementation work that follows ADR-020 (MCP transport) and ADR-038 (write semantics) without introducing new architectural decisions.
