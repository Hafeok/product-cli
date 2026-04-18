---
id: FT-038
title: Front-Matter Field Management
phase: 5
status: complete
depends-on: []
adrs:
- ADR-037
- ADR-002
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
  ADR-042: Pre-dates ADR-042; this feature does not define TC types or validate the type vocabulary. FT-048 owns the mechanics.
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