---
id: FT-066
title: MCP Parity for Feature/TC Status Writes and Reciprocal Linking
phase: 5
status: complete
depends-on:
- FT-021
- FT-046
- FT-058
adrs:
- ADR-010
- ADR-020
- ADR-032
- ADR-047
tests:
- TC-778
- TC-779
- TC-780
- TC-781
- TC-782
- TC-783
- TC-784
- TC-785
- TC-786
- TC-787
- TC-788
domains:
- api
- testing
patterns:
- PAT-001
- PAT-002
- PAT-003
domains-acknowledged:
  ADR-048: No file-layout changes. New code reads and writes through the existing `KnowledgeGraph` + `fileops::write_batch_atomic` paths; the canonical `.product/` layout is orthogonal to this feature.
  ADR-049: This feature does not introduce or modify observability hooks. The MCP write path already emits the same tool-call logs as before; status-write success/failure flows through the existing JSON-RPC envelope without new instrumentation.
  ADR-041: Purely additive write parity. Nothing is removed or deprecated; no absence TCs and no removes/deprecates fields are required. The legacy no-op `handle_status_update` is deleted as dead code, not as a deprecated API.
  ADR-040: This feature does not introduce or alter verify-pipeline structure and adds no LLM-bounded calls. All new behaviour lands inside the existing MCP request/response loop already governed by ADR-040.
  ADR-018: FT-066 ships scenario and invariant TCs that exercise the MCP write surface through the compiled binary, matching the established property/session/scenario coverage strategy. ADR-018's testing taxonomy is unaffected — no new TC type is introduced.
  ADR-043: FT-066 ships its MCP handlers through the existing slice + adapter pattern (calling `feature::plan_status_change` / `tc::plan_status_change` and the new `feature::plan_link`). ADR-043's command-adapter contract is honoured; no parallel architecture is introduced.
  ADR-042: All TCs in this feature use the existing `scenario`, `invariant`, and `exit-criteria` types. No new structural or descriptive TC types are introduced; ADR-042's reserved/open partition is unaffected.
  ADR-047: Feature body conforms to the Description / Functional Specification / Out of scope structure required by ADR-047; W030 completeness gate behaviour is not changed by this feature, only enforced at parity over MCP.
---

## Description

Close two related FT-046 follow-on gaps in the MCP write surface. Both
have the same shape: a write tool that advertises a bidirectional
contract but only persists one side, returning a misleading success
envelope. While filing this feature itself, the seven scenario TCs
created for it were silently orphaned by the second of these bugs —
`product_feature_link` calls registered TC IDs in `FT-066.tests` but
left every `TC-XXX.validates.features` empty, requiring a manual
back-fill via `product request apply`. The bug ate its own dogfood.

### Gap 1 — Status writes are no-ops

The MCP tools `product_feature_status` and `product_test_status` are
advertised as "Set feature status" / "Set test criterion status" with
`requires_write: true` and a proper JSON input schema, but the
dispatcher routes both to a single no-op stub `handle_status_update`
(in `src/mcp/write_handlers.rs:183-187`) that:

1. Echoes the requested `id` and `status` back unchanged.
2. Appends a `note: "Use CLI for status updates with full
   side-effects"` field.
3. **Does not write to disk.**
4. Returns success (no JSON-RPC error, no MCP tool error).

This makes the MCP write contract silently unsound: an agent
reasonably trusting the `{ id, status }` echo will believe the
transition happened. It did not. Every subsequent `product graph
check`, `product status`, or `product context` still observes the
stale status. The buried `note` is the only signal of failure and
sits below the success envelope where an LLM is least likely to
surface it.

### Gap 2 — Link writes are one-sided

The MCP tool `product_feature_link` (and the CLI command of the same
name) updates the calling feature's `adrs:` / `tests:` / `depends-on`
arrays but does not reciprocate the back-references on the link
targets. After `product_feature_link FT-X --test TC-Y`:

- `FT-X.tests` contains `TC-Y` ✓
- `TC-Y.validates.features` is **unchanged** ✗

Symmetric gap exists for `--adr ADR-Z` (which does not update
`ADR-Z.features`). The existing TC-inference path inside
`commands::feature_write::feature_link` *does* reciprocate
`validates.features` on **inferred** TCs (those reached transitively
through an ADR link), so explicit `--test` and `--adr` are the
asymmetric outlier. The graph appears half-linked: the feature can
walk to the TC, but the TC has no record of being a validator.

### Why fix together

Both gaps share the FT-046 root cause (MCP write parity not finished)
and both fix in the same slice layer (`src/feature/`). FT-046 closed
the same hole for ADR lifecycle transitions and explicitly named both
follow-ons. The fix pattern is identical: route MCP handlers through
the existing or newly-extracted pure planning functions
(`feature::plan_status_change`, `tc::plan_status_change`, new
`feature::plan_link`) that the CLI adapters already call.

---

## Depends on

- **FT-021** — MCP Server. Owns the tool surface this feature fixes.
- **FT-046** — MCP Parity for ADR Lifecycle Operations. Established
  the pattern this feature applies to feature/TC status and to
  bidirectional linking.
- **FT-058** — Enforce TC Runner Configuration. The
  `plan_status_change` path already refuses `in-progress`
  transitions when linked TCs lack runner config; that gate must
  fire over MCP too.

---

## Scope of this feature

### In

1. **`product_feature_status` writes the requested status.** The MCP
   handler dispatches to `feature::plan_status_change` +
   `apply_status_change`. ADR-010 orphan-test cascade runs
   identically over MCP and CLI (when transitioning to `abandoned`,
   tests with this feature as their only validator have it removed
   from `validates.features`).
2. **`product_test_status` writes the requested status.** The MCP
   handler dispatches to `tc::plan_status_change` +
   `apply_status_change`.
3. **FT-058 runner-config gate fires over MCP.** Transitions into
   a status that requires runner config (`in-progress`, etc.) over
   MCP return the same `TcRunnerMissing` error the CLI returns,
   with the offending TC list. The file is not modified.
4. **FT-055 / W030 completeness gate fires over MCP.** Transitions
   to `in-progress` on a feature missing required body sections
   return an error when `[features].completeness-severity = "error"`,
   matching CLI behaviour.
5. **Consistent JSON return shape for status tools.** Success returns
   `{ id, status, orphaned-tests: [...] }` for features (the
   `orphaned-tests` array is empty for non-abandonment transitions)
   and `{ id, status }` for tests. The `note` field is removed —
   there is no longer a need to redirect to the CLI.
6. **Error envelope on failure.** Unknown ID returns the standard
   `NotFound` error. Invalid status strings return a parse error.
   No silent no-ops.
7. **`handle_status_update` is deleted.** The shared stub is
   replaced by two purpose-built handlers
   (`handle_feature_status_update`, `handle_test_status_update`)
   routed individually from the dispatcher.
8. **Session tests in `tests/sessions/`.** Each scenario TC composes
   a temp repo via `product request apply`, drives the MCP tool
   under test through the compiled binary, and asserts on the
   post-write front-matter byte-for-byte against the equivalent
   CLI invocation.
9. **`product_feature_link` reciprocates back-references on link
   targets.** When `test: TC-Y` is provided, the same atomic batch
   that adds `TC-Y` to the feature's `tests:` array also adds the
   linking feature ID to `TC-Y.validates.features`. When
   `adr: ADR-Z` is provided, the same batch adds the linking feature
   ID to `ADR-Z.features`. The fix lands in the slice layer (a new
   pure `feature::plan_link` returning a batch of writes), so both
   CLI and MCP inherit it. The existing TC-inference reciprocation
   path is unchanged — explicit `--test` and `--adr` now match the
   behaviour transitive TC inference already performs.
10. **`product_feature_link` returns a structured writes report.**
    The current `{ id, linked: bool }` shape is replaced by
    `{ id, writes: [{ path, kind }], reciprocated: [{ id, field }] }`
    so callers can see exactly which files changed and which back-
    references were filled in. CLI text output is unchanged.
11. **`product_feature_link` validates link targets exist before
    writing.** Today the MCP handler accepts a non-existent
    `test: TC-999` and writes the dangling reference into the
    feature. After this feature, unknown link targets return a
    `NotFound` error before any write, matching the CLI's
    feature-side cycle/existence checks in `link_dep`.

### Out

- **ADR status transitions over MCP.** Already fixed in FT-046.
- **Dependency status transitions over MCP.** Out of scope —
  dependencies use a different field set (`status: active |
  evaluating | deprecated | migrating`) and are not affected by
  the FT-046 follow-on observation. Track separately if needed.
- **Bulk status updates.** One artifact per call. Multi-artifact
  atomic status updates go through `product_request_apply`.
- **Changes to front-matter schema.** No new fields; no new error
  codes beyond reusing the existing ones.
- **Removing reciprocal back-references on unlink.**
  `product_feature_link` is add-only today. Symmetric removal would
  need a new tool (`product_feature_unlink`) and is tracked
  separately if needed — the back-fill direction is the user-facing
  bug.
- **Dependency reciprocation.** `dep:` links from features to DEPs
  use `link_dep` which already validates the target's existence
  and cycle properties; no behavioural change there. Reciprocation
  of `DEP-N.features` is out of scope for this feature.

---

## Tool surface changes

### `product_feature_status` — current vs. new

| Case | Current behaviour | New behaviour |
|---|---|---|
| `planned → in-progress` (TCs configured) | returns `{ status, note: "Use CLI…" }`, **file unchanged** | writes `status: in-progress`; returns `{ id, status, orphaned-tests: [] }` |
| `planned → in-progress` (TCs missing runner) | returns OK, **file unchanged** | returns `TcRunnerMissing` error with offending TC list; file unchanged |
| `* → complete` | returns OK, **file unchanged** | writes `status: complete` |
| `* → abandoned` | returns OK, **file unchanged** | writes `status: abandoned`; orphan-test cascade runs (ADR-010); returns `orphaned-tests` array of updates |
| unknown ID | returns `{ id: "FT-999", status, note: "…" }` | returns `NotFound` error |
| invalid status string | returns `{ id, status: <garbage>, note: "…" }` | returns parse error |

### `product_test_status` — current vs. new

| Case | Current behaviour | New behaviour |
|---|---|---|
| `unimplemented → implemented` | returns `{ status, note: "Use CLI…" }`, **file unchanged** | writes `status: implemented` |
| `* → passing` | returns OK, **file unchanged** | writes `status: passing` |
| `* → failing` | returns OK, **file unchanged** | writes `status: failing` |
| `* → unrunnable` | returns OK, **file unchanged** | writes `status: unrunnable` |
| unknown ID | returns `{ id: "TC-999", status, note: "…" }` | returns `NotFound` error |
| invalid status string | returns `{ id, status: <garbage>, note: "…" }` | returns parse error |

### `product_feature_link` — current vs. new

| Case | Current behaviour | New behaviour |
|---|---|---|
| `--test TC-Y` | adds `TC-Y` to feature's `tests:`; **TC-Y.validates.features unchanged** | adds `TC-Y` to feature's `tests:` **and** adds feature ID to `TC-Y.validates.features` in the same atomic batch |
| `--adr ADR-Z` | adds `ADR-Z` to feature's `adrs:`; **ADR-Z.features unchanged** | adds `ADR-Z` to feature's `adrs:` **and** adds feature ID to `ADR-Z.features` in the same atomic batch |
| TC-inference path (`--adr` triggers transitive TC link) | already reciprocates `validates.features` on inferred TCs | unchanged — this is the existing behaviour the explicit path now matches |
| target ID does not exist | accepts silently (writes feature side, no validation) | returns `NotFound` error before any write |
| target already linked | no-op write, `linked: false` | no-op write, returns `{ writes: [], reciprocated: [] }` |

---

## Implementation notes

- **`src/mcp/registry.rs`** — split the combined match arm

  ```rust
  "product_feature_status" | "product_test_status" => {
      write_handlers::handle_status_update(args)
  }
  ```

  into two:

  ```rust
  "product_feature_status" => {
      write_handlers::handle_feature_status_update(args, graph)
  }
  "product_test_status" => {
      write_handlers::handle_test_status_update(args, graph)
  }
  ```

- **`src/mcp/write_handlers.rs`** — delete `handle_status_update`.
  Add two handlers that mirror `commands::feature_write::feature_status`
  and the equivalent TC adapter. The handlers do not need to
  re-implement the W030 completeness gate or FT-058 runner gate —
  `feature::plan_status_change` already enforces them and returns
  `ProductError::TcRunnerMissing` / similar. The MCP handler just
  needs to propagate the error through the `Result<Value, String>`
  return type using `format!("{}", e)`.

- **`src/tc/status_change.rs`** — confirm a `plan_status_change` +
  `apply_status_change` pair exists and is symmetric with the
  feature slice. If `apply_status_change` does not yet exist in the
  TC slice, add it; the existing `commands::tc_write` adapter is
  the reference.

- **`src/feature/link.rs`** (new module in the feature slice) —
  extract a pure `plan_link(graph, feature_id, adr, test, dep) ->
  LinkPlan` function. `LinkPlan` carries: (a) updated feature
  front-matter, (b) updated TC front-matter (when `test` is set),
  (c) updated ADR front-matter (when `adr` is set), (d) cycle-check
  result for `dep`. `apply_link` is a thin `write_batch_atomic`
  wrapper. Both the CLI adapter `commands::feature_write::feature_link`
  and the MCP handler `write_handlers::handle_feature_link` migrate
  to this pair.

- **`src/mcp/write_handlers.rs::handle_feature_link`** — current
  handler builds `front: FeatureFrontMatter` ad-hoc and writes only
  the feature file. Replace with a call to `feature::plan_link` +
  `apply_link`. The existing FT-062 `plan_depends_on_edit` path for
  the `feature:` argument stays — `plan_link` composes with it.

- **`src/commands/feature_write.rs::feature_link`** — currently
  intermixes link logic, TC inference, and interactive prompting.
  Refactor so the non-interactive write paths go through `plan_link`;
  the interactive TC-inference prompt remains in the CLI adapter
  (out of slice, by design — it reads stdin).

- **Concurrency.** No new concerns. `registry::call_tool` already
  acquires `RepoLock` for `requires_write: true` tools; all
  affected tools already declare `requires_write: true` in
  `src/mcp/tools/write.rs`.

- **Schema changes.** None on the input side. The
  `product_feature_link` output JSON shape changes (`linked: bool`
  is dropped in favour of `writes: [...]` and `reciprocated: [...]`)
  — that is a breaking change in the MCP response only; the CLI
  text output is unchanged.

- **Runner config.** Every TC in this feature gets
  `runner: cargo-test` and `runner-args: tc_XXX_snake_case` at the
  moment the test is written, per CLAUDE.md.

---

## Acceptance criteria

An MCP client can:

1. Call `product_feature_status` with a valid feature ID and status
   and observe the on-disk front-matter `status:` field updated to
   match (parity with CLI byte-for-byte) — **TC-778**.
2. Call `product_feature_status` to abandon a feature and observe
   the orphan-test cascade run identically to the CLI, with the
   updated TC files written in the same atomic batch — **TC-780**.
3. Call `product_feature_status` with `in-progress` against a
   feature whose linked TCs lack runner config and receive a
   `TcRunnerMissing` error naming every offender; the feature file
   is not modified — **TC-781**.
4. Call `product_test_status` with a valid TC ID and status and
   observe the on-disk front-matter `status:` field updated —
   **TC-779**.
5. Call either status tool with an unknown ID and receive a
   `NotFound` error — **TC-782**.
6. Call either status tool with an invalid status string and
   receive a parse error — **TC-783**.
7. The string `"Use CLI for status updates with full side-effects"`
   is absent from the codebase — **TC-784** (invariant grep guard).
8. Call `product_feature_link` with `test: TC-Y` and observe both
   the feature's `tests:` array updated **and**
   `TC-Y.validates.features` updated in the same atomic batch —
   **TC-785**.
9. Call `product_feature_link` with `adr: ADR-Z` and observe both
   the feature's `adrs:` array updated **and** `ADR-Z.features`
   updated in the same atomic batch — **TC-786**.
10. The MCP response from `product_feature_link` includes a `writes`
    array enumerating every file touched and a `reciprocated` array
    naming the back-references filled in — **TC-787**.
11. `product graph check` exits 0 after each successful transition.
12. `cargo t`, `cargo clippy -- -D warnings -D clippy::unwrap_used`,
    and `cargo build` all pass.
13. Every TC in the feature has `runner: cargo-test` and
    `runner-args` matching the Rust test function name.

See **TC-788** (exit-criteria) for the consolidated check-list.

---

## Functional Specification

### Inputs

- `product_feature_status` MCP tool: `{ id: String, status: String }`.
- `product_test_status` MCP tool: `{ id: String, status: String }`.
- `product_feature_link` MCP tool: `{ id: String, adr?: String,
  test?: String, dep?: String, feature?: String }` (unchanged input
  schema).

### Outputs

- Status tools, on success: JSON object with `{ id, status }` (and
  `orphaned-tests: [...]` for feature abandonment).
- Link tool, on success: JSON object with
  `{ id, writes: [{ path, kind }], reciprocated: [{ id, field }] }`.
- Any tool, on failure: JSON-RPC error envelope with `ProductError`
  text.

### State

- Front-matter `status:` field on the targeted feature / TC file
  (status tools).
- For feature abandonment: `validates.features` arrays on any TC
  whose only validating feature was the abandoned one.
- For link: feature's `tests:` / `adrs:` arrays **and** the target
  TC's `validates.features` / target ADR's `features` array.

### Behaviour

- Status tools dispatch to the existing pure slice functions; apply
  via `fileops::write_file_atomic` (single file) or
  `write_batch_atomic` (orphan-test cascade).
- Link tool dispatches to the new `feature::plan_link` +
  `apply_link`; apply is always via `write_batch_atomic` because the
  reciprocal write can touch multiple files in one call.
- All tools hold the repo write lock for the duration of the call
  (already enforced by `registry::call_tool`).

### Invariants

- No success response without a corresponding on-disk write.
- The on-disk result of MCP and CLI invocations on byte-identical
  inputs is byte-identical (parity invariant).
- For every `feature.tests` membership written, the corresponding
  `tc.validates.features` membership is written in the same atomic
  batch (link reciprocity invariant). Symmetric for `feature.adrs`
  / `adr.features`.
- FT-058 runner gate and FT-055 / W030 completeness gate enforce
  equally over MCP and CLI.

### Error handling

- `NotFound` for unknown IDs (both status and link tools).
- `TcRunnerMissing` for `in-progress` transitions blocked by FT-058.
- W030 error for `in-progress` transitions blocked by FT-055 when
  completeness severity is `error`.
- Parse error for unknown status strings.

### Boundaries

- This feature does not change ADR or dependency status handling.
- This feature does not introduce new error codes.
- This feature does not change the JSON input schemas of the
  affected tools (only the link tool's output shape changes).

## Out of scope

- ADR and dependency status transitions (see "Out" above).
- Bulk status updates (use `product_request_apply`).
- New error codes or schema changes.
- A symmetric `product_feature_unlink` tool.
