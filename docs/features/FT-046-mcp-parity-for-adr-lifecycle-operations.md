---
id: FT-046
title: MCP Parity for ADR Lifecycle Operations
phase: 5
status: complete
depends-on:
- FT-021
- FT-034
adrs:
- ADR-015
- ADR-020
- ADR-021
- ADR-032
- ADR-038
- ADR-040
tests:
- TC-577
- TC-578
- TC-579
- TC-580
- TC-581
- TC-582
- TC-583
- TC-584
- TC-585
domains:
- api
- error-handling
domains-acknowledged:
  ADR-043: Predates ADR-043; feature command adapters were written before the slice+adapter pattern was formalised and are not retroactively refactored.
  ADR-041: MCP lifecycle parity does not introduce or alter removes/deprecates fields or absence TCs; scope is ADR status/amend transitions only.
  ADR-018: Predates the 2026-04-22 scope promotion of ADR-018 to cross-cutting. Test coverage reflects the property/session/benchmark strategy as it existed when this feature shipped; not retroactively reclassified.
  ADR-048: Predates ADR-048; this feature does not author the canonical .product/ folder layout. FT-057 owns the migration command and discovery fallback.
  ADR-042: MCP lifecycle parity does not introduce or alter TC types; wiring is orthogonal to the type system.
  ADR-047: Predates ADR-047; this feature does not author the functional-spec body convention. FT-055 owns the structural validator and W030 mechanics.
---

## Description

Close the MCP write-path gaps that prevent a spec-authoring session from completing ADR lifecycle work without dropping back to the CLI. Two concrete gaps, both discovered while implementing FT-044 / FT-045:

1. **`product_adr_amend` cannot actually amend a body via MCP.** The tool records an audit entry but requires the on-disk body to already differ from the stored content-hash. `product_body_update` refuses accepted ADR bodies with `"Cannot modify body of accepted ADR ADR-XXX. Use product adr amend..."`. That creates a dead loop: the only tool that lets a body change is `amend`, but `amend` only records the audit entry *after* the body has already changed. Agents without file-write access cannot amend accepted ADRs.

2. **`product_adr_status` accepts the request, returns `{ status: <new> }`, but does not write the file.** The note `"Use CLI for status updates with full side-effects"` is advisory. The status field in front-matter stays unchanged. This silently breaks every agent workflow that tries to progress an ADR's lifecycle.

This feature brings MCP to parity with the CLI for every ADR lifecycle transition **except accepting an ADR** — that one stays manual by design (per ADR-032 governance: a sealing action deserves a human-in-the-loop with a local CLI that can print impact analysis and confirm the content hash).

---

## Depends on

- **FT-021** — MCP Server. Owns the tool surface this feature extends.
- **FT-034** — Content Hash Immutability. Owns the `content-hash` + `amendments` front-matter fields this feature writes to.

---

## Scope of this feature

### In

1. **`product_adr_amend` accepts an optional `body` parameter.** When `body` is provided, the tool atomically: (a) writes the new body to disk, (b) computes the new content-hash, (c) appends an `Amendment { date, reason, previous_hash }` entry to the front-matter, (d) updates `content-hash` to the new hash. All in one MCP call. If `body` is omitted, behaviour is unchanged (records an amendment against whatever body is already on disk — the legacy path for direct-file-edit workflows).
2. **`product_body_update` accepts accepted ADR bodies when routed through `amend`.** The error message stays the same, but the authoritative recommendation is now `product_adr_amend` with a `body` parameter. No behavioural change for non-accepted ADRs.
3. **`product_adr_status` actually writes non-`accepted` transitions.** All of these work identically over MCP and CLI: `proposed → superseded`, `proposed → abandoned`, `accepted → superseded`, `accepted → abandoned`. When `by: ADR-YYY` is provided on supersession, the target ADR's `supersedes` array is updated bidirectionally in the same atomic write batch (same behaviour as the existing CLI `adr_supersede`).
4. **`product_adr_status` refuses `accepted` over MCP with an explicit error.** The error message names the CLI command to run: `"Accepting an ADR is a manual step. Run: product adr status ADR-XXX accepted"`. No silent success, no "advisory" note. Exit behaviour: the tool returns an error result; the ADR file is not modified.
5. **`product_adr_status` refuses any demotion from `accepted → proposed`.** Already-sealed ADRs cannot be unsealed. This preserves the ADR-032 immutability invariant.
6. **`product_adr_amend` refuses to change the `status` field.** Amendments are body-only audit records. Any attempt to pass `status` through `amend` (directly or through a body field) returns an error. Status transitions go through `product_adr_status` (and `accepted` still goes through the CLI).
7. **Consistent return shape.** Every lifecycle tool returns the same JSON envelope on success: `{ id, status, content-hash, amendments: [...] }` where `amendments` is the updated audit array. No more `{ note: "Use CLI..." }` divergence.
8. **Session tests in `tests/sessions/`.** Each scenario TC (TC-577 through TC-584) composes a temp repo via `product request apply`, drives the MCP tool under test through the compiled binary, and asserts on the post-write front-matter + amendments array + content-hash.

### Out

- **`accepted` over MCP.** Deliberately out of scope. See the rationale on ADR-032 governance.
- **Feature / TC / DEP status transitions.** Already work correctly over MCP. This feature is scoped to ADRs because the gap is specific to content-hashed artifacts.
- **Bulk amendment across multiple ADRs.** One ADR per `product_adr_amend` call. Multi-ADR atomic amendment is a follow-on if the need arises.
- **Changes to the on-disk front-matter schema.** No new fields; the existing `content-hash` and `amendments` shape is unchanged.

---

## Tool surface changes

### `product_adr_amend` — current vs. new

| Parameter | Current | New |
|---|---|---|
| `id` | required | required |
| `reason` | required | required |
| `body` | not accepted | **optional** — when present, atomic body replace + amend |

**New success response:**

```json
{
  "id": "ADR-019",
  "status": "accepted",
  "content-hash": "sha256:abc123…",
  "amendments": [
    { "date": "2026-04-18T06:42:01Z", "reason": "Remove internal LLM call per ADR-040", "previous-hash": "sha256:def456…" }
  ]
}
```

**Error cases:**

- `E017 amendment-nothing-changed` — `body` omitted and the on-disk body already matches the stored hash.
- `E018 amendment-not-accepted` — the ADR is not in `accepted` status.
- `E019 amendment-carries-status` — the request attempted to change `status` via amend.

### `product_adr_status` — current vs. new

| Case | Current behaviour | New behaviour |
|---|---|---|
| `proposed → accepted` | returns `{ status: accepted, note: "Use CLI…" }`, **file unchanged** | returns `E020 status-accepted-is-manual` error; file unchanged |
| `proposed → superseded` | returns OK, **file unchanged** | writes `status: superseded`; bidirectional supersession with `by` target |
| `proposed → abandoned` | returns OK, **file unchanged** | writes `status: abandoned` |
| `accepted → superseded` | returns OK, **file unchanged** | writes `status: superseded`; bidirectional; preserves content-hash |
| `accepted → abandoned` | returns OK, **file unchanged** | writes `status: abandoned`; preserves content-hash |
| `accepted → proposed` | returns OK, **file unchanged** | returns `E021 status-cannot-demote-accepted` error |

**Success response:**

```json
{
  "id": "ADR-019",
  "status": "superseded",
  "superseded-by": ["ADR-040"],
  "content-hash": "sha256:abc123…"
}
```

---

## Implementation notes

- **`src/mcp/tools/adr.rs`** — this is where the tool handlers live. The current `adr_status_tool` returns an advisory `note` without calling the write path. Replace it so it dispatches to the same function the CLI uses (`commands::adr::adr_status` or its extracted helper). The only branch that differs: when `new_status == Accepted`, return `Err(ProductError::ConfigError("E020 …"))` instead of calling through.
- **`src/commands/adr.rs::adr_amend`** — refactor to accept an optional `new_body: Option<&str>`. When `Some(body)`, the function writes the body via `fileops::write_file_atomic` **before** computing the new hash, so the normal "body changed → new hash differs from stored → amendment valid" path just works. Check for the `E019 amendment-carries-status` case by rejecting any payload that also carries a `status` or `amendments` field.
- **`src/mcp/tools/adr.rs::adr_amend_tool`** — wire the new `body` parameter through to the helper. Schema update: `body: { type: "string", nullable: true }` in the tool's JSON schema.
- **Error codes.** New E-codes E017, E018, E019, E020, E021 all follow ADR-013 format. Register them in `src/error.rs` and document them in `docs/guide/FT-046-mcp-parity-for-adr-lifecycle-operations.md` when the guide is generated.
- **Parity tests.** Each scenario TC runs the same transition through CLI and then through MCP against a fresh temp repo, asserting the on-disk result is byte-identical. That's the key correctness property: MCP and CLI produce the same file.
- **Runner config.** Every TC in this feature gets `runner: cargo-test` and `runner-args: tc_XXX_snake_case` at the moment the test is written, per CLAUDE.md.

---

## Acceptance criteria

A spec-authoring agent connected over MCP can:

1. Call `product_adr_amend` with `id`, `reason`, and `body` and observe the on-disk ADR body replaced, a new amendment appended, and content-hash updated — all in one round trip (TC-577).
2. Call `product_adr_amend` with a payload that also carries a `status` field and receive an `E019 amendment-carries-status` error, with the ADR file unchanged (TC-578).
3. Call `product_adr_amend` with the body already matching the stored hash and receive `E017 amendment-nothing-changed`, with the file unchanged (TC-579).
4. Call `product_adr_status` for any non-`accepted` transition and observe the file on disk updated (TC-580, TC-582, TC-583).
5. Call `product_adr_status` with `accepted` and receive `E020 status-accepted-is-manual` with an explicit hint naming the CLI command — file unchanged (TC-581).
6. Call `product_adr_status` to demote an accepted ADR to `proposed` and receive `E021 status-cannot-demote-accepted` — file unchanged (TC-584).
7. When `product_adr_status` succeeds with supersession, the target ADR's `supersedes` array is updated in the same atomic batch (TC-582 asserts both files post-write).
8. `product graph check` exits 0 after each successful transition.
9. `cargo test`, `cargo clippy -- -D warnings -D clippy::unwrap_used`, and `cargo build` all pass.
10. Every TC in the feature (TC-577 through TC-585) has `runner: cargo-test` and `runner-args` matching the Rust test function name.

See TC-585 (exit criteria) for the consolidated check-list.

---

## Follow-on work

- **Feature/TC/DEP parity audit.** This feature closes the ADR gap. A follow-on should walk the remaining MCP tools (`product_feature_status`, `product_test_status`, etc.) and confirm none of them silently drop writes. If any do, add them to a follow-on feature with the same pattern.
- **Content-hash seal audit via MCP.** Once ADR-032 governance is comfortable with MCP being authoritative for amendments, consider exposing `product_adr_rehash` as a first-class MCP tool for sealing ADRs that predate content-hash.
- **Optional: MCP-visible `product_adr_accept_preview`.** A read-only tool that returns what `product adr status ADR-XXX accepted` *would* do (impact analysis, computed hash, affected features), without performing the write. Lets an agent prepare the human for the manual CLI step.

---

## Functional Specification

### Inputs

- **`product_adr_amend`** MCP tool: `id` (required), `reason` (required), `body` (optional string — when present, the new body text to replace the ADR's prose below the front-matter delimiter).
- **`product_adr_status`** MCP tool: `id` (required), `new_status` (required — one of `proposed`, `accepted`, `superseded`, `abandoned`), `by` (optional — target ADR ID for supersession).
- The ADR file on disk identified by `id` — read and written via `fileops::write_file_atomic` (ADR-015 atomic writes).
- The knowledge graph — read to resolve the target ADR and any supersession target.

### Outputs

- **`product_adr_amend` success:** JSON envelope `{ id, status, content-hash, amendments: [...] }` where `amendments` is the updated array including the new entry `{ date, reason, previous-hash }`.
- **`product_adr_amend` error:** structured error with the appropriate E-code and message; ADR file unchanged.
- **`product_adr_status` success (non-`accepted`):** JSON envelope `{ id, status, content-hash }` plus `superseded-by` when the transition is supersession. Both the target ADR and the `by` target (if provided) are updated in the same atomic write batch.
- **`product_adr_status` error:** structured error with E-code and message; ADR file(s) unchanged.
- All success responses share the same envelope shape — no `{ note: "Use CLI..." }` advisory divergence.

### State

- **ADR front-matter on disk** — `product_adr_amend` updates `content-hash` and appends to the `amendments` array; the body text below the front-matter delimiter is replaced when `body` is provided.
- **Supersession target's front-matter on disk** — when `product_adr_status` performs a supersession with `by: ADR-YYY`, the target ADR's `supersedes` array is updated in the same atomic batch.
- **Advisory lock (ADR-015)** — acquired before any write, released after. Serialises concurrent MCP calls.
- No new persistent state is introduced. The existing `content-hash` + `amendments` front-matter shape (from FT-034) is written to; this feature does not add new fields.

### Behaviour

1. **`product_adr_amend` with `body`:** (a) acquires advisory lock; (b) writes the new body to disk via `fileops::write_file_atomic`; (c) computes the new content-hash over the updated body; (d) appends `{ date, reason, previous-hash }` to the `amendments` array; (e) writes the updated front-matter; (f) releases lock; (g) returns the success envelope. The "body changed → new hash differs from stored → amendment valid" check is satisfied automatically because the write precedes the hash computation.
2. **`product_adr_amend` without `body` (legacy path):** records an amendment against whatever body is already on disk. Returns E017 if the on-disk body already matches the stored content-hash (nothing changed to amend).
3. **`product_adr_status` — non-`accepted` transitions:** validates the transition is legal (see error table), acquires the lock, writes the `status` field (and `superseded-by` / `supersedes` for supersession), releases the lock, returns the success envelope. When supersession involves `by: ADR-YYY`, the `by` target's `supersedes` array is updated in the same atomic batch.
4. **`product_adr_status accepted` over MCP:** returns E020 immediately; the ADR file is not modified. The error message names the CLI command to run.
5. **`product_adr_status accepted → proposed` demotion:** returns E021; the ADR file is not modified. Sealed ADRs cannot be unsealed.
6. **Parity contract:** each lifecycle tool produces the same on-disk result as the equivalent CLI command. Session tests (TC-577–TC-584) run both paths against identical temp repos and assert byte-identical file output.

### Invariants

- A successful `product_adr_amend` always results in `content-hash` reflecting the body that is currently on disk.
- `product_adr_amend` never changes the `status` field; any attempt to pass `status` in the request returns E019 before any write.
- `product_adr_status accepted` over MCP never modifies the ADR file; it always returns E020.
- `product_adr_status` with a demotion from `accepted` to `proposed` never modifies the ADR file; it always returns E021.
- `product graph check` exits 0 after every successful lifecycle transition (verified by session tests).
- MCP and CLI produce byte-identical ADR files for the same lifecycle transition (parity contract).

### Error handling

| Code | Tool | Condition |
|---|---|---|
| E017 | `product_adr_amend` | `body` omitted and the on-disk body already matches the stored content-hash (nothing changed) |
| E018 | `product_adr_amend` | The ADR is not in `accepted` status (amendment is for accepted ADRs only) |
| E019 | `product_adr_amend` | The request attempts to change the `status` field via amend |
| E020 | `product_adr_status` | `new_status` is `accepted`; accepting an ADR requires the CLI |
| E021 | `product_adr_status` | Demotion from `accepted` to `proposed` is forbidden |

All error responses leave the ADR file(s) unchanged. Error messages for E020 name the specific CLI command to run.

### Boundaries

- `accepted` over MCP is deliberately excluded (ADR-032 governance: a sealing action deserves a human-in-the-loop via the local CLI that can print impact analysis and confirm the content hash). This is a permanent boundary, not a temporary gap.
- Feature, TC, and DEP status transitions are already correct over MCP; this feature is scoped to ADRs because the gap is specific to content-hashed artifacts.
- Bulk amendment across multiple ADRs in one call is not supported; one ADR per `product_adr_amend` call.
- No new front-matter fields are introduced; the existing `content-hash` and `amendments` shape is written to unchanged.

## Out of scope

- `accepted` over MCP: deliberately excluded per ADR-032 governance. A human-in-the-loop CLI step is required for the sealing action.
- Feature, TC, and DEP status transitions: already work correctly over MCP; this feature is ADR-specific.
- Bulk amendment across multiple ADRs in one call: one ADR per `product_adr_amend` invocation. Multi-ADR amendment is a follow-on if needed.
- Changes to the on-disk front-matter schema: the existing `content-hash` and `amendments` shape is used as-is.
- `product_adr_accept_preview` MCP tool (read-only impact preview before the manual CLI accept step): a possible follow-on, not part of this feature.
