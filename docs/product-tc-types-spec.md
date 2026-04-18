# Product Test Criterion Types Specification

> Standalone reference for the TC type system — fixed structural types and
> customisable descriptive types.
>
> New product.toml section: `[tc-types]`
> New validation: E006 on unknown type, E017 on reserved type in custom list

---

## Overview

The `type` field on a TC answers: **what kind of assertion is this?**

Types fall into two categories with different rules:

| Category | Types | Customisable? | Product behaviour |
|---|---|---|---|
| Structural | `exit-criteria`, `invariant`, `chaos`, `absence` | **No — reserved** | Product drives mechanics from these |
| Descriptive | `scenario`, `benchmark` | **Supplementable** | Product treats them uniformly |

**Structural types are fixed.** Product uses them to drive phase gate evaluation,
formal block requirements (W004), gap codes (G002, G009), and the verify pipeline.
They cannot be renamed, removed, or redefined.

**Descriptive types are open.** `scenario` and `benchmark` are built-in defaults.
Teams add their own via `[tc-types].custom` in `product.toml`. Product treats all
descriptive types — built-in and custom — identically in mechanics. The type name
is a signal to agents and humans, not an instruction to Product.

---

## Structural Types — Reserved

### `exit-criteria`

Used by the **phase gate** (ADR-012). `phase_gate_satisfied(N)` checks that all
TCs of type `exit-criteria` linked to features in phase N are passing before
allowing phase N+1 features to be returned by `product feature next`.

Also used in context bundle ordering — exit-criteria TCs appear first so an agent
sees completion conditions before other constraints.

**Cannot be customised, renamed, or removed.**

### `invariant`

Triggers **W004** (formal block requirement) — an invariant TC without a
`⟦Γ:Invariants⟧` or `⟦Σ:Types⟧` formal block is a specification quality gap.

Also referenced by **G002** — an ADR with a `⟦Γ:Invariants⟧` block but no linked
`invariant` or `chaos` TC is a gap finding.

**Cannot be customised, renamed, or removed.**

### `chaos`

Triggers **W004** alongside `invariant`.

Referenced by **G002** alongside `invariant`.

Semantically distinct from `invariant` — a chaos TC exercises failure paths
(network partition, node kill, disk failure) rather than correctness properties
under normal operation. Both require formal specification blocks.

**Cannot be customised, renamed, or removed.**

### `absence`

Triggers **G009** and **W022** — an ADR with `removes` or `deprecates` entries
but no linked `absence` TC is a specification gap. (See product-removal-deprecation-spec.md.)

Semantically distinct — asserts that something which should be gone is in fact
absent, or that a deprecated thing produces the correct warning.

**Cannot be customised, renamed, or removed.**

---

## Descriptive Types — Built-in

### `scenario`

A narrative test case: given precondition, when action, then outcome. The most
common TC type. No special Product mechanics — included in context bundles,
runs via the configured runner, status tracked normally.

Referenced by **G002** — ADR invariant blocks should have linked scenario or
chaos TCs.

### `benchmark`

A performance measurement. Asserts that some operation completes within a time
or resource bound. No special Product mechanics beyond runner execution and
status tracking. Benchmark TCs are typically at `level: unit` or `level: component`
for fast execution, or `level: system` for hardware-realistic numbers.

---

## Custom Descriptive Types

Teams declare custom types in `product.toml`:

```toml
[tc-types]
custom = ["contract", "migration", "smoke", "load", "end-to-end"]
```

Any value in this list becomes a valid `type` value for TCs in this repository.
Product treats custom types identically to `scenario` in all mechanics — they
appear in context bundles, run via their configured runner, and have status
tracked. The type name is purely descriptive: a signal to agents and humans
about the nature of the assertion.

### Common custom type patterns

**`contract`** — API contract tests. Asserts that a service matches a published
interface specification (OpenAPI, AsyncAPI, Pact). Typically at `level: integration`.

```yaml
---
id: TC-099
title: Orders API response matches OpenAPI contract v2.1
type: contract
level: integration
runner: bash
runner-args: ["scripts/test-harness/openapi-contract.sh", "orders", "v2.1"]
validates:
  adrs: [ADR-018]
  features: [FT-012]
---
```

**`migration`** — database or schema migration tests. Asserts that a migration
applies cleanly, is idempotent, and is reversible. Typically at `level: component`
or `level: integration`.

```yaml
---
id: TC-100
title: Migration 0042 applies and rolls back cleanly
type: migration
level: component
runner: bash
runner-args: ["scripts/test-harness/migration-roundtrip.sh", "0042"]
validates:
  adrs: [ADR-021]
  features: [FT-015]
---
```

**`smoke`** — post-deployment smoke tests. A minimal set of checks that the
system is alive after a deploy. Typically at `level: acceptance`.

```yaml
---
id: TC-101
title: API health endpoint responds within 2s after deployment
type: smoke
level: acceptance
runner: bash
runner-args: ["scripts/test-harness/smoke-health.sh"]
requires: [deployment-complete]
validates:
  adrs: [ADR-022]
  features: [FT-001, FT-002]
---
```

**`load`** — load and stress tests. Asserts behaviour under sustained traffic.
Typically at `level: system` or `level: acceptance`.

**`end-to-end`** — full user journey tests driven by a real client. Typically
at `level: acceptance`.

**`property`** — property-based tests asserting over generated inputs. Can be
at any level depending on whether I/O is involved.

```toml
# Example for a .NET project migrating off AutoMapper
[tc-types]
custom = ["contract", "migration", "smoke"]
```

```toml
# Example for a distributed platform
[tc-types]
custom = ["load", "end-to-end", "property"]
```

---

## Validation Rules

### E006 — Unknown TC type

`product graph check` and `product request validate` emit E006 if a TC declares
a `type` that is neither a built-in type nor in `[tc-types].custom`:

```
error[E006]: unknown TC type 'regression' in TC-042
  TC-042: Leader election regression test
  'regression' is not a built-in type and is not in [tc-types].custom.

  Built-in types: exit-criteria, invariant, chaos, absence, scenario, benchmark
  Custom types:   contract, migration, smoke  (from product.toml [tc-types])

  To add 'regression' as a custom type:
    product request change:
      target: product.toml [tc-types].custom
      op: append, value: regression
  Or use the closest built-in type: scenario
```

### E017 — Reserved type in custom list

`product.toml` validation emits E017 if `[tc-types].custom` contains a reserved
structural type name:

```
error[E017]: reserved type 'exit-criteria' in [tc-types].custom
  product.toml: custom = ["contract", "exit-criteria"]
                                       ^^^^^^^^^^^^^
  'exit-criteria' is a structural type reserved by Product.
  Structural types drive Product mechanics and cannot be redefined.

  Reserved names: exit-criteria, invariant, chaos, absence
```

This fires at startup — Product refuses to load a configuration that would
silently shadow a structural type.

### Context bundle ordering

Within a context bundle, TCs are ordered by type in a fixed sequence:

```
1. exit-criteria    (completion conditions — agent reads these first)
2. invariant        (properties that must always hold)
3. chaos            (failure-mode assertions)
4. absence          (removal mandates)
5. scenario         (narrative test cases)
6. benchmark        (performance bounds)
7. [custom types]   (alphabetical within custom)
```

Custom types always sort after built-in descriptive types. This ordering ensures
agents encounter completion criteria and invariants before detailed test cases.

---

## `product.toml` Schema

```toml
# [tc-types] is optional — only needed if you use custom types.
# If absent, only built-in types are valid.

[tc-types]
# Custom descriptive types for this repository.
# Product behaviour for custom types is identical to 'scenario'.
# Reserved names (exit-criteria, invariant, chaos, absence) may not appear here.
custom = []

# Examples:
# custom = ["contract", "migration", "smoke"]
# custom = ["load", "end-to-end", "property"]
```

---

## Effect on AGENT.md

`product agent-init` includes the full type vocabulary in the generated `AGENT.md`
schema section, so agents always see the current type list without needing to check
`product.toml`:

```markdown
### Test Criterion (TC-XXX)

type: scenario        # exit-criteria | invariant | chaos | absence
                      # scenario | benchmark         (built-in descriptive)
                      # contract | migration | smoke  (custom — this project)
```

The structural types are listed first with their mechanics noted. Custom types
are listed with the "(custom — this project)" annotation.

---

## `product request create` — specifying custom type

Custom types work identically to built-in types in create requests:

```yaml
type: create
reason: "Add contract test for Orders API"
artifacts:
  - type: tc
    ref: tc-orders-contract
    title: Orders API matches OpenAPI contract
    tc-type: contract        # custom type
    level: integration
    validates:
      features: [FT-012]
      adrs: [ADR-018]
```

`product request validate` checks that `contract` is in `[tc-types].custom`
before accepting the request. If not, E006 with a hint to add it.

---

## New Validation Codes

| Code | Tier | Condition |
|---|---|---|
| E006 | Validation | TC declares a `type` that is neither a built-in type nor in `[tc-types].custom` |
| E017 | Configuration | `[tc-types].custom` contains a reserved structural type name |

E006 is the user-facing error for misconfigured TCs. E017 is the startup guard
that prevents accidental structural type shadowing.

---

## Session Tests

```
# Built-in type validation
ST-180  tc-type-exit-criteria-drives-phase-gate
ST-181  tc-type-invariant-requires-formal-block
ST-182  tc-type-chaos-requires-formal-block
ST-183  tc-type-absence-drives-g009

# Custom type validation
ST-184  custom-type-valid-when-in-toml
ST-185  custom-type-e006-when-not-in-toml
ST-186  custom-type-treated-as-scenario-in-mechanics
ST-187  custom-type-appears-in-agent-md-schema
ST-188  custom-type-appears-in-context-bundle-after-builtins

# Reserved type guard
ST-189  e017-reserved-type-in-custom-list
ST-190  e017-fires-at-startup-not-lazily

# Ordering
ST-191  bundle-type-ordering-exit-criteria-first
ST-192  bundle-type-ordering-custom-types-last-alphabetical

# Request integration
ST-193  request-create-with-custom-type-validates-against-toml
ST-194  request-create-unknown-type-emits-e006
```

---

## Invariants

- **Structural types are immutable.** The four structural type names
  (`exit-criteria`, `invariant`, `chaos`, `absence`) are compiled into Product.
  No configuration can change their meaning or disable their mechanics.
- **Custom types have no mechanics.** Product never inspects the value of a custom
  type beyond checking it is declared in `[tc-types].custom`. All Product behaviour
  driven by type (phase gate, W004, G002, G009) uses only structural type names.
- **E017 fires at startup.** A configuration with a reserved type in `[tc-types].custom`
  is rejected before any command runs. There is no way for a custom type to silently
  shadow a structural type at runtime.
- **Context bundle ordering is stable.** The ordering sequence is fixed in Product.
  Custom types always sort after all built-in types, alphabetically among themselves.
  Adding a new custom type never changes the position of existing TCs in a bundle.
