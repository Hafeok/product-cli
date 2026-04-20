---
id: FT-050
title: MCP body_update Supports Dependencies
phase: 5
status: planned
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
