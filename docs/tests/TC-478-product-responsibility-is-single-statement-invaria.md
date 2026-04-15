---
id: TC-478
title: product responsibility is single statement invariant
type: invariant
status: passing
validates:
  features:
  - FT-039
  adrs: []
phase: 1
runner: cargo-test
runner-args: "tc_478_product_responsibility_is_single_statement_invariant"
last-run: 2026-04-15T11:22:02.279019545+00:00
last-run-duration: 0.2s
---

```
⟦Γ:Invariants⟧{
  ∀ config ∈ ProductConfig:
    config.product.responsibility.is_some() ⟹
      ¬ contains_top_level_conjunction(config.product.responsibility.unwrap())
  
  -- A product responsibility statement that contains " and " as a top-level
  -- conjunction (not within a subordinate clause) indicates two products,
  -- not one. This mirrors ADR-029's single-responsibility rule.
}
```

**Given** a responsibility statement containing a top-level " and " conjunction
**When** `ProductConfig::load()` parses the file
**Then** a validation warning is emitted indicating the responsibility may describe multiple products

**Given** a responsibility statement with " and " only inside subordinate clauses (e.g., "no configuration and no external dependencies" as a list within a single statement)
**When** `ProductConfig::load()` parses the file  
**Then** no warning is emitted — subordinate conjunctions within a single statement are acceptable