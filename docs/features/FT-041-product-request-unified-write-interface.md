---
id: FT-041
title: Product Request — Unified Write Interface
phase: 5
status: planned
depends-on: []
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
domains:
- api
- data-model
- error-handling
domains-acknowledged: {}
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

- Changing the content-hash immutability model for accepted ADRs (ADR-032). Body mutations on accepted ADRs apply successfully at the request layer and surface as E014 via the post-apply `graph check`; resolution remains `product adr accept --amend --reason`.
- Removing any granular write command.
- Server-side request storage, request IDs, or a request registry. Requests are YAML files on disk; the filesystem is their identity.
- Deletion of artifacts via requests. Deletion remains a manual operation.
- Partial rollback on post-apply `graph check` findings. Step 11 is a monitor; if it surfaces findings, the user resolves via follow-up requests. The invariant is that a successful apply produces only exit 0 or 2 from `graph check`.
- Migration of existing authoring prompts (FT-022) and orchestration (FT-023) to use requests. Those updates are separate feature work once FT-041 ships.

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
