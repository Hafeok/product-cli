---
id: FT-064
title: Strict Change-Spec Validation and Artifact Deletion Surface
phase: 5
status: planned
depends-on:
- FT-041
- FT-062
adrs:
- ADR-018
- ADR-043
- ADR-047
tests:
- TC-770
- TC-771
- TC-772
- TC-773
- TC-774
- TC-775
domains: []
domains-acknowledged:
  ADR-040: No verify-pipeline change and no LLM calls introduced. Strict-shape rejections (E025-class) and the new deletion request type surface through the existing request validate/apply path; stage 2 graph check picks up post-apply state unchanged. ADR-040's six-stage pipeline and zero-LLM boundary are unaffected.
  ADR-049: No context-bundle rendering changes. FT-064 touches the request write surface only; bundle assembly, template resolution, and `--target` routing are untouched. Deleted artifacts naturally drop out of subsequent bundles because the graph rebuilds from disk on every invocation (ADR-003).
  ADR-042: All TCs in this feature use the existing `scenario` descriptive type (and `exit-criteria` for TC-775). No new structural types are introduced and no custom descriptive types are added to product.toml; ADR-042's reserved/open partition is unaffected.
  ADR-048: No file-layout changes. Strict-shape validation lives in the existing request parser/validator paths; the new deletion operation writes to the same configured artifact directories and the same `requests.jsonl` location that ADR-048 canonicalises under `.product/`. No hardcoded paths are introduced.
  ADR-041: ADR-041 governs negative assertions about code/dependency presence via absence TCs and ADR `removes`/`deprecates` fields. FT-064 deletes spec-layer artifact files via the request interface — an orthogonal mechanism that records audit entries in requests.jsonl and does not interact with the absence-TC machinery. No `removes` or `deprecates` declarations and no absence TCs are required.
---

## Description

Two related gaps in the request-based MCP write surface that, together,
make the agent experience "validates clean, applies nothing" — the worst
possible failure mode for an autonomous spec-authoring loop.

**Gap 1 — `change` and `mutation` blocks accept unknown keys silently.**
FT-062 closed the same loophole at the request top level (E025) and at
the mutation-field level (E026), but the **shape** of a `change` block
and a `mutation` block is still permissive. A `change` block that
mis-nests its mutation fields (`op:` / `field:` / `value:` declared at
the change level instead of inside a `mutations:` list) parses as
`target: FT-XXX, mutations: []`. The request validates clean and applies
with **`mutations: 0`** — every byte on disk is unchanged, every linked
front-matter list still contains the entry the user thought they
removed. The same shape bug exists inside a mutation: any key other
than `op` / `field` / `value` is silently dropped.

**Gap 2 — there is no way to delete an artifact file over MCP or via the
request interface.** FT-041 explicitly lists deletion as out of scope
("Deletion remains a manual operation."). In practice this means an
agent that wants to retire a TC, an obsolete feature stub, or a draft
ADR has to shell out to `rm` and update the graph by hand — which
breaks the audit trail (`requests.jsonl` records every mutation but
records nothing for a manual deletion) and defeats the FT-042
hash-chain guarantee that the on-disk graph state can be reconstructed
from the log.

---

## Depends on

- **FT-041** — Product Request — Unified Write Interface. Owns the
  request parser / validator / apply pipeline. The strict-shape work
  extends the same E025-style validation that FT-062 introduced; the
  deletion work adds a new `type: delete` (or `delete:` section, TBD)
  to the request grammar.
- **FT-062** — MCP Parity for Feature `depends-on` and Strict Request
  Shape Validation. Sets the precedent for strict closed-set key
  validation (E025) and field allowlists (E026). This feature applies
  the same pattern one nesting level deeper.

---

## Scope of this feature

### In

1. **Strict closed-set key validation on `change` blocks.** A change
   accepts exactly `{target, mutations}`. Any other key surfaces as
   **E025** (or a new code, TBD during design) with a JSONPath
   `location` pointing at the offending key, listed in one validation
   pass alongside any other findings.
2. **Strict closed-set key validation on `mutation` blocks.** A
   mutation accepts exactly `{op, field, value}`. Any other key
   surfaces with the same error code as the change-level check.
3. **Reject empty `mutations: []` on a change.** A change with no
   mutations is meaningless — the request is rejected with **E006**
   (invalid shape) with a clear "this change has no mutations — did
   you mean to nest `op:`/`field:`/`value:` inside a `mutations:`
   list?" hint.
4. **Artifact deletion via the request interface.** Either as a new
   `type: delete` request type or as a `deletions:` top-level section
   on `change` / `create-and-change` requests (decision deferred to
   design). The deletion records target IDs, validates that no other
   artifact links to them (or accepts an `--allow-orphan` style
   override), and atomically removes the file plus appends a `delete`
   entry to `requests.jsonl` so the log replay can reconstruct the
   state.
5. **CLI parity.** Whatever the request surface looks like, the CLI
   exposes the same operation (`product request delete ...` or
   equivalent) and the MCP tool surface registers it.
6. **Regression cover for the no-op symptom.** A request that
   attempts to `op: remove` an item from a list-valued front-matter
   field **must** actually remove the item from the rendered file and
   report `mutations: >= 1` in the apply summary. TC-coverage of this
   path stays in place after the strict-shape work lands so a future
   refactor can't silently re-introduce the no-op.

### Out

- **Bulk / glob deletion** (`product request delete FT-*`). Deletion
  targets are enumerated explicitly per request.
- **Soft-delete / tombstones.** Deletion physically removes the file;
  if the user wants to keep a marker, they `set status: abandoned`
  first.
- **Cascading deletion of linked artifacts.** Deletion fails when
  other artifacts link to the target unless the request opts in to
  cascade behaviour (mechanism TBD — likely a `cascade: true` flag on
  the deletion entry rather than a CLI flag, to keep the audit
  trail).
- **Schema-version bump.** Strict-shape changes are additive — any
  previously-valid request stays valid. Deletion is a new request
  operation; whether it warrants a schema bump is part of design.

---

## Functional Specification

### Inputs

- Strict-shape validation: existing `change` / `mutation` blocks in
  request YAML, no new inputs.
- Deletion: a new request shape carrying one or more artifact IDs
  (`FT-NNN` / `ADR-NNN` / `TC-NNN` / `DEP-NNN`) and a `reason:`.

### Outputs

- Strict-shape: existing `findings[]` envelope; new entries on
  previously silent shape errors.
- Deletion: existing apply envelope plus a new `deleted: [{id, file}]`
  array, mirroring `created` / `changed`.

### State

- Strict-shape: stateless — pure parser tightening.
- Deletion: removes one or more files from the artifact directories;
  appends a `delete` entry to `requests.jsonl` with the same hash-chain
  as every other request type (FT-042).

### Behaviour

- **Strict-shape** runs in the same single-pass validator FT-062 set
  up; new findings are appended to the existing `findings[]` and the
  request is rejected on any E-class finding.
- **Deletion** runs the standard apply pipeline: pre-checksum, lock,
  validate (every target ID must exist and must not be linked
  elsewhere unless cascade is opted in), batch atomic remove
  (`unlink` of each target), post-checksum invariant, append log
  entry, return summary.

### Invariants

- **No silent acceptance of mis-shaped requests.** After this feature
  lands, every previously-silently-accepted mis-shaped change /
  mutation is rejected with an E-class finding. Enforced by the new
  TCs.
- **Deletion is auditable.** Every deletion appears in
  `requests.jsonl` and survives `product request replay`. Manual
  `rm` is still possible but recommended against in AGENTS.md.

### Error handling

- **E025-class** (or new code TBD) — unknown key inside a `change`
  block; unknown key inside a `mutation` block.
- **E006** — empty `mutations: []` on a change.
- **E002** — deletion target does not exist.
- **E0XX** (TBD) — deletion target is linked elsewhere and no
  cascade was opted in.

### Boundaries

- **In**: request parser, validator, apply pipeline; one new
  request operation; one new MCP tool / CLI subcommand.
- **Out**: any change to the granular write tools, manual file
  deletion behaviour, or git's view of the repo. Deletion does not
  commit — `--commit` continues to work the same way for the new
  operation.

## Out of scope

Bulk deletion, soft-delete tombstones, cascading deletion without
explicit opt-in, schema-version bump, deletion from non-tracked
paths (e.g. `.product/requests/` drafts — those are managed by
`product request discard`).
