---
id: ADR-038
title: Product Request — Unified Atomic Write Interface
status: proposed
features:
- FT-041
supersedes: []
superseded-by: []
domains:
- api
- data-model
- error-handling
scope: domain
source-files:
- docs/product-request-spec.md
- src/fileops.rs
- src/request.rs
---

**Status:** Proposed

**Context:** Product's write surface has grown organically through successive features: `feature new` / `adr new` / `test new` (FT-004), `feature link` / `status` (FT-004), `body_update` (FT-004), and the seven granular field-mutation tools in ADR-037 (`feature domain`, `feature acknowledge`, `adr domain`, `adr scope`, `adr supersede`, `adr source-files`, `test runner`). Each tool is idempotent, typed, validated in isolation, and atomic on its own file(s). The surface is correct in the small.

It is not correct in the large. An authoring session that creates one feature, two ADRs, one TC and one DEP with cross-links between them requires 12+ tool calls. Three failure modes emerge at that scale:

1. **Partial graph states from mid-sequence failure.** If the MCP connection drops between call 7 and call 8, the repository is left with half-created artifacts: a feature pointing to an ADR that lists it as a backlink but has no `scope` or `domains` set, a DEP with no governing ADR, a TC with no runner. `graph check` catches the inconsistency after the fact, but the broken state is already on disk.
2. **No cross-artifact validation.** Rules that span multiple artifacts — "every DEP has a governing ADR" (E013), "a feature's declared domains have either governing ADRs or written acknowledgements" (W010), "a new ADR's `supersedes` creates no cycle" (E004) — can only be evaluated once the full intent is known. Under the granular-tool model, the agent must interleave creates and links carefully enough to pass validation at every intermediate state. This inverts the right ergonomics: the caller should declare intent, Product should figure out the order.
3. **No inspectable intent.** An authoring session leaves no trace beyond commit diffs. There is no file to re-run, diff against the graph, validate offline, or hand to another agent to continue. This makes the authoring flow non-reproducible and non-auditable.

**ADR-037's batch-rejection** is relevant. ADR-037 explicitly rejected a batch mutation tool:

> Batch mutation tool (`product frontmatter patch ARTIFACT '{...}'`) — JSON patch over front-matter. More flexible than granular tools but harder to validate, harder to document, and produces opaque tool calls that agents and humans both find harder to read. Rejected for MCP use.

That rejection was correct for its scope: a generic JSON-patch-over-front-matter primitive is opaque and shifts validation responsibility to the caller. But ADR-037 was reasoning about **single-artifact field edits**. Requests address a different problem — **multi-artifact composition with cross-references**, which granular tools cannot express at all. The rejection of "flexible single-artifact patches" is not incompatible with "structured multi-artifact atomic transactions". This ADR supersedes ADR-037's rejection **only for the multi-artifact case**; single-field granular tools remain the right interface for trivial one-field edits.

---

**Decision:** Introduce a unified request interface that treats authoring intent as a first-class YAML document. A request is a typed, versioned, validated, atomically-applied description of changes to the graph. It is the **only composable write interface** — the existing granular tools remain available and are additive, not deprecated.

The full request schema, validation table, and apply pipeline are specified in [`docs/product-request-spec.md`](../product-request-spec.md). This ADR commits to the decisions that shape the spec.

---

### Decisions pinned by this ADR

**1. Three operation types, one schema.** A request declares `type: create`, `type: change`, or `type: create-and-change`. All three share the same validation pipeline, the same advisory lock, the same atomic-write primitives (ADR-015), and the same terminal/MCP output shape. There is no fourth type; deletion of artifacts is out of scope for the request interface and remains a manual operation.

**2. Product assigns IDs, never the author.** On a `create`, the author declares artifacts with optional `ref: local-name` tags. On apply, Product topologically sorts the request's artifact graph, assigns real IDs in dependency order, and rewrites every `ref:` occurrence (including in `changes:` mutation values) to the assigned ID. The returned `created` array maps `ref` → `id` so callers can resume with real IDs. Forward references that don't resolve within the request are E002.

**3. Cross-artifact validation is a single pass, and reports every finding.** Validation runs over the full request before any file is written. The output is the complete list of findings, not just the first. Partial-validation-with-early-exit is explicitly rejected because it forces the agent into a guess-fix-retry loop. This is the converse of the granular-tool model where each call validates its own narrow contract.

**4. Mutations are a closed set of four operations.** `set`, `append`, `remove`, `delete`. Dot-notation addresses nested fields (`domains-acknowledged.security`, `interface.port`). No `patch`, no `merge`, no JSON-patch RFC 6902 operations. The closed set is what makes requests reviewable — a human or an agent can read a change block and predict the result without consulting an external spec for each op.

**5. The `reason:` field is mandatory on every request and is recorded for audit.** Every request must declare a non-empty `reason:` string at the top level. On apply, the reason is:
  - Printed in terminal output and MCP `applied` response
  - Appended to `.product/request-log.jsonl` — one line per apply, with request hash, timestamp, reason, and the `created` / `changed` arrays
  - Used as the default git commit message suffix if apply is invoked with `--commit`
  This gives every graph mutation a human-readable justification that survives beyond the git log. Missing or whitespace-only `reason:` is **E011** (same code as the acknowledgement reasoning rule, which enforces the same principle).

**6. Request YAML has a `schema-version:` field.** Optional, defaults to `1`. The request parser checks the version on load. Version mismatch (e.g. a request written for schema 2 applied by a v1 Product binary) produces a clear error with upgrade instructions — it is not silently ignored. Future schema changes (adding fields, changing op semantics) bump the version and come with a migration path. The initial version is `1` for the schema specified in `product-request-spec.md` at the time of implementation.

**7. W-class findings are advisory, E-class findings block apply.** `product request validate` always prints both. `product request apply` proceeds if only W-class findings are present (with the warnings printed); it fails with exit code 1 if any E-class finding is present. This matches the existing `graph check` semantics (ADR-009) and keeps the error-code contract uniform across Product.

**8. The post-apply `graph check` is a health monitor, not a gate.** Steps 1–7 of the apply pipeline (validate → lock → sort → resolve → write new → write mutated → unlock) are the transactional boundary. Step 8 runs `graph check` and reports findings, but the files are already written. This is deliberate: validation *before* apply must be strong enough to guarantee a clean graph; if `graph check` exits non-zero after a successful apply, that is a Product bug, not a user error. The invariant is: **apply returning success implies `graph check` would exit 0 or 2, never 1.** This invariant is enforced by test (see below).

**9. Body mutations on accepted ADRs are allowed by the request schema, but trigger E014 on the next `graph check`.** The request interface does not duplicate ADR-032's content-hash enforcement. A `change` request with `field: body` on an accepted ADR succeeds — the file is written — and the resulting content-hash mismatch is caught by the very next `graph check` invocation (step 8 of the same apply, in fact). The user resolves it with `product adr accept --amend --reason "..."`. Blocking body mutations at the request layer would duplicate logic that already lives in the immutability machinery; reporting them after the fact keeps the two concerns cleanly separated. The spec prose already reflects this behaviour; this ADR makes it a pinned decision.

**10. Failed apply = zero files changed. Enforced by pre-apply checksum + post-failure verification.** Before step 5, Product records SHA-256 hashes of every artifact file that the request could possibly touch (all files in `docs/features/`, `docs/adrs/`, `docs/tests/`, `docs/deps/`). If apply fails at any step after step 4, Product:
  1. Deletes any new files written in step 5 that exist on disk
  2. Restores any mutated file from its `.product-tmp.<pid>` sidecar (kept until step 7 releases the lock — step 6 writes tmp first, renames only after all targets are written successfully)
  3. Re-checksums all files that could have been touched and asserts they match the pre-apply hashes
  4. Releases the lock
  5. Surfaces the original error

The "write all tmp files, then rename all" pattern replaces the simpler "tmp + rename per file" pattern used by single-file atomic writes. This is the one place the request interface cannot reuse the existing atomic-write primitive verbatim — it needs a batch variant. Single-file writes continue to use the existing primitive unchanged.

**11. The error `location:` field is JSONPath against the request document.** `"$.artifacts[4]"` points to the fifth element of the `artifacts` array. `"$.changes[0].mutations[1].value"` points to a specific mutation value. This replaces the ambiguous `"artifacts[4]"` shorthand in the early spec draft. JSONPath is a published standard (RFC 9535), supported by common validators and editors, and is unambiguous.

**12. Drafts live in `.product/requests/` and are gitignored by default.** `product request apply FILE` works on any YAML file at any path — the drafts directory is a convention, not a store. `product request draft` lists the drafts directory; `product request create` writes a new template file there with a timestamp-prefixed name. No draft registry, no draft IDs, no draft metadata beyond what lives in the YAML itself.

**13. `ref:` names are case-sensitive and match `^[a-z][a-z0-9-]*$`.** Lowercase ASCII letters, digits, and hyphens. Starts with a letter. This avoids case-sensitivity bugs across macOS/Linux filesystems and matches the style of the spec's own examples (`ref:ft-rate-limiting`). Invalid ref names are E001.

**14. Request surface coexists with granular tools; no deprecation.** The seven granular tools from ADR-037, plus `feature new` / `adr new` / `test new` / `feature link` / `status` / `body_update`, remain supported CLI commands and MCP tools. Internally they are implemented on top of the same validation + atomic-write primitives the request interface uses — the request interface is a composition layer, not a replacement. An agent that prefers granular tools continues to work; an agent that prefers requests benefits from atomicity and cross-artifact validation. Deprecation of any granular tool, if pursued later, requires its own ADR.

---

### Test coverage

This ADR is validated by the TCs under FT-041. Core scenarios each pinned decision requires:

| Decision | Covered by TC (title) |
|---|---|
| Three types, one schema | `request type create round-trips`, `request type change round-trips`, `request type create-and-change round-trips` |
| IDs assigned by Product, `ref:` resolution in dependency order | `request forward refs resolve in topological order` |
| All findings reported at once | `request validate reports every finding in one pass` |
| Four-op closed set with dot-notation | `request mutation ops cover set append remove delete with dot-notation` |
| `reason:` mandatory and logged | `request rejects empty reason`, `request writes reason to request-log.jsonl` |
| `schema-version:` handling | `request rejects unknown schema version with upgrade hint` |
| W-advisory / E-blocking contract | `request apply proceeds on W-class, blocks on E-class` |
| Post-apply `graph check` invariant | `successful apply never produces graph-check exit 1` |
| Body-mutation-on-accepted-ADR path | `body mutation on accepted ADR succeeds and reports E014 on next graph check` |
| Zero-files-changed on failure | `failed apply leaves every file unchanged (pre/post checksum verified)` (invariant) |
| JSONPath error locations | `request validate findings include JSONPath location` |
| Drafts directory convention | `request draft lists .product/requests/ entries` |
| Case-sensitive ref name grammar | `request rejects invalid ref name format` |
| Coexistence with granular tools | `granular tools continue to work alongside request interface` |

A chaos TC also covers process-kill between steps 5 and 6 to verify step-10's restoration behaviour end-to-end.

---

**Rationale:**

- **Composition-first interface for a composition-heavy problem.** Authoring sessions are almost always multi-artifact. Making the multi-artifact case require N tool calls with interleaved validation is hostile to the primary workflow. One request = one atomic unit of intent = one validation pass = one write = one audit record.
- **Reversing ADR-037 only for multi-artifact composition.** ADR-037's reasoning about opaque single-field patches is still correct. This ADR is scoped to the different problem of multi-artifact transactions. Both can coexist because they address different shapes of authoring work.
- **Intent as data.** A request YAML is inspectable before apply, diffable against reality, saveable for later, and shareable across agents. Granular tool calls leave no equivalent artefact. This matches the PRD's broader position that the knowledge graph itself is data, not state.
- **`reason:` as a mandatory audit string** makes every mutation justifiable in retrospect. The request log (`.product/request-log.jsonl`) becomes the single place to answer "why did this artifact change?" without archaeology through git blame on YAML diffs.
- **Schema versioning from day one** avoids the common mistake of adding version handling only after the first breaking change. v1 is defined now; v2 and its migration are future work.
- **JSONPath over ad-hoc location strings** gives MCP clients a standard, parseable, editor-friendly pointer format for mapping errors back to the source YAML. This matters because the MCP consumer is typically an LLM producing the YAML — precise error locations drive reliable self-correction.
- **Step-10's batch-write-then-batch-rename** preserves the zero-files-changed invariant that single-file atomic writes achieve naturally. The slight complexity cost is paid once in `fileops` and amortised across all request applies.
- **Post-apply `graph check` as a health monitor** separates "did the transaction succeed" from "is the graph healthy". Conflating the two would either block apply on W-class findings (noisy and wrong) or hide E-class findings (dangerous). Reporting post-apply with the apply-succeeded invariant enforced by test is the clean split.

**Rejected alternatives:**

- **Extend granular tools with transactions (`product transaction begin ... commit`).** Keeps the existing tool surface but wraps it in a BEGIN/COMMIT envelope. Rejected because it splits intent across many calls (so the YAML-as-data benefit is lost), requires a transaction state machine on the server side (state the filesystem doesn't otherwise need), and makes validation-of-the-whole-intent impossible — each tool call inside the transaction still validates in isolation. The whole point of the request is that the full intent is known at validation time.
- **JSON-patch-style generic requests (`[{"op":"add","path":"/features/-","value":{...}}, ...]`).** Maximum flexibility but aligns with the patch rejection in ADR-037: opaque, validation-hostile, not reviewable. Rejected for the same reasons.
- **GraphQL-like mutation document.** A mutation DSL with typed fields and structured responses. Technically excellent but imports a large external grammar and runtime cost for a problem that YAML + typed ops solves directly. Rejected for over-engineering.
- **Request stored server-side with a numeric ID.** Allows `product request status REQ-007`, long-lived requests, agent collaboration on a single request over time. Rejected because the filesystem already solves this: a request YAML file in `.product/requests/` is the request's identity. Adding a server-side request store duplicates git-tracked state and introduces a registry that needs its own consistency story.
- **No `reason:` requirement — make it optional.** Rejected because the audit record is one of the request's primary benefits. An optional reason becomes a usually-missing reason. E011 on empty reason is cheap and the rule matches the domain-acknowledgement precedent (ADR-037).
- **Block body-mutations-on-accepted-ADR at the request layer.** Rejected because it duplicates ADR-032 immutability logic, risks drift between the two enforcement points, and removes the ability to amend via the request interface (the amendment workflow remains via `product adr accept --amend` as before).
- **Early-exit validation on first finding.** Rejected because it forces a guess-fix-retry loop and defeats the purpose of having a structured request. Reporting every finding at once is the primary usability win over the granular-tool model.
- **Deprecate the granular tools immediately.** Rejected because no one has asked for that, it breaks existing agent integrations, and the granular tools are cheaper to call than a full request for trivial one-field edits. Coexistence is free.
