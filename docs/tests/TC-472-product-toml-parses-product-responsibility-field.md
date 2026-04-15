---
id: TC-472
title: product.toml parses product responsibility field
type: scenario
status: unimplemented
validates:
  features: []
  adrs: []
phase: 1
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