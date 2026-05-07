---
id: FT-062
title: MCP Parity for Feature `depends-on` and Strict Request Shape Validation
phase: 5
status: planned
depends-on:
- FT-018
- FT-021
- FT-038
- FT-041
adrs:
- ADR-013
- ADR-020
- ADR-037
- ADR-038
- ADR-043
tests: []
domains:
- api
- error-handling
- testing
domains-acknowledged: {}
---

## Description

Close two related defects in the MCP write surface that, together, make feature `depends-on` invisible to remote agents and silently mask the gap.

**Defect 1 — `depends-on` has no MCP setter.** The MCP write surface has dedicated tools for every other foundational link a feature owns (`adrs`, `tests`, `domains`, `domains-acknowledged`, `status`, `body`) but none for `depends-on`. The CLI command `product feature link --dep FT-YYY` already exists and performs cycle detection (`src/commands/feature_write.rs::link_dep`); the MCP `product_feature_link` handler in `src/mcp/write_handlers.rs` only reads the `adr` and `test` arguments and silently ignores anything else. FT-041's body explicitly notes this gap: *"the current tool surface has no MCP setter for feature `depends-on`, so they are documented here until FT-041 itself delivers the request interface capable of setting that field."* In practice, FT-041's request interface does not detect the gap because of Defect 2.

**Defect 2 — `product_request_apply` accepts unknown shapes silently.** The request parser (`src/request/parse.rs`) only consumes the recognised top-level keys (`type`, `schema-version`, `reason`, `artifacts`, `changes`); any other top-level key is silently dropped. Inside a `change` block, `mutation.field` is treated as an opaque dot-path and is never validated against the front-matter schema of the target artifact. A request that mistypes `depends-on` as `dependsOn` — or that wraps the intended mutation in a wrapper key like `update:` or `patch:` — is currently accepted as `valid: true` with `mutations: 0` (when the unknown wrapper hides the real change) or applied to a junk field name (when the typo lands inside a real `mutations[]` entry). Either way, the agent sees success and the graph silently diverges from intent.

The combination is what bites in practice: an agent looking for a way to set `depends-on` over MCP correctly reaches for `product_request_apply` (FT-041's documented mechanism), guesses at a shape, and the apply pipeline reports success while doing nothing. The fix is to add the missing granular setter **and** to tighten the request parser so unknown shapes fail loudly.

---

## Depends on

- **FT-021** — MCP Server. Owns the tool surface this feature extends.
- **FT-038** — Front-Matter Field Management. The pattern (`feature domain`, `feature acknowledge`, `adr supersede`) the new `depends-on` setter mirrors. New tool follows ADR-037's idempotent add/remove + cycle-detection contract.
- **FT-041** — Product Request — Unified Write Interface. Owns the request parser/validator that Defect 2 lives in. Tightening it requires a schema-version bump path (handled additively here — see Functional Specification → Compatibility).
- **FT-018** — Validation and Graph Health. Owner of the E-class error code registry; this feature adds **E025** (unknown change-shape key) and **E026** (unknown mutation field for target artifact type).

---

## Scope of this feature

### In

1. **`product_feature_depends_on` MCP tool + `product feature depends-on` CLI command.** Idempotent `add[]` / `remove[]` semantics, mirroring `product_feature_domain`. Performs the same cycle-detection check that `feature_write.rs::link_dep` already does, returning **E003 dependency-cycle** on violation. Validates that every added ID exists in the graph, returning **E002 broken-link** on miss. Bidirectional behaviour is **not** added (depends-on is not a two-way relationship; the back-pointer is computed by graph inversion at read time).
2. **Extend `product_feature_link` MCP tool with `feature: FT-YYY` argument.** For backwards compatibility with the existing tool name and the patterns documented in the AGENTS.md `Key MCP Tools` table, `product_feature_link` accepts an optional `feature` argument that is delegated to the same handler as `product_feature_depends_on` add. Idempotent — already-present links return `linked: false`. Documented as the lightweight one-shot variant; the dedicated tool is preferred for batch add/remove.
3. **Strict top-level key validation in `product_request_apply` and `product_request_validate`.** The request parser rejects any top-level key not in the closed set `{type, schema-version, reason, artifacts, changes}` with **E025 unknown-request-key**, listing every offending key with its JSONPath location in one validation pass. Applies in both `validate` and `apply` paths.
4. **Strict mutation-field validation against the target artifact's front-matter schema.** Each `mutation.field` is checked against the known front-matter fields for the target artifact's type (feature/adr/tc/dep) using the same schema that drives `product_schema`. Unknown fields are reported as **E026 unknown-mutation-field**, listing the offending field, the artifact type, and a suggested closest-match (Levenshtein distance ≤ 2) when available.
   - Dot-paths are validated against the **first segment**: `domains-acknowledged.security` is accepted (the head `domains-acknowledged` is a known field). The leaf is intentionally not checked because nested keys (e.g. domain names, dependency interface fields) are open vocabularies.
   - The pseudo-field `body` continues to be accepted as a special case (ADR-038 decision 9).
5. **Extend `product_schema` output with the field allowlist.** The schema MCP tool returns each artifact type's field set, which is the same canonical list the new mutation-field validator consults. This makes the contract introspectable by agents.
6. **Error code registration.** `E025` and `E026` registered in `src/error.rs` and the ADR-013 error-code table. Both are tier-2 graph errors; both block apply.
7. **AGENTS.md update.** The "Key MCP Tools" table (and the MCP write-tool documentation in the agent context) gains a row for `product_feature_depends_on`. The `Working Protocol` section gains a note that requests with unknown top-level or mutation-field keys are rejected, not silently accepted.
8. **Session tests in `tests/sessions/`.** Each scenario TC builds a temp repo via `product request apply`, drives the new tool / new validation through the compiled binary's stdio MCP transport, and asserts on the JSON envelope. Same pattern as FT-046 / FT-059.

### Out

- **Bidirectional `depends-on` materialisation.** The graph already inverts edges at read time for impact analysis. Adding a `depended-on-by` field would duplicate state.
- **Auto-creating missing target features when `add` lists an unknown ID.** This feature insists on **E002** for unknown targets — staying consistent with `product_feature_link --adr/--test` behaviour.
- **Schema-version bump.** The new validation is strictly additive — every previously-valid request remains valid. No request that was rejected before is now accepted; some requests that were silently accepted are now correctly rejected. Per ADR-038 decision 6 the bump is unnecessary because no in-the-wild request *intentionally* used unknown keys (Defect 2 was a bug, not a documented capability).
- **Levenshtein hints on top-level keys.** `E025` is enough; the top-level key set is small and stable (5 entries). Suggesting `chnages` → `changes` is not worth a fuzzy-match dependency at this layer.
- **Generic field setter (`product feature set FIELD VALUE`).** Already rejected by ADR-037; this feature does not relitigate that decision.
- **Changes to existing CLI behaviour beyond adding the new subcommand.** All other commands stay byte-identical.

---

## Tool surface

### `product_feature_depends_on` (new)

| Parameter | Type | Required | Notes |
|---|---|---|---|
| `id` | string | yes | Feature ID — `FT-NNN`. |
| `add` | array of string | no | Feature IDs to add to `depends-on`. Validated against graph. |
| `remove` | array of string | no | Feature IDs to remove. No-op if not present. |

**Success response:**

```json
{
  "id": "FT-009",
  "depends_on": ["FT-004", "FT-018", "FT-021"],
  "added": ["FT-021"],
  "removed": [],
  "changed": true
}
```

**Error cases:**

- **E002 broken-link** — any value in `add` does not exist in the graph.
- **E003 dependency-cycle** — adding the proposed edges would close a cycle in the feature DAG. The response carries the offending cycle path.
- **E001** — `id` malformed or missing.

### `product_feature_link` (extended)

A new optional `feature` parameter is accepted; semantics identical to `product_feature_depends_on` with a single-element `add`. Provided for backwards compatibility with the existing one-shot link tool and to keep parity with the `--adr` / `--test` arguments.

```json
{ "id": "FT-009", "feature": "FT-021" }
```

### `product_feature_depends_on` (CLI)

```bash
product feature depends-on FT-009 --add FT-021 --add FT-018
product feature depends-on FT-009 --remove FT-018
```

Mirrors `product feature domain` in flag style.

### `product_request_validate` and `product_request_apply` (tightened)

No new parameters. New rejections:

```json
{
  "valid": false,
  "findings": [
    {
      "code": "E025",
      "severity": "error",
      "description": "unknown top-level key 'patch' in request — expected one of: type, schema-version, reason, artifacts, changes",
      "location": "$.patch"
    },
    {
      "code": "E026",
      "severity": "error",
      "description": "unknown mutation field 'dependsOn' for feature — did you mean 'depends-on'?",
      "location": "$.changes[0].mutations[0].field"
    }
  ]
}
```

Both errors are reported in the existing single-pass validation (ADR-038 decision 3), alongside any other findings.

---

## Implementation notes

- **`src/feature/depends_on.rs`** (new). Slice module following ADR-043. Pure `plan_depends_on_edit(graph, id, add, remove) -> DependsOnPlan` performs cycle detection by building a hypothetical `KnowledgeGraph` and running `topological_sort()` (the same approach `link_dep` uses today). `apply_depends_on_edit(plan)` writes one feature file via `fileops::write_file_atomic`.
- **`src/commands/feature_write.rs`** — add a thin `feature_depends_on(id, add, remove) -> CmdResult` adapter wired into `dispatch()`. Reuse the existing `acquire_write_lock_typed` / `load_graph_typed` helpers.
- **`src/mcp/field_handlers.rs`** — add `handle_feature_depends_on(args, graph, repo_root)` calling the same plan/apply pair. Register in `src/mcp/registry.rs::dispatch_tool` and `src/mcp/tools.rs`.
- **`src/mcp/write_handlers.rs::handle_feature_link`** — extend to accept an optional `feature` argument and add it to `front.depends_on` (with cycle detection) before writing.
- **`src/request/parse.rs::parse_request_str`** — after constructing the `Request`, walk the source `Mapping` and emit `E025` for any key not in the closed set. Findings are returned as a `Vec<Finding>`, integrated into the existing `parse_request_str` error path.
- **`src/request/validate/changes.rs::validate_change`** — call a new `field_schema::known_fields_for(artifact_type)` helper that returns a `&'static [&'static str]` of recognised front-matter field names. The first segment of `mutation.field` (split on `.`) must either match one of those names or be `body`; otherwise emit **E026** with a Levenshtein-1 / -2 hint.
- **`src/field_schema.rs`** (new). Single source of truth for the recognised field names per artifact type. Same data the `product_schema` MCP tool already exposes — refactor that tool to read from this module so the validator and the schema tool can never diverge. Fitness test (TC-732) asserts they stay in lockstep.
- **`src/error.rs`** — register `E025 UnknownRequestKey { key, location }` and `E026 UnknownMutationField { field, artifact_type, suggestion }`. Both map to exit code 1 (validation failure, ADR-013).
- **Tool-surface drift.** TC-723 (FT-059) already enforces parity between AGENTS.md "Key MCP Tools" and the registry. After this feature lands, that table grows by one row.
- **Runner config.** Every TC in this feature gets `runner: cargo-test` and `runner-args: "tc_NNN_snake_case"` at the same time the test is written, per CLAUDE.md.

---

## Acceptance criteria

A spec-authoring agent connected over MCP can:

1. Call `product_feature_depends_on { "id": "FT-XXX", "add": ["FT-YYY"] }` and observe `FT-YYY` appended to `FT-XXX`'s `depends-on` front-matter, with `changed: true` in the response (TC-733).
2. Call `product_feature_depends_on { "id": "FT-XXX", "add": ["FT-XXX"] }` (self-loop) or any cycle-creating add, and receive **E003 dependency-cycle** with the offending path; no file is mutated (TC-734).
3. Call `product_feature_depends_on { "id": "FT-XXX", "add": ["FT-DOES-NOT-EXIST"] }` and receive **E002 broken-link**; no file is mutated (TC-735).
4. Call `product_feature_link { "id": "FT-XXX", "feature": "FT-YYY" }` and observe the same edge added through the existing tool (TC-736).
5. Run `product feature depends-on FT-XXX --add FT-YYY` from the CLI and observe identical behaviour to the MCP tool — same edge written, same cycle detection (TC-737).
6. Submit a request with a top-level key `patch:` (or any other unknown key) and receive **E025** in `product_request_validate` and `product_request_apply`; `mutations: 0` is **not** returned, the request is rejected (TC-738).
7. Submit a `change` request whose `mutations[].field` is `dependsOn` (camelCase typo) and receive **E026** with `did you mean 'depends-on'?`; the request is rejected (TC-739).
8. Submit a `change` request whose `mutations[].field` is `domains-acknowledged.security` and observe it is accepted (head matches a known field; leaf is open-vocabulary) — confirms the new validation does not over-reach (TC-740).
9. Run `product graph check` after the feature lands; existing graph remains clean (TC-741, exit-criteria).
10. The fitness test that scans `AGENTS.md` "Key MCP Tools" passes with the new `product_feature_depends_on` row (extends TC-723 from FT-059).
11. `cargo t`, `cargo clippy -- -D warnings -D clippy::unwrap_used`, and `cargo build` all pass.
12. Every TC in the feature has `runner: cargo-test` and `runner-args` matching the Rust test function name.

---

## Functional Specification

### Inputs

- **`product_feature_depends_on`** — required `id: string` (FT-NNN), optional `add: array<string>` (FT-NNN list), optional `remove: array<string>`. At least one of `add` / `remove` must be present.
- **`product feature depends-on`** (CLI) — same shape as MCP tool, expressed as `--add` / `--remove` flags (each repeatable).
- **`product_feature_link`** — existing `id`, `adr`, `test`, plus new optional `feature: string`.
- **Request validation** — no new inputs; new rejections on existing input shapes.

### Outputs

- **`product_feature_depends_on`** returns the post-mutation `depends_on` list, the diff (`added`, `removed`), and `changed: bool`.
- **Request validation** continues to return the existing `findings[]` envelope; new error codes appear as additional entries.
- **`product_schema`** continues to return the existing schema envelope; the field allowlist is now sourced from `field_schema::known_fields_for`.

### State

- Graph mutation is limited to the target feature's `depends-on` list. No cascade. The reverse edge (impact direction) is always derived at read time and is never persisted.
- The request parser is stateless; tightening adds no new state.

### Behaviour

- **`product_feature_depends_on`**: load graph → validate every `add` ID exists → build hypothetical graph with proposed edges → `topological_sort` cycle check → write feature file atomically → return diff.
- **Idempotency**: adding an already-present ID and removing an absent ID are no-ops; `changed` is `false` only when neither list mutated state.
- **Strict request key validation**: runs as the first pass in `parse_request_str`, so callers see all key-shape findings before any other validation. Unknown keys emit **E025** for each offender.
- **Strict mutation-field validation**: runs in `validate_change` after the existing `field` non-empty check. The first segment of `mutation.field` is matched against `field_schema::known_fields_for(artifact_type)`. The pseudo-field `body` is unconditionally accepted. Unknown fields emit **E026**.

### Invariants

- **No silent acceptance of unknown shapes.** After this feature lands, every previously-silently-accepted request shape is either still valid (because it used recognised keys / fields) or now rejected with E025 or E026. There is no third path. Enforced by TC-738 / TC-739.
- **`depends-on` mutations always pass cycle detection.** A successful mutation never leaves the feature DAG with a cycle. Enforced by TC-734 and by the slice's unit test (`feature::depends_on::tests::adding_self_is_rejected`).
- **`product_schema` and the request validator share one source of truth.** Adding a new front-matter field to the schema automatically updates the request validator's allowlist. Enforced by TC-732 (parity test).
- **Existing tool surface preserved.** `product_feature_link` with `adr` or `test` set continues to produce identical output to pre-FT-062. The new `feature` argument is purely additive.

### Error handling

- **E002 broken-link** — `add` references a feature that doesn't exist.
- **E003 dependency-cycle** — proposed edges would close a cycle. Response carries the offending path (same shape as the existing CLI cycle error).
- **E025 unknown-request-key** — top-level request key not in `{type, schema-version, reason, artifacts, changes}`. Reported once per offender.
- **E026 unknown-mutation-field** — `mutation.field` head segment not in the artifact-type schema. Includes a closest-match suggestion when one exists at Levenshtein distance ≤ 2.
- All errors flow through `ProductError` and surface as JSON-RPC error objects via the existing MCP envelope. The MCP server does not panic and does not exit.

### Boundaries

- **In**: read access to the knowledge graph; write access to one feature file per `depends-on` mutation; read access to the request YAML for parser tightening.
- **Out**: writing to baseline files, writing to `requests.jsonl` beyond what the existing apply pipeline already does, network egress, anything that needs more than the existing advisory lock.
- **Caller responsibilities**: agents must inspect `findings[]` for E-class entries; callers that previously relied on "valid: true / mutations: 0" as a signal of success must migrate to checking the `mutations` array length **and** the absence of E-class findings.

### Compatibility

- **Schema version unchanged.** No `schema-version` bump (decision: ADR-038 decision 6's "version mismatch is a clear error" applies to deliberate format breaks; this feature only fixes parser bugs that masked invalid input as valid).
- **Existing valid requests remain valid.** Every request that was correctly accepted continues to be correctly accepted.
- **Existing silently-accepted-and-no-op'd requests now fail loudly.** This is the intended fix; the user-visible behaviour change is that previously hidden bugs surface as E025 / E026.
- **Existing CLI / MCP tools unchanged in shape.** `product_feature_link` gains an optional argument; all existing arguments behave identically.

## Out of scope

- **Bidirectional `depended-on-by` materialisation** — derived at read time, not persisted.
- **Auto-create on missing target** — `add` of unknown IDs is **E002**, not auto-creation.
- **Schema-version bump** — additive validation, no version change.
- **Generic `frontmatter set` setter** — already rejected by ADR-037.
- **Levenshtein hints on top-level keys** — closed set of 5, hint is unnecessary.
- **Changes to existing CLI behaviour beyond adding the new subcommand** — none.
- **New ADRs** — this feature is implementation work that follows ADR-037 (granular tools) and ADR-038 (request semantics) without introducing new architectural decisions. The closest-match suggestion is implementation detail of E026, not a new contract.
