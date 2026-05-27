---
id: FT-069
title: MCP Parity for product_graph_check — config-aware checks, domain validation, due-dates, log findings
phase: 5
status: complete
depends-on:
- FT-021
- FT-042
- FT-046
- FT-053
- FT-055
- FT-059
adrs:
- ADR-020
- ADR-013
- ADR-039
- ADR-045
- ADR-047
- ADR-043
- ADR-018
- ADR-042
tests:
- TC-806
- TC-807
- TC-808
- TC-809
- TC-810
- TC-811
domains: []
patterns:
- PAT-001
- PAT-002
domains-acknowledged:
  ADR-050: ADR-050 introduces the PAT artifact type and explicitly targets the FT-070–FT-075 wave. FT-069 predates that wave and authors no pattern artifacts; it follows the pre-existing slice + adapter shape already established by FT-046 and FT-059. No PAT files are created, linked, or referenced.
  ADR-049: ADR-049 governs per-model context bundle templates (data-file driven). FT-069 touches the graph-check tool surface, not the context-bundle assembly path. Bundle templates are unaffected; no template files are added, modified, or referenced.
  ADR-051: ADR-051 requires TCs for side-effectful operations to assert on observed state (causation) rather than only the response envelope. FT-069's `product_graph_check` is strictly read-only — there is no on-disk state transition to observe. The TCs assert MCP/CLI JSON envelope parity for fixtures whose findings are themselves derived from disk, which is the appropriate observation surface for a read-only check.
  ADR-040: ADR-040 governs the unified product verify pipeline. FT-069 only fixes parity in the read-only product_graph_check tool surface — it does not alter the verify pipeline, the LLM boundary, or any pipeline stage. The verify pipeline does invoke graph check internally as stage 1, and will transparently benefit from the additional findings, but there is no behavioural change to the pipeline contract itself.
  ADR-048: ADR-048 defines the canonical .product/ repository layout. FT-069 is layout-agnostic — graph::full_check::run consumes a KnowledgeGraph, ProductConfig, and repo_root that have already been resolved through the standard config loader, which honours ADR-048 transparently. No new paths, no new directories, no layout assumptions.
  ADR-041: ADR-041 covers removal/deprecation absence-TCs and ADR removes/deprecates fields. FT-069 introduces no removal or deprecation tracking — it surfaces existing validation layers that were already running on the CLI. No absence TCs are needed because nothing is being removed.
---

## Description

Close the latest MCP write/read parity gap in the FT-046 / FT-059 / FT-062
/ FT-066 series. The MCP tool `product_graph_check` and the CLI command
`product graph check` are advertised as equivalent read-only health
checks, but the MCP handler executes only a subset of the validation
layers the CLI runs. Agents trusting the MCP output therefore observe a
**silently incomplete** health report — findings that block humans on
the CLI are invisible to the LLM.

Diagnosis (`src/mcp/registry.rs:145-156` vs.
`src/commands/graph_cmd.rs:78-113`):

| Layer | CLI | MCP | Findings hidden over MCP |
|---|---|---|---|
| Structural (`check_with_config`) | ✓ with config | ✗ no config | W030 (FT-055 functional-spec completeness), E006 unknown TC types (ADR-042) |
| Domain validation | ✓ `domains::validate_domains` | ✗ | W009, W010, E011, E012 |
| Responsibility (W019) | ✓ | ✓ | — (already at parity) |
| Planning / due-dates | ✓ `planning_validation::check_due_dates` | ✗ | W028 (overdue), W029 (approaching) |
| Request-log verification | ✓ when `[log].verify-on-check` | ✗ | Hash-chain findings from `verify_log` |

The CLI path accumulated these layers as FT-053, FT-055, and FT-042
landed; the MCP handler was never updated. There is no automated parity
guard, so this exact class of drift is repeatable.

## Depends on

- **FT-021** — MCP Server. Owns the tool surface this feature fixes.
- **FT-042** — Request Log Hash-Chain and Replay. Source of the
  log-verification findings the MCP must surface.
- **FT-053** — Planning — Feature Due Dates and Started Tags. Owner of
  W028 / W029.
- **FT-055** — Feature Functional Specification Section. Owner of W030.
- **FT-046** — MCP Parity for ADR Lifecycle Operations. Established the
  parity-via-shared-slice pattern this feature applies to graph check.
- **FT-059** — MCP Parity for Health-Check Tools. Adjacent parity work
  for `drift check` and `preflight`; this feature extends the pattern
  to `graph check`.

## Scope of this feature

### In

1. **New shared library function `graph::full_check::run(graph, config,
   root) -> CheckResult`** consolidates every validation layer the
   user-facing `product graph check` exposes: structural
   (`check_with_config`), domain (`validate_domains`), responsibility
   (`check_responsibility`), planning (`check_due_dates`), and — when
   enabled by `[log].verify-on-check` — request-log verification.
2. **CLI adapter migrates** from inline orchestration to calling
   `graph::full_check::run`. The `append_log_findings_to_check` helper
   is removed from `src/commands/graph_cmd.rs`.
3. **MCP handler migrates** from `graph.check()` +
   `check_responsibility` to calling `graph::full_check::run` directly,
   producing a JSON envelope byte-for-byte equivalent to the CLI's
   `--format json` output for the same fixture.
4. **Parity invariant TC.** A session test composes a fixture that
   exercises every validation layer (intentional W030 violation, due-
   date warning, domain-acknowledgement E011, log-verify finding) and
   asserts that the MCP `product_graph_check` JSON output equals the
   CLI `product graph check --format json` output verbatim.
5. **Per-layer scenario TCs.** Four targeted session tests — one per
   missing layer — confirm the MCP surfaces each finding class.

### Out

- **New validation layers.** This feature only achieves parity with the
  current CLI behaviour; it does not introduce new checks.
- **Output schema changes.** The MCP envelope shape stays exactly what
  `CheckResult::to_json()` already returns.
- **`config.validate_product_section()` warnings.** These are printed
  to stderr by the CLI and are not part of the structured result — they
  remain CLI-only and out of scope for MCP parity.
- **Exit-code semantics.** The MCP tool surface does not signal exit
  codes; the JSON envelope already carries the equivalent signal via
  the `errors` / `warnings` arrays.

## Tool surface changes

### `product_graph_check` — current vs. new

| Case | Current MCP behaviour | New MCP behaviour |
|---|---|---|
| Feature missing required body sections (W030) | finding absent | finding present, code `W030` |
| Feature with acknowledged domain but empty reason (E011) | finding absent | finding present, code `E011` |
| TC with `type: <unknown>` not in `[tc-types].custom` (E006) | finding absent | finding present, code `E006` |
| Feature past `due-date` (W028) | finding absent | finding present, code `W028` |
| Feature approaching `due-date` within threshold (W029) | finding absent | finding present, code `W029` |
| `requests.jsonl` chain broken with `[log].verify-on-check = true` | finding absent | finding present, propagated from `verify_log` |
| Clean repo | clean envelope | clean envelope (unchanged) |
| Feature outside responsibility (W019) | finding present | finding present (unchanged) |

## Implementation notes

- **`src/graph/full_check.rs`** (new file in the graph slice).
  Exports `pub fn run(graph: &KnowledgeGraph, config:
  &ProductConfig, root: &Path) -> CheckResult` plus a private
  `append_log_findings`. The module owns no state; it is a pure
  orchestration over already-existing pure helpers.

- **`src/graph/mod.rs`** — add `pub mod full_check;`.

- **`src/commands/graph_cmd.rs`** — replace the body of `graph_check`
  with a call to `product_lib::graph::full_check::run(&graph, &config,
  &root)`. Delete `append_log_findings_to_check`. The stderr
  `config.validate_product_section()` loop stays in the CLI adapter —
  it is presentation, not a structured finding.

- **`src/mcp/registry.rs`** — replace the inline arm

  ```rust
  "product_graph_check" => {
      let mut result = graph.check();
      let config = crate::config::ProductConfig::load_from_root(repo_root)
          .map_err(|e| format!("{}", e))?;
      crate::graph::responsibility::check_responsibility(
          graph, config.responsibility(), &mut result,
      );
      Ok(result.to_json())
  }
  ```

  with

  ```rust
  "product_graph_check" => {
      let config = crate::config::ProductConfig::load_from_root(repo_root)
          .map_err(|e| format!("{}", e))?;
      let result = crate::graph::full_check::run(graph, &config, repo_root);
      Ok(result.to_json())
  }
  ```

- **Concurrency.** `product_graph_check` is read-only; the
  `requires_write: false` flag is unchanged.

- **Schema changes.** None on input. The output JSON gains the missing
  finding classes; it is not a breaking shape change — the envelope
  was always `{ errors: [...], warnings: [...], summary: {...} }`.

- **Session tests in `tests/sessions/`.** Each per-layer TC composes a
  temp repo, invokes the MCP and CLI through the compiled binary, and
  diffs the JSON. The parity invariant TC iterates over a synthetic
  fixture that triggers every layer at once.

## Acceptance criteria

An MCP client can:

1. Call `product_graph_check` on a fixture with a feature missing
   required body sections and observe a `W030` finding in the JSON
   envelope — **TC scenario A**.
2. Call `product_graph_check` on a fixture with a feature carrying
   `domains-acknowledged.<d>: ""` and observe an `E011` finding —
   **TC scenario B**.
3. Call `product_graph_check` on a fixture with a feature past its
   `due-date` and observe a `W028` finding — **TC scenario C**.
4. Call `product_graph_check` on a fixture with `[log].verify-on-check
   = true` and a corrupted `requests.jsonl` and observe the
   propagated log-verification finding — **TC scenario D**.
5. For every fixture, the MCP envelope equals the CLI
   `--format json` envelope byte-for-byte — **TC invariant**.
6. `cargo t`, `cargo clippy -- -D warnings -D
   clippy::unwrap_used`, and `cargo build` all pass — **TC exit-
   criteria**.

## Functional Specification

### Inputs

- `product_graph_check` MCP tool: `{}` (no parameters today; unchanged).
- CLI: `product graph check [--format json]` (unchanged).
- Implicit input: every artifact under `[paths].features`, `.adrs`,
  `.tests`, `.dependencies`; plus `product.toml`; plus the request log
  at `[paths].requests` when `[log].verify-on-check = true`.

### Outputs

- JSON object with shape `{ errors: [...], warnings: [...], summary:
  { errors: N, warnings: M } }` (produced by `CheckResult::to_json`).
- Each finding has `code`, `tier`, `message`, optional `file`, `line`,
  `detail`, `hint`.

### State

- Read-only. No mutations.

### Behaviour

- The MCP handler loads `ProductConfig` from `repo_root` and dispatches
  to `graph::full_check::run(graph, &config, repo_root)`.
- `full_check::run` invokes, in order: `check_with_config(Some(&config))`,
  `domains::validate_domains`, `responsibility::check_responsibility`,
  `planning_validation::check_due_dates` (with `chrono::Local::now`),
  and — when `config.log.verify_on_check` — `append_log_findings`.
- The CLI adapter calls the same function and additionally prints
  `config.validate_product_section()` warnings to stderr (presentation
  only).

### Invariants

- For every fixture, `product_graph_check` (MCP) and `product graph
  check --format json` (CLI) return byte-identical JSON envelopes.
- No validation layer is added to the CLI without simultaneously being
  surfaced over the MCP (achieved structurally by routing both through
  `graph::full_check::run`).
- The MCP envelope schema is unchanged from prior versions; only the
  set of populated findings expands.

### Error handling

- Config-load failure returns the existing `format!("{}", e)` error
  envelope; no new error codes.
- A missing or unreadable `requests.jsonl` (when verify-on-check is
  enabled) is treated as "no findings" by `verify_log`, matching CLI
  behaviour.

### Boundaries

- This feature does not change the structural validation logic, the
  domain validation logic, the responsibility logic, the due-date
  logic, or the log-verification logic — it only ensures the MCP
  exercises all of them.
- It does not introduce new error codes.
- It does not change the input or output schema of
  `product_graph_check`.

## Out of scope

- New validation layers (no W031, no E013, no new findings).
- Restructuring `CheckResult` or its JSON serialisation.
- Changing CLI exit-code semantics.
- Migrating other read-only MCP tools to a shared "full check"
  pattern (track separately if needed — `gap check`, `coverage`,
  etc. each have their own slice).
