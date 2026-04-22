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
  ADR-041: Pre-dates ADR-041; this feature does not author absence TCs or set removes/deprecates on ADRs. FT-047 owns the mechanics.
  ADR-043: Predates ADR-043; feature command adapters were written before the slice+adapter pattern was formalised and are not retroactively refactored.
  ADR-042: Pre-dates ADR-042; this feature does not define TC types or validate the type vocabulary. FT-048 owns the mechanics.
  ADR-018: Predates the 2026-04-22 scope promotion of ADR-018 to cross-cutting. Test coverage reflects the property/session/benchmark strategy as it existed when this feature shipped; not retroactively reclassified.
  ADR-040: predates ADR-040; feature ships independently of the unified verify / LLM-boundary pipeline
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