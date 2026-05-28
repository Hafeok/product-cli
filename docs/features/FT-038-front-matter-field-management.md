---
id: FT-038
title: Front-Matter Field Management
phase: 5
status: complete
depends-on: []
adrs:
- ADR-002
- ADR-037
tests:
- TC-461
- TC-462
- TC-463
- TC-464
- TC-465
- TC-466
- TC-467
- TC-468
- TC-469
- TC-470
- TC-471
domains: []
domains-acknowledged:
  ADR-040: predates ADR-040; feature ships independently of the unified verify / LLM-boundary pipeline
  ADR-041: Pre-dates ADR-041; this feature does not author absence TCs or set removes/deprecates on ADRs. FT-047 owns the mechanics.
  ADR-018: Predates the 2026-04-22 scope promotion of ADR-018 to cross-cutting. Test coverage reflects the property/session/benchmark strategy as it existed when this feature shipped; not retroactively reclassified.
  ADR-042: Pre-dates ADR-042; this feature does not define TC types or validate the type vocabulary. FT-048 owns the mechanics.
  ADR-047: Predates ADR-047; this feature does not author the functional-spec body convention. FT-055 owns the structural validator and W030 mechanics.
  ADR-048: Predates ADR-048; this feature does not author the canonical .product/ folder layout. FT-057 owns the migration command and discovery fallback.
  ADR-043: Predates ADR-043; feature command adapters were written before the slice+adapter pattern was formalised and are not retroactively refactored.
---

Product exposes granular CLI commands and MCP tools for editing every front-matter field on features, ADRs, and test criteria. This closes the authoring gap where agents can scaffold artifacts via `product_feature_new` and `product_adr_new` but cannot set domains, supersession chains, scope, source files, runner config, or domain acknowledgements without manual YAML editing.

### Problem

The current write tool surface covers: create, link, set status, update body. The following fields have no write tool:

- **Feature:** `domains`, `domains-acknowledged`
- **ADR:** `domains`, `scope`, `supersedes`, `superseded-by`, `source-files`
- **TC:** `runner`, `runner-args`, `runner-timeout`, `requires`

During a phone-based authoring session (FT-022), the agent produces incomplete artifacts. Domain classification, supersession chains, runner config, and scope must be manually edited afterward. This breaks the self-service authoring flow that FT-022 and FT-021 are designed to enable.

### New Tools

**Domain management:**

```bash
# Features
product feature domain FT-009 --add networking --add security
product feature domain FT-009 --remove storage

# ADRs
product adr domain ADR-013 --add error-handling --add api
```

Domains are validated against the `[domains]` vocabulary in `product.toml`. Invalid domain names produce E012.

**Domain acknowledgement:**

```bash
product feature acknowledge FT-009 --domain security \
  --reason "No new trust boundaries introduced."

# Remove acknowledgement:
product feature acknowledge FT-009 --domain security --remove
```

Empty or whitespace-only `--reason` produces E011. Acknowledgements close domain gaps from `product preflight` without requiring an ADR link.

**ADR scope:**

```bash
product adr scope ADR-013 cross-cutting
product adr scope ADR-040 domain
product adr scope ADR-041 feature-specific
```

**ADR supersession (bidirectional):**

```bash
product adr supersede ADR-036 --supersedes ADR-035
```

This writes to both files atomically: adds `ADR-035` to the `supersedes` list of `ADR-036`, adds `ADR-036` to the `superseded-by` list of `ADR-035`, and sets `ADR-035` status to `superseded` if it was `accepted`. Cycle detection runs before writing.

```bash
product adr supersede ADR-036 --remove ADR-035   # reverse the link
```

**ADR source files:**

```bash
product adr source-files ADR-023 --add src/drift.rs --add src/drift/
product adr source-files ADR-023 --remove src/old_drift.rs
```

**TC runner configuration:**

```bash
product test runner TC-054 --runner cargo-test --args "tc_054_product_impact_adr_001"
product test runner TC-054 --timeout 60s
product test runner TC-054 --requires binary-compiled
```

### MCP Tool Surface

All new commands are exposed as MCP write tools:

| MCP Tool | Parameters |
|---|---|
| `product_feature_domain` | `id`, `add[]`, `remove[]` |
| `product_feature_acknowledge` | `id`, `domain`, `reason` (or `remove: true`) |
| `product_adr_domain` | `id`, `add[]`, `remove[]` |
| `product_adr_scope` | `id`, `scope` |
| `product_adr_supersede` | `id`, `supersedes` (or `remove`) |
| `product_adr_source_files` | `id`, `add[]`, `remove[]` |
| `product_test_runner` | `id`, `runner`, `args`, `timeout`, `requires[]` |

### Validation

All tools validate before writing:

- Domain names checked against `product.toml` `[domains]` vocabulary (E012)
- Scope values checked against enum (E001)
- Supersession targets must exist (E002) and not create cycles (E004)
- Runner values checked against supported set: `cargo-test`, `bash`, `pytest`, `custom` (E001)
- Prerequisites checked against `product.toml` `[verify.prerequisites]` (E001)
- Acknowledgement reasoning must be non-empty (E011)
- All add/remove operations are idempotent — safe to retry

### Authoring Flow Integration

After this feature, the author-feature prompt flow becomes:

1. `product_feature_new` — scaffold
2. `product_feature_link` — wire ADRs and TCs
3. `product_feature_domain` — classify by concern area
4. `product_feature_acknowledge` — close domain gaps with reasoning
5. `product_graph_check` — verify structural health
6. `product_gap_check` — verify spec completeness

The author-adr prompt flow:

1. `product_adr_new` — scaffold
2. `product_adr_domain` — classify by concern area
3. `product_adr_scope` — set cross-cutting/domain/feature-specific
4. `product_adr_supersede` — declare supersession (if applicable)
5. `product_adr_source_files` — declare governed files
6. `product_adr_status` — accept when ready

After implementation, TC runner config:

1. `product_test_runner` — set runner, args, timeout, requires
2. `product verify FT-XXX` — execute and update status

---

## Description

Granular CLI commands and MCP write tools for editing every previously unmanaged front-matter field on features, ADRs, and test criteria (ADR-037). Covers domain classification, domain acknowledgements, ADR scope, bidirectional ADR supersession, ADR source-file declarations, and TC runner configuration. All tools validate inputs against the graph vocabulary and write atomically. Closes the authoring gap identified in FT-022 where agents could scaffold artifacts but not fully classify or wire them.

## Functional Specification

### Inputs

- `product feature domain FT-XXX --add DOMAIN [--remove DOMAIN]` — domain names validated against `[domains]` vocabulary in `product.toml`.
- `product feature acknowledge FT-XXX --domain DOMAIN --reason "..."` — non-empty reason string; `--remove` flag removes an existing acknowledgement.
- `product adr domain ADR-XXX --add DOMAIN [--remove DOMAIN]` — domain names validated against `[domains]` vocabulary.
- `product adr scope ADR-XXX VALUE` — value must be one of `cross-cutting`, `domain`, `feature-specific`.
- `product adr supersede ADR-XXX --supersedes ADR-YYY` — target ADR must exist; cycle check runs before writing. `--remove ADR-YYY` reverses the link.
- `product adr source-files ADR-XXX --add PATH [--remove PATH]` — paths validated for existence (warning, not error, for future files).
- `product test runner TC-XXX --runner RUNNER --args ARGS [--timeout DURATION] [--requires PREREQ]` — runner must be one of `cargo-test`, `bash`, `pytest`, `custom`; prerequisites validated against `[verify.prerequisites]` in `product.toml`.
- MCP equivalents for all above: `product_feature_domain`, `product_feature_acknowledge`, `product_adr_domain`, `product_adr_scope`, `product_adr_supersede`, `product_adr_source_files`, `product_test_runner`.

### Outputs

- All commands write updated YAML front-matter to the target artifact file atomically via `fileops::write_file_atomic`.
- `product adr supersede` writes to two files in a single advisory lock acquisition (bidirectional update): `supersedes` on the new ADR and `superseded-by` on the superseded ADR. If the superseded ADR has `status: accepted`, its status is changed to `superseded`.
- Console output confirms the mutation: e.g. `FT-009 domains updated: [networking, security]`, `ADR-036 supersedes ADR-035 (bidirectional)`.
- MCP tools return the same confirmation as a JSON response.

### State

All state is stored in YAML front-matter of the affected artifact files. No separate index or cache is maintained. Add/remove operations are idempotent — adding a domain that already exists or removing one that doesn't is a no-op without error.

### Behaviour

1. **Domain add/remove** — reads current `domains` list, applies the add/remove delta, deduplicates, and writes back. Domain names are validated against the `[domains]` vocabulary in `product.toml`; unknown names produce E012 and abort without writing.
2. **Domain acknowledgement** — reads current `domains-acknowledged` map, adds or replaces the entry for the specified domain key with the provided reason. `--remove` deletes the key. Empty or whitespace-only reason produces E011 and aborts.
3. **ADR scope** — overwrites the `scope` field with the validated enum value. One of three values only; any other string produces E001.
4. **ADR supersession** — acquires the advisory write lock, reads both ADR files, validates existence and absence of cycles in the supersession graph (E004 on cycle detected), writes the first file (adds to `supersedes`), then writes the second file (adds to `superseded-by` and updates `status` to `superseded` if currently `accepted`). Both writes use atomic temp-file rename. If the second write fails, the first is committed and the error message instructs the user to complete manually or re-run (the operation is idempotent).
5. **ADR source-files** — reads current `source-files` list, applies add/remove delta. Paths that do not exist on disk at command time produce a W-class warning (not an error) because the source file may not exist yet during authoring.
6. **TC runner config** — overwrites `runner`, `runner-args`, `runner-timeout`, and/or `requires` fields. Runner enum and prerequisite names are validated before writing.
7. **MCP gating** — all write tools are gated behind `mcp.write = true` in `product.toml`. Read tools are unaffected.

### Invariants

- All add/remove operations are idempotent: repeated calls with the same arguments produce the same result without error.
- Domain names in `feature.domains` and `adr.domains` must always be drawn from the `[domains]` vocabulary in `product.toml`. E012 is emitted if the vocabulary check fails.
- `adr supersede` always produces a consistent bidirectional link: if `ADR-036.supersedes` contains `ADR-035`, then `ADR-035.superseded-by` contains `ADR-036`. Half-links are not possible through the tool.
- Cycle detection in the supersession graph runs before any file is written. A cycle (A supersedes B supersedes A) is rejected with E004.
- `feature acknowledge --reason` is never empty or whitespace-only; E011 is enforced before writing.

### Error handling

- **E001** — invalid enum value (scope or runner); names the field and the valid options.
- **E002** — artifact ID not found (target of supersession or linked artifact); names the missing ID.
- **E004** — supersession cycle detected; names the cycle path.
- **E011** — acknowledgement reason is empty or whitespace-only; aborts without writing.
- **E012** — domain name not in `product.toml` `[domains]` vocabulary; names the invalid domain and hints to add it to config.
- **Partial failure in `adr supersede`** — if the second file write fails after the first succeeds, the error message names both files and instructs the user to manually complete the second update or re-run the command.

### Boundaries

- These commands mutate front-matter fields only. They do not modify the markdown body of any artifact.
- `product adr supersede --remove` reverses the `supersedes`/`superseded-by` link but does not automatically restore the superseded ADR's status — status restoration is a deliberate manual step.
- Source-file path existence is a warning, not an error — authoring precedes implementation, and the source file may not exist yet.
- All tools are write-gated via `mcp.write`. Read tools (`product graph check`, `product impact`, etc.) are not affected by this feature.

## Out of scope

- Batch field mutations across multiple artifacts in a single command invocation.
- Generic `product frontmatter set ARTIFACT FIELD VALUE` — rejected in ADR-037 in favour of typed, validated per-field tools.
- Mutation of body text — that is `product adr body-update` / `product feature body-update`.
- Changing artifact `id` or `title` — these are immutable once set (ADR-032 convention for ADRs; convention for features and dependencies).
- Authoring flow prompts or interactive sessions — these tools are building blocks for the authoring flow; the flow itself is defined in ADR-022 and the authoring system prompts.
