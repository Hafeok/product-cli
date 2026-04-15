---
id: TC-472
title: product.toml parses product responsibility field
type: scenario
status: passing
validates:
  features:
  - FT-039
  adrs: []
phase: 1
runner: cargo-test
runner-args: "tc_472_product_toml_parses_product_responsibility_field"
last-run: 2026-04-15T11:22:02.279019545+00:00
last-run-duration: 0.4s
---

**Given** a product.toml with a `[product]` section containing `name` and `responsibility` fields
**When** `ProductConfig::load()` parses the file
**Then** `config.product.name` equals the declared name AND `config.product.responsibility` equals the declared responsibility string

**Given** a product.toml without a `[product]` section
**When** `ProductConfig::load()` parses the file
**Then** `config.product.responsibility` is `None` AND `config.product.name` falls back to the top-level `name` field

```
⟦Γ:Invariants⟧{
  ∀ config ∈ ProductConfig:
    config.product.responsibility.is_some() ⟹
      config.product.responsibility.unwrap().len() > 0
}
```