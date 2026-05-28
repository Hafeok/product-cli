---
id: FT-039
title: Product Responsibility Statement
phase: 1
status: complete
depends-on: []
adrs:
- ADR-006
- ADR-013
- ADR-022
tests:
- TC-472
- TC-473
- TC-474
- TC-475
- TC-476
- TC-477
- TC-478
- TC-479
domains: []
domains-acknowledged:
  ADR-018: Predates the 2026-04-22 scope promotion of ADR-018 to cross-cutting. Test coverage reflects the property/session/benchmark strategy as it existed when this feature shipped; not retroactively reclassified.
  ADR-048: Predates ADR-048; this feature does not author the canonical .product/ folder layout. FT-057 owns the migration command and discovery fallback.
  ADR-047: Predates ADR-047; this feature does not author the functional-spec body convention. FT-055 owns the structural validator and W030 mechanics.
  ADR-043: Predates ADR-043; feature command adapters were written before the slice+adapter pattern was formalised and are not retroactively refactored.
  ADR-040: predates ADR-040; feature ships independently of the unified verify / LLM-boundary pipeline
  ADR-041: Pre-dates ADR-041; this feature does not author absence TCs or set removes/deprecates on ADRs. FT-047 owns the mechanics.
  ADR-042: Pre-dates ADR-042; this feature does not define TC types or validate the type vocabulary. FT-048 owns the mechanics.
---

`product.toml` gains a `[product]` section with a `responsibility` field — a single statement declaring what the product is and what it is not. This field is the semantic scope boundary for all specification work. It is not a mechanical filter; it is a reference point that tools surface at the right moments.

### product.toml Schema

```toml
[product]
name = "picloud"
responsibility = """
  A single-binary private cloud platform for Raspberry Pi 5 clusters
  that turns bare nodes into an elastic, event-sourced, self-managing
  compute environment — no external dependencies, no configuration,
  no infrastructure to run the infrastructure.
"""
```

The `responsibility` field is a single statement. Same constraint as ADR-029's single-responsibility rule: one statement, no "and" at the top level. If you can't describe the product without "and," it's two products.

The `name` field already exists in product.toml (top-level). The `[product]` section groups it alongside `responsibility`. The top-level `name` remains as an alias for backward compatibility; `[product].name` takes precedence if both are present.

### MCP Tool: `product_responsibility`

A new read-only MCP tool that returns the product name and responsibility statement:

```json
{
  "name": "picloud",
  "responsibility": "A single-binary private cloud platform for Raspberry Pi 5 clusters..."
}
```

Returns an error if `responsibility` is not set in product.toml. This tool is the first call an agent makes in any session — before reading the feature list, before reading ADRs, the agent knows what the product is.

### Context Bundle Header

The ⟦Ω:Bundle⟧ header block gains a `responsibility` field (amendment to ADR-006):

```
⟦Ω:Bundle⟧{
  product≜picloud:Product
  responsibility≜"single-binary private cloud for Raspberry Pi 5 clusters"
  feature≜FT-001:Feature
  phase≜1:Phase
  status≜InProgress:FeatureStatus
  generated≜2026-04-11T09:00:00Z
  implementedBy≜⟨ADR-001,ADR-002⟩:Decision+
  validatedBy≜⟨TC-001,TC-002⟩:TestCriterion+
}
```

When `responsibility` is not configured, the `product` and `responsibility` lines are omitted — bundles from repositories without the field remain unchanged.

### Authoring Scope Gate

The authoring prompts (ADR-022) gain a step 0: read `product_responsibility` before anything else. The prompt excerpt:

```markdown
Before writing any content:
0. Call product_responsibility — understand what this product is and is not
1. Call product_feature_list — understand what features exist
2. Call product_graph_central — identify the top-5 foundational ADRs
...
```

When an agent proposes a feature that is outside the declared responsibility, the responsibility statement is the basis for the agent to flag the mismatch — "this appears to be outside the declared scope of PiCloud — are you sure?"

### Validation: W019

`product graph check` gains warning W019 (amendment to ADR-013):

| Code | Tier | Description |
|---|---|---|
| W019 | Validation | Feature title/description appears outside the declared product responsibility |

W019 is a **warning**, not an error. Sometimes a product needs scaffolding features that aren't directly in scope. The warning surfaces them for review; it doesn't block them.

The check is deliberately loose: features should be **derivable** from the responsibility through a chain of reasoning. Infrastructure features, tooling features, and enablement features are expected. A feature that cannot be traced back to the responsibility at all is a W019 candidate.

W019 requires the `responsibility` field to be set. When responsibility is absent, W019 is not emitted for any feature — it is not an error to omit the field, the check simply doesn't activate.

### CLI Surface

No new CLI command. The responsibility field is read from product.toml by existing commands:

- `product context FT-XXX` — includes responsibility in the ⟦Ω:Bundle⟧ header
- `product graph check` — emits W019 when appropriate
- `product agent-init` — includes responsibility in AGENTS.md generation
- MCP: `product_responsibility` — standalone read tool

### Relationship to Existing Config

The current product.toml has `name = "product-cli"` at the top level. The `[product]` section is new. Migration path:

1. If `[product]` section exists, use `[product].name` and `[product].responsibility`
2. If `[product]` section does not exist, fall back to top-level `name`; `responsibility` is None
3. `product init` scaffolds the `[product]` section in new repositories
4. No schema version bump — the field is optional with a graceful fallback

---

## Description

Adds a `[product]` section to `product.toml` with a `responsibility` field — a single statement declaring what the product is and is not. The field is surfaced in context bundle headers (`product context FT-XXX`), in the MCP `product_responsibility` read tool, and in `AGENTS.md` generation. `product graph check` gains warning W019 for features whose title or description appears outside the declared responsibility.

## Functional Specification

### Inputs

- `product.toml` `[product]` section — `name` and `responsibility` fields. `responsibility` is a multi-line string. The field is optional; when absent, responsibility-dependent behaviour is suppressed rather than erroring.
- `product graph check` — reads `responsibility` from config; emits W019 for features that cannot be traced back to the declared scope.
- `product context FT-XXX` — reads `responsibility` from config to populate the ⟦Ω:Bundle⟧ header.
- `product agent-init` — reads `responsibility` to include in the generated `AGENTS.md`.
- MCP tool call `product_responsibility` (no parameters).
- Existing top-level `name` field in `product.toml` — falls back to this when `[product]` section is absent; backward compatible.

### Outputs

- **`product context FT-XXX` bundle header** — when `responsibility` is set, the ⟦Ω:Bundle⟧ header includes `product≜NAME:Product` and `responsibility≜"..."` lines. When absent, these lines are omitted and existing bundle output is unchanged.
- **MCP `product_responsibility`** — JSON object `{"name": "...", "responsibility": "..."}`. Returns an error if `responsibility` is not configured.
- **`product graph check` — W019** — warning when a feature's title or description appears outside the declared product responsibility. Exit code 2 (warning-only). Suppressed entirely when `responsibility` is not set.
- **`product agent-init` — AGENTS.md** — responsibility statement included in the generated file when configured.
- **`product init`** — scaffolds the `[product]` section in new repositories with empty `responsibility` value and a comment prompt.

### State

The `responsibility` field is persisted in `product.toml` as a TOML multi-line string under the `[product]` section. It is read on every invocation of the commands listed above; no cache is maintained. The field is optional; its absence is a valid configuration state that silently disables W019 and omits the field from bundle headers.

### Behaviour

1. **Config parsing** — `ProductConfig` reads `[product].name` and `[product].responsibility`. When `[product]` is absent, `config.responsibility()` returns `None`. The top-level `name` key remains valid as an alias for backward compatibility; `[product].name` takes precedence when both are present.
2. **Bundle header injection** — `product context FT-XXX` calls `config.responsibility()`. If `Some`, it prepends `product` and `responsibility` lines to the ⟦Ω:Bundle⟧ header block. If `None`, the header is identical to pre-feature output.
3. **W019 check** — `graph::responsibility::check_responsibility` is called by `product graph check` after structural validation. It is a loose check: features should be derivable from the responsibility through a chain of reasoning. Infrastructure and enablement features are expected and pass. The check is deliberately not a hard gate — W019 is always exit code 2.
4. **MCP `product_responsibility`** — a read tool that returns the name and responsibility as a JSON object. It is the recommended first call in any authoring session before reading the feature list or ADRs.
5. **Single-statement invariant** — the responsibility field should be one statement without "and" at the top level (same SRP constraint as ADR-029's module doc-comment rule). This is documented guidance, not mechanically enforced by the tool.

### Invariants

- W019 is never emitted when `responsibility` is not configured — the check does not activate.
- W019 is always a warning (exit 2), never a hard error — some scaffolding and tooling features may legitimately lie outside the declared scope.
- The bundle header change is additive and backward-compatible — repositories without `responsibility` produce identical bundle output to pre-feature behaviour.
- MCP `product_responsibility` returns an error (not a silent empty result) when `responsibility` is absent, so agents can distinguish "not configured" from "configured but empty".

### Error handling

- **`product_responsibility` with no `responsibility` field** — returns a tool error naming the missing field and hinting to add `[product] responsibility = "..."` to `product.toml`.
- **W019** — feature title or description appears outside declared responsibility. Exit code 2. Message names the feature and the responsibility statement. Never blocks any operation.
- **`product.toml` parse error in `[product]` section** — propagates as `ProductError::ConfigError` via the standard config loading path; all commands that load config will fail with the same error.

### Boundaries

- No new CLI command is added. The responsibility field is read by existing commands (`product context`, `product graph check`, `product agent-init`) and the new MCP read tool.
- W019 is informational and advisory. It does not gate `product verify`, `product feature status`, or any other workflow transition.
- The `responsibility` constraint (single statement, no "and") is documented guidance only — the tool accepts any non-empty string.
- `product init` scaffolds the section in new repositories but does not enforce migration of existing repositories. W019 is simply never emitted for repos without the field.

## Out of scope

- Automatic enforcement of the single-statement constraint via syntax analysis — this is a stylistic guideline, not a parse-time rule.
- Machine-learning-based scope classification for W019 — the check is intentionally loose and LLM-driven rather than deterministic.
- Per-feature responsibility overrides — every feature is evaluated against the single product-level responsibility statement.
- Responsibility versioning or changelog — the field is a mutable config value; changes are tracked via git history of `product.toml`, not by Product itself.
