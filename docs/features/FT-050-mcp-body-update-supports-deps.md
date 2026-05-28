---
id: FT-050
title: MCP body_update Supports Dependencies
phase: 5
status: complete
depends-on:
- FT-032
- FT-046
adrs:
- ADR-030
- ADR-031
- ADR-038
tests:
- TC-620
- TC-621
- TC-622
domains:
- api
- data-model
domains-acknowledged:
  ADR-018: Predates the 2026-04-22 scope promotion of ADR-018 to cross-cutting. Test coverage reflects the property/session/benchmark strategy as it existed when this feature shipped; not retroactively reclassified.
  ADR-048: Predates ADR-048; this feature does not author the canonical .product/ folder layout. FT-057 owns the migration command and discovery fallback.
  ADR-047: Predates ADR-047; this feature does not author the functional-spec body convention. FT-055 owns the structural validator and W030 mechanics.
  ADR-043: MCP handlers live in `src/mcp/`; the feature adds one branch to the existing prefix dispatcher and one helper mirroring the other three, rather than a new slice.
  ADR-040: body_update is an MCP write-path, not a verify-pipeline stage; no new stage hook is added and the LLM-boundary surface expands only by one accepted prefix.
  ADR-042: Dep bodies carry no tc-type partition; the body_update contract is identical for every artifact type and does not depend on the TC vocabulary.
  ADR-041: Deps participate in absence-TC semantics via FT-047; this feature only adds body-text editing, leaving removal / deprecation lifecycle untouched.
---

## Description

The `product_body_update` MCP tool lets an LLM rewrite the narrative body
of a feature, ADR, or TC in place without touching front-matter. It is the
only safe path for bulk body edits over MCP because it re-uses the
existing parser/renderer round-trip.

Today the handler dispatches on the prefix of the supplied ID and rejects
any prefix that is not `config.prefixes.feature / adr / test`. The
dependency prefix (`DEP-`) is missing, even though `render_dependency`
exists and deps carry a body like every other artifact. An LLM trying to
amend a dep's "Rationale" or "Migration plan" section currently has to
edit the file directly — defeating the atomic-write contract and bypassing
locking.

This feature closes that gap: `product_body_update` grows a fourth branch
for the dep prefix, calling `render_dependency` via the same atomic-write
+ locking path as the other three types.

Originates from GitHub issue #5 ("MCP body_update doesn't support deps").

---

## Depends on

- **FT-032** — Dependency Artifact Type. Defines the dep front-matter and
  body shape this feature edits.
- **FT-046** — MCP Parity for ADR Lifecycle Operations. Established the
  MCP-writes-must-match-CLI contract that this feature extends to deps.

---

## Scope of this feature

### In

1. **`handle_body_update` dep branch.** Add a fourth prefix check against
   `config.prefixes.dependency` that resolves the dep from
   `graph.deps`, calls `parser::render_dependency(&d.front, body)`, and
   writes atomically via `fileops::write_file_atomic`.
2. **`update_dep_body` helper** mirroring `update_feature_body` /
   `update_adr_body` / `update_test_body`. Single-responsibility, under
   15 lines.
3. **Tool schema update.** `product_body_update` in `src/mcp/tools.rs`
   names `DEP-NNN` in its `id` description alongside the other prefixes so
   discovery surfaces the new capability.
4. **Error parity.** Unknown prefixes still error with the existing
   message; `Dep not found` mirrors `Feature not found` / `ADR not found`
   / `TC not found` in wording.
5. **Unit + integration tests.** One per: success case, unknown-ID case,
   unknown-prefix case (regression).

### Out

- **Content-hash enforcement for deps.** Deps do not currently carry
  `content-hash` (ADR-032 is ADR-only). Out of scope for this feature.
- **Amend semantics for deps.** Unlike ADRs, deps have no "accepted" gate;
  body is editable at any status. No `product dep amend` analogue is
  introduced.
- **New MCP endpoint.** No `product_dep_body_update` — the existing
  `product_body_update` is prefix-dispatched and covers all four types.

---

## Commands

No new CLI subcommand. Surfaces through MCP:

- `product_body_update` — accepts `DEP-NNN` IDs in addition to `FT-`,
  `ADR-`, `TC-`.

CLI `product dep body` is out of scope (the feature is an MCP gap, and CLI
body edits are covered by direct file edits plus `product request`).

---

## Implementation notes

- **`src/mcp/write_handlers.rs`** — add the dep branch to
  `handle_body_update`. Add `update_dep_body(id, body, graph) -> Result`.
  Uses the same `write_file_atomic` path as the other three.
- **`src/mcp/tools.rs`** — update the `product_body_update` tool
  description to list `DEP-NNN` alongside `FT-`, `ADR-`, `TC-`.
- **No config changes.** The dep prefix is already in
  `config.prefixes.dependency`.
- **Tests (`src/mcp/tests.rs` or integration).** Construct a graph with
  one dep, call `handle_body_update` with a new body, read the file back,
  and assert front-matter is preserved, body is replaced, and the rest of
  the content round-trips byte-for-byte outside the body.

---

## Acceptance criteria

A developer running on a clean repository can:

1. Load a repo with `DEP-001`, call `product_body_update` over MCP with
   `id: "DEP-001"` and a new body, and observe the dep's file has the new
   body and unchanged front-matter (TC-620).
2. Call `product_body_update` with `id: "DEP-999"` (no such dep) and
   observe an error naming the missing dep (TC-621).
3. Call `product_body_update` with an unknown prefix (`FOO-001`) and
   observe the existing "Unknown artifact ID prefix" error, unchanged
   (TC-621).
4. Run `cargo test`, `cargo clippy -- -D warnings -D clippy::unwrap_used`,
   and `cargo build` and observe all pass.

See TC-622 (exit criteria) for the consolidated check-list.

---

## Follow-on work

- **Body-change audit in the request log.** `product_body_update` does
  not currently emit a `change` entry in `requests.jsonl`. Extending it to
  do so for all four types is a separate feature (covers FT / ADR / TC /
  DEP uniformly); tracked out-of-band.
- **Diff preview before write.** An MCP caller could request a preview of
  the rendered file before commit. Nice-to-have.

---

## Functional Specification

### Inputs

- **`product_body_update` MCP tool:** `id` (required — a `DEP-NNN` identifier, or any of the existing `FT-`, `ADR-`, `TC-` prefixes), `body` (required — the new prose text to replace the artifact's body below the front-matter delimiter).
- The dep file on disk identified by `id` — resolved via `graph.deps` to its file path, then read and written via `fileops::write_file_atomic`.
- The knowledge graph — read to resolve the dep and confirm it exists.

### Outputs

- **Success:** the dep file on disk has its body (the content below the `---` delimiter) replaced with the new text. Front-matter is preserved byte-for-byte. The MCP response mirrors the shape returned for feature, ADR, and TC body updates.
- **`DEP-NNN` not found:** structured error naming the missing dep (`"Dep not found: DEP-NNN"`), mirroring the wording of `"Feature not found"` / `"ADR not found"` / `"TC not found"`.
- **Unknown prefix:** the existing `"Unknown artifact ID prefix"` error, unchanged. The new dep branch is additive; the existing error path for unrecognised prefixes is not modified.

### State

- **Dep file on disk** — body text replaced atomically via `fileops::write_file_atomic` (ADR-015). Front-matter is preserved; only the prose below the `---` delimiter changes.
- **Advisory lock (ADR-015)** — acquired before the write, released after. Serialises concurrent MCP calls targeting the same file.
- No new persistent state. `requests.jsonl` is not appended to by `product_body_update` (audit log integration for body updates is a separate future feature).

### Behaviour

1. **Prefix dispatch.** `handle_body_update` checks the `id` prefix. The existing three branches (`config.prefixes.feature`, `config.prefixes.adr`, `config.prefixes.test`) are unchanged. A new fourth branch matches `config.prefixes.dependency`; it resolves the dep via `graph.deps`, calls `update_dep_body(id, body, graph)`, and returns the result.
2. **`update_dep_body` helper.** A new function mirroring `update_feature_body` / `update_adr_body` / `update_test_body` — under 15 lines. It looks up the dep, calls `parser::render_dependency(&d.front, body)` to produce the updated file content, and writes it via `fileops::write_file_atomic`. Returns a success or error result.
3. **Error on unknown dep ID.** If the dep prefix is matched but the ID does not exist in `graph.deps`, returns `"Dep not found: DEP-NNN"`. No file is written.
4. **Error on unknown prefix.** If the prefix matches none of the four known prefixes, the existing `"Unknown artifact ID prefix"` error path is taken, unchanged. No file is written.
5. **Tool schema update.** The `product_body_update` tool description in `src/mcp/tools.rs` is updated to list `DEP-NNN` alongside `FT-`, `ADR-`, `TC-` in the `id` parameter description, so MCP discovery surfaces the new capability.

### Invariants

- Front-matter is always preserved byte-for-byte on a successful body update; only the prose body changes.
- A successful `product_body_update` with a `DEP-NNN` id produces the same file content as a direct file edit followed by `fileops::write_file_atomic` with the same body text.
- The existing behaviour for `FT-`, `ADR-`, and `TC-` prefixes is unchanged; the dep branch is purely additive.
- `product_body_update` for any unknown prefix — including `DEP-` IDs that do not exist — returns an error and writes nothing to disk.

### Error handling

| Condition | Error |
|---|---|
| `id` has the dep prefix and the dep exists | Success; body replaced atomically |
| `id` has the dep prefix but the dep does not exist in the graph | `"Dep not found: DEP-NNN"` error; no write |
| `id` has an unknown prefix (not FT-, ADR-, TC-, or DEP-) | Existing `"Unknown artifact ID prefix"` error; no write |

### Boundaries

- Content-hash enforcement for deps is not implemented: deps do not currently carry `content-hash` (that field is ADR-specific per ADR-032). This feature does not add content-hash to deps.
- Amend semantics for deps are not introduced: unlike ADRs, deps have no "accepted" gate; body is editable at any status. No `product dep amend` analogue is added.
- A new `product_dep_body_update` MCP endpoint is not created: the existing `product_body_update` is prefix-dispatched and covers all four types after this feature.
- `requests.jsonl` audit log integration for body updates is out of scope: extending `product_body_update` to append log entries for all four artifact types is a separate future feature.
- CLI `product dep body` is not added: the feature closes an MCP gap; direct file edits plus `product request` cover CLI body editing for deps.

## Out of scope

- Content-hash enforcement for deps: deps do not carry `content-hash`; this feature does not change that.
- Amend semantics for deps: no "accepted" gate exists on deps; no `product dep amend` analogue is needed.
- A separate `product_dep_body_update` MCP endpoint: the existing `product_body_update` prefix dispatcher is extended instead.
- Audit log (`requests.jsonl`) integration for body updates: extending `product_body_update` to append log entries covers all four types uniformly and is a separate future feature.
- CLI `product dep body` subcommand: the MCP gap is the scope; CLI body editing for deps is covered by direct file edits and `product request`.
