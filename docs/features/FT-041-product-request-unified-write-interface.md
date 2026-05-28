---
id: FT-041
title: Product Request — Unified Write Interface
phase: 5
status: complete
depends-on:
- FT-004
- FT-018
- FT-021
- FT-034
- FT-038
adrs:
- ADR-002
- ADR-013
- ADR-015
- ADR-020
- ADR-032
- ADR-037
- ADR-038
tests:
- TC-486
- TC-487
- TC-488
- TC-489
- TC-490
- TC-491
- TC-492
- TC-493
- TC-494
- TC-495
- TC-496
- TC-497
- TC-498
- TC-499
- TC-500
- TC-501
- TC-502
- TC-503
- TC-504
- TC-665
- TC-666
- TC-667
- TC-668
- TC-669
- TC-670
domains:
- api
- data-model
- error-handling
domains-acknowledged:
  ADR-040: predates ADR-040; feature ships independently of the unified verify / LLM-boundary pipeline
  ADR-018: Predates the 2026-04-22 scope promotion of ADR-018 to cross-cutting. Test coverage reflects the property/session/benchmark strategy as it existed when this feature shipped; not retroactively reclassified.
  ADR-042: Pre-dates ADR-042; this feature does not define TC types or validate the type vocabulary. FT-048 owns the mechanics.
  ADR-047: Predates ADR-047; this feature does not author the functional-spec body convention. FT-055 owns the structural validator and W030 mechanics.
  ADR-041: Pre-dates ADR-041; this feature does not author absence TCs or set removes/deprecates on ADRs. FT-047 owns the mechanics.
  ADR-048: Predates ADR-048; this feature does not author the canonical .product/ folder layout. FT-057 owns the migration command and discovery fallback.
  ADR-043: Predates ADR-043; feature command adapters were written before the slice+adapter pattern was formalised and are not retroactively refactored.
---

The Product request is the single composable write interface to the knowledge graph. Multi-artifact mutations — creating a feature with its governing ADRs, TCs and DEPs in one atomic step; linking and re-linking across several artifacts; changing a batch of fields with one audit record — flow through one of two MCP tools (`product_request_validate`, `product_request_apply`) and one CLI command (`product request`).

The request interface is additive. The existing granular write tools (FT-004, FT-038) remain available for trivial single-field edits. Requests are the right interface whenever intent spans more than one artifact or more than one field.

The full schema and apply pipeline are defined in [`docs/product-request-spec.md`](/docs/product-request-spec.md); the pinned decisions that shape the spec live in [ADR-038](/docs/adrs/ADR-038-product-request-unified-atomic-write-interface.md).

---

## Depends on

- **FT-004** — artifact authoring; the underlying atomic-write primitives
- **FT-018** — validation and graph health; the E-class codes and `graph check`
- **FT-021** — MCP server; new write tools on the dual transport
- **FT-034** — content hash immutability; E014 interaction on body mutations of accepted ADRs
- **FT-038** — front-matter field management; the granular tools that coexist with the request interface

(These are feature dependencies — `depends-on` in front-matter. The current tool surface has no MCP setter for feature `depends-on`, so they are documented here until FT-041 itself delivers the request interface capable of setting that field.)

---

## Why a unified request

The current write surface is a growing catalogue of granular commands — one per field family (FT-004 scaffolding, FT-038 field management, the `link` / `status` / `acknowledge` / `scope` / `supersede` / `source-files` / `runner` commands). Each is idempotent and validated in isolation, but composing a realistic authoring session across them has three problems:

1. **Partial graph states** — creating a feature, three ADRs, a TC and a DEP requires 12+ tool calls. If the agent crashes or the MCP connection drops mid-sequence, the graph is left with orphan artifacts, missing links, or dangling forward references. `graph check` catches the inconsistency after the fact, but the broken state is already committed to disk.
2. **No cross-artifact validation** — rules like "every DEP has a governing ADR" (E013) or "a feature's declared domains have either governing ADRs or written acknowledgements" (W010) can only be evaluated once the full intent is known. Under granular tools, the agent must interleave creates and links carefully enough to pass validation at every intermediate state. The request interface inverts this: declare intent, Product figures out the order.
3. **No inspectable intent** — an authoring session leaves no trace beyond commit diffs. There is no file to re-run, diff against the graph, validate offline, or hand to another agent to continue. Request YAMLs are intent as data: saveable, re-validatable, diffable, auditable.

ADR-038 pins the decision and explains why ADR-037's earlier rejection of "batch mutation tools" does not apply: that rejection addressed opaque single-artifact field patches; requests are structured multi-artifact transactions and are a different shape of interface.

---

## The three operations

| Type | Use |
|---|---|
| `create` | New artifacts that do not exist yet |
| `change` | Mutations to existing artifacts |
| `create-and-change` | Both in one atomic operation |

All three share the same validation pipeline, advisory lock, atomic-write primitives (ADR-015), and output shape.

### `type: create`

New artifacts. **IDs are assigned by Product on apply — never declared by the author.** Forward references (`ref:local-name`) let artifacts in the same request cross-link each other before IDs exist. On apply, Product topologically sorts the request's artifact graph, assigns real IDs in dependency order, and rewrites every `ref:` occurrence to the assigned ID. Cross-links are bidirectional — declaring `adrs: [ref:adr-x]` on a feature also sets `features: [FT-009]` on the ADR automatically. Ref names must match `^[a-z][a-z0-9-]*$` (ADR-038 decision 13).

See the spec §`type: create` for the field tables per artifact type and the forward-reference rules.

### `type: change`

Mutations to existing artifacts. Each change targets one artifact by its real ID and declares one or more mutations from a closed set of four operations (ADR-038 decision 4):

| Op | Applies to | Behaviour |
|---|---|---|
| `set` | any scalar, string, nested field | Replace field value entirely |
| `append` | array fields | Add value — deduplicates, no error if already present |
| `remove` | array fields | Remove value — no error if not present |
| `delete` | optional fields | Remove the field from front-matter entirely |

Dot-notation addresses nested fields (`domains-acknowledged.security`, `interface.port`). The `field: body` case mutates prose below the front-matter. On accepted ADRs this **succeeds at apply time and triggers content-hash mismatch (E014) on the post-apply `graph check`**, resolved via `product adr accept --amend --reason "..."` (ADR-032, ADR-038 decision 9). The request layer does not duplicate immutability enforcement.

### `type: create-and-change`

Both sections (`artifacts` and `changes`) present in one request. Forward references from new artifacts can appear in change mutation values (e.g. append a newly-created TC to an existing feature's `tests` list). Resolved in the same topological pass.

---

## Required fields

Every request, regardless of type, must declare:

- `type:` — one of `create`, `change`, `create-and-change`
- `schema-version:` — integer, defaults to `1` if omitted (ADR-038 decision 6). Version mismatch is a clear error with upgrade instructions, not silent.
- `reason:` — non-empty human-readable string. Missing or whitespace-only is **E011** (same code as domain-acknowledgement reasoning, ADR-038 decision 5). The reason is printed in apply output, appended to `.product/request-log.jsonl`, and used as the default git commit message suffix if apply is invoked with `--commit`.

---

## Validation

Validation runs across the full request before any file is written. Every finding is reported at once — not the first one (ADR-038 decision 3). E-class findings block apply; W-class findings are advisory and print but do not block (ADR-038 decision 7).

**Within the request:**

| Rule | Error |
|---|---|
| `ref:` value not defined in request | E002 |
| `ref:` name does not match `^[a-z][a-z0-9-]*$` | E001 |
| DEP with no governing ADR in request or existing graph | E013 |
| Domain value not in `[domains]` vocabulary | E012 |
| `scope` not one of `cross-cutting / domain / feature-specific` | E006 |
| `tc-type` / `dep-type` not a valid value | E006 |
| Missing or whitespace-only `reason:` | E011 |
| Unknown `schema-version:` | E001 (with upgrade hint) |

**Against the existing graph:**

| Rule | Error |
|---|---|
| `target:` ID does not exist | E002 |
| `value:` ID (non-ref) does not exist and is not being created | E002 |
| `depends-on` creates a cycle | E003 |
| `supersedes` creates a cycle | E004 |

**Advisory (non-blocking):** new-ADR conflicts (G005), `breaking-change-risk: high` on new DEPs.

Findings include a `location:` field in **JSONPath syntax** against the request document (ADR-038 decision 11): e.g. `"$.artifacts[4]"`, `"$.changes[0].mutations[1].value"`.

---

## Apply pipeline

1. Record SHA-256 hashes of every file the request could touch (all files in `docs/features/`, `docs/adrs/`, `docs/tests/`, `docs/deps/`)
2. Validate full request — exit 1 on any E-class finding, nothing written
3. Acquire advisory lock (ADR-015)
4. Topologically sort the request's artifact graph, assign IDs in order
5. Resolve all `ref:` values
6. Write all new artifact files to `.product-tmp.<pid>` sidecars
7. Write all mutated artifact files to `.product-tmp.<pid>` sidecars
8. If any of step 6 or 7 failed: delete sidecars, re-checksum to assert zero-files-changed, release lock, surface error
9. Rename all sidecars to their target paths (the commit point)
10. Release lock
11. Run `product graph check` as a health monitor (step 10 already committed the transaction)
12. Append `reason:` + `created` + `changed` to `.product/request-log.jsonl`
13. Print summary with all assigned IDs

The step 6/7/9 batch-write-then-batch-rename pattern (ADR-038 decision 10) replaces the simpler per-file atomic write for single-artifact mutations. It is the one place the request interface extends `fileops` rather than reusing it verbatim.

### Invariants

- **Failed apply leaves zero files changed.** Verified by pre/post checksum of all touched files (ADR-038 decision 10).
- **Successful apply never produces `graph check` exit 1.** Exit 1 after apply is a Product bug (ADR-038 decision 8). Enforced by test.
- **`product request validate` never writes to disk** under any circumstance.
- **`append` / `remove` are idempotent** — applying the same request twice produces the same end state.

---

## Command and MCP surface

**CLI:**

```
product request create        # open $EDITOR with a create template in .product/requests/
product request change        # open $EDITOR with a change template
product request validate FILE # validate without writing — reports every finding
product request apply FILE    # validate then write atomically
product request apply FILE --commit  # atomic apply followed by a git commit with reason as message
product request diff FILE     # show what would change, write nothing
product request draft         # list saved drafts in .product/requests/
```

Drafts live in `.product/requests/` by convention (gitignored by default). The directory is a convention, not a store — `product request apply` works on any YAML file at any path (ADR-038 decision 12).

**MCP:**

| Tool | Purpose | Writes |
|---|---|---|
| `product_request_validate` | Parse + validate request YAML, return findings array. Never writes. | No |
| `product_request_apply` | Full apply pipeline. Returns assigned IDs in `created` / `changed` arrays. | Yes |

Agent workflow:
1. Produce request YAML (from the authoring prompts, FT-022)
2. Call `product_request_validate` — fix any E-class findings
3. Call `product_request_apply` — receive assigned IDs
4. Continue with the real IDs (e.g. `product_context FT-009`)

---

## Coexistence with existing granular tools

The request interface is additive (ADR-038 decision 14). No granular tool is deprecated by this feature:

- `product feature new` / `adr new` / `test new` (FT-004)
- `product feature link` / `feature status` / `adr status` / `test status` (FT-004)
- `product body_update` (FT-004)
- `product feature domain` / `feature acknowledge` / `adr domain` / `adr scope` / `adr supersede` / `adr source-files` / `test runner` (FT-038)

All remain supported in CLI and MCP. Internally they share the validation + atomic-write primitives the request interface uses. An agent that prefers granular tools continues to work; an agent that needs atomicity, cross-artifact validation, or auditability picks the request interface. Deprecation of any granular tool requires its own ADR.

---

## Out of scope

- Changing the content-hash immutability model for accepted ADRs (ADR-032): body mutations on accepted ADRs apply successfully at the request layer and surface as E014 via the post-apply `graph check`; resolution remains `product adr accept --amend --reason`.
- Removing any granular write command: the request interface is additive; FT-004 and FT-038 tooling coexists unchanged.
- Server-side request storage, request IDs, or a request registry: requests are YAML files on disk identified by path alone.
- Deletion of artifacts via requests: deletion remains a manual operation outside the request model.
- Partial rollback on post-apply `graph check` findings: step 11 is a health monitor; if it surfaces findings, the user resolves via follow-up requests.
- Migration of existing authoring prompts (FT-022) and orchestration (FT-023) to use requests: those updates are separate feature work once FT-041 ships.

---

## Acceptance criteria summary

A user can:

1. Write a `type: create` request YAML describing a feature, two ADRs, a TC and a DEP with cross-references by `ref:` name, run `product request validate FILE`, see zero findings, run `product request apply FILE` and observe all five files written with resolved IDs and bidirectional cross-links.
2. Write a `type: change` request YAML targeting one or more existing artifacts with `set` / `append` / `remove` / `delete` mutations — including dot-notation on nested fields — and apply it atomically.
3. Write a `type: create-and-change` request that creates a TC and appends it to an existing feature's `tests` list in one atomic operation.
4. Observe that a request with any E-class finding writes nothing to disk (invariant verified by pre/post checksumming of all touched files).
5. Observe that the same request applied twice produces the same end state (idempotent `append` / `remove` semantics).
6. Observe that a request missing `reason:` or with whitespace-only `reason:` is rejected with E011 before any file is touched.
7. Observe that a request with `schema-version: 99` is rejected with a clear upgrade hint.
8. Observe that findings carry JSONPath `location:` values that map unambiguously to positions in the request YAML.
9. Observe that a body mutation on an accepted ADR succeeds at apply time and that the same apply's post-apply `graph check` surfaces E014.
10. Observe that `.product/request-log.jsonl` gains one line per successful apply containing the reason, timestamp, request content hash, and the `created` / `changed` arrays.
11. Call `product_request_validate` and `product_request_apply` via MCP and receive the documented JSON shape including the `created` and `changed` arrays with assigned IDs.
12. Run existing granular write tools (e.g. `product feature domain`) and observe they continue to work unchanged alongside the request interface.

---

## Description

The Product request is the single composable write interface to the knowledge graph. It replaces ad-hoc sequences of granular CLI commands with a structured, validatable YAML document that expresses multi-artifact intent atomically. Three operation types (`create`, `change`, `create-and-change`) share one validation pipeline, one advisory lock, one atomic-write commit point, and one audit log entry. The interface is additive: every granular write tool (FT-004, FT-038) remains available for single-field edits.

## Functional Specification

### Inputs

- A request YAML file (at any path; the `.product/requests/` directory is a convention, not a requirement) containing:
  - `type:` — one of `create`, `change`, `create-and-change`
  - `schema-version:` — integer (defaults to `1` if omitted)
  - `reason:` — non-empty human-readable string; whitespace-only is rejected with E011
  - `artifacts:` (for `create` and `create-and-change`) — array of artifact declarations with optional `ref:` names for forward references
  - `changes:` (for `change` and `create-and-change`) — array of mutation objects with `target:` ID and one or more `op` / `field` / `value` triples (`set`, `append`, `remove`, `delete`)
- For CLI: the path is a positional argument to `product request validate FILE` or `product request apply FILE`
- For MCP: `product_request_validate` or `product_request_apply` receive the YAML text as a parameter

### Outputs

- **`product request validate`** — exits 0 with a findings report (empty on success) or exits 1 with all E-class findings listed; never writes to disk
- **`product request apply`** (and `product_request_apply`) — on success: exits 0, prints (or returns as JSON) the list of assigned IDs in `created` and `changed` arrays, appends one entry to `.product/request-log.jsonl`, and runs `product graph check` as a post-apply monitor; on failure: exits 1 with all E-class findings, zero files changed
- **`product request diff`** — prints a textual diff of what would change; writes nothing
- **`product request log`** — tabular list of past requests; writes nothing
- **MCP `product_request_validate`** — returns a `findings` array; no writes
- **MCP `product_request_apply`** — returns `{ applied, created, changed, findings }`; writes atomically

### State

- **`.product/request-log.jsonl`** — append-only log of every successful apply; each line records `reason`, `timestamp`, `request content hash`, `created`, and `changed` arrays
- **`docs/features/`, `docs/adrs/`, `docs/tests/`, `docs/deps/`** — artifact files created or mutated by `apply`; the advisory lock (ADR-015) serialises concurrent writes
- No other persistent state. Pre-apply checksums of all touchable files are held in memory during apply and discarded after the rename commit point

### Behaviour

1. Parse and structurally validate the request YAML (schema version, required fields, ref-name format)
2. Validate full intent before any write: resolve `ref:` forward references, check cross-artifact rules (E002 unknown target, E003 dependency cycle, E004 supersedes cycle, E012 domain vocabulary, E013 DEP without governing ADR), report all findings at once (never stop at first)
3. Acquire advisory lock (ADR-015)
4. Topologically sort the artifact dependency graph; assign real IDs in order; rewrite every `ref:` occurrence to the assigned ID
5. Write all new and mutated artifact files to `.product-tmp.<pid>` sidecars
6. If any sidecar write fails: delete all sidecars, verify checksums show zero-files-changed, release lock, surface error
7. Batch-rename all sidecars to target paths (the atomic commit point)
8. Release lock; run `product graph check` as a health monitor; append log entry; print summary

**Forward references:** `ref:local-name` values bind an artifact declared in the same request; `ref:` names must match `^[a-z][a-z0-9-]*$`. Cross-links are bidirectional on apply.

**Body mutations on accepted ADRs:** succeed at the request layer and surface as E014 on the post-apply `graph check`; resolution is `product adr accept --amend --reason "..."`.

**`append` / `remove` idempotency:** applying the same request twice produces the same end state; duplicates on `append` are silently deduplicated.

**`--commit` flag on `product request apply`:** after a successful apply, performs a git commit using `reason:` as the message suffix.

### Invariants

- A failed apply leaves zero files changed (enforced by pre/post SHA-256 checksums of all touched files)
- A successful apply never produces `product graph check` exit 1 (exit 1 after apply is a Product bug)
- `product request validate` never writes to disk under any circumstance
- `append` and `remove` mutations are idempotent across repeated applies
- All E-class findings are reported together; validation never stops at the first error (ADR-038 decision 3)

### Error handling

| Code | Condition |
|---|---|
| E001 | Unknown `schema-version:` (includes upgrade hint) or `ref:` name not matching `^[a-z][a-z0-9-]*$` |
| E002 | `ref:` not defined in request, or `target:` / `value:` ID does not exist |
| E003 | `depends-on` creates a cycle |
| E004 | `supersedes` creates a cycle |
| E006 | `scope`, `tc-type`, `dep-type` value not in valid set |
| E011 | `reason:` missing or whitespace-only |
| E012 | Domain value not in `[domains]` vocabulary |
| E013 | DEP with no governing ADR in request or existing graph |
| W-class | Advisory: new-ADR conflicts (G005), `breaking-change-risk: high` on new DEPs — reported but do not block apply |

Findings include a `location:` in JSONPath syntax (`$.artifacts[4]`, `$.changes[0].mutations[1].value`) so the author can locate the offending node in the request document.

### Boundaries

- Does not change the content-hash immutability model for accepted ADRs (ADR-032); body mutations on accepted ADRs are applied and surface as E014 post-apply
- Does not deprecate or remove any granular write command (FT-004, FT-038); the request interface is purely additive (ADR-038 decision 14)
- Does not support deletion of artifacts; deletion remains a manual operation
- Does not store requests server-side or assign request IDs; requests are YAML files on disk identified by path
- Does not support partial rollback after a successful apply; post-apply `graph check` findings are resolved via follow-up requests
