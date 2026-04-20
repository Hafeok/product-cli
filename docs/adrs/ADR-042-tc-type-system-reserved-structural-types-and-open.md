---
id: ADR-042
title: TC Type System — Reserved Structural Types and Open Descriptive Types
status: accepted
features:
- FT-048
supersedes: []
superseded-by: []
domains:
- api
- data-model
- error-handling
scope: cross-cutting
content-hash: sha256:a6f6606f62c03cbad2b7822a35fec992b5e3e8732b5c92c18a9f45db88f9fcad
---

**Status:** Proposed

**Context:** The `type` field on a TC is currently an enum (`scenario | invariant
| chaos | exit-criteria`, with `benchmark` and `absence` as later additions in
ADR-011 and ADR-041). Product behaviour is driven by this field:

- `exit-criteria` drives the phase gate (ADR-012, ADR-034).
- `invariant` and `chaos` trigger W004 (formal block requirement) and are
  referenced by G002 (ADR with `⟦Γ:Invariants⟧` block and no linked TC).
- `absence` triggers G009/W022 (ADR-041).

Two problems:

1. **The vocabulary is closed at the wrong level.** Real teams use type names
   like `contract`, `migration`, `smoke`, `load`, `end-to-end`, `property`. They
   carry intent that helps reviewers and agents triage tests. Today, all of
   these collapse to `scenario`, losing the distinction. The user's mental
   model — "this is a contract test" — is invisible to the graph.

2. **The mechanic-bearing types are not distinguished from labels.** Adding a
   new descriptive type (e.g. `smoke`) is not the same kind of change as adding
   a new mechanic-bearing type (e.g. another `phase-gate-driving` type).
   Conflating the two means every type-name addition risks accidentally
   collision with a reserved identifier or silently introducing new mechanics.

The spec at `docs/product-tc-types-spec.md` describes the resolution in detail.
This ADR pins the architectural decisions; the spec covers operational detail.

---

**Decision:** Partition TC types into two categories with different rules.

- **Structural types** (`exit-criteria`, `invariant`, `chaos`, `absence`) are
  **reserved**. Their names are compiled into Product. Their mechanics — phase
  gate evaluation, formal-block W004, G002, G009, verify pipeline routing —
  are driven by exact matches against these names. They cannot be renamed,
  removed, or redefined.
- **Descriptive types** (`scenario`, `benchmark` built-in; user-supplied via
  `[tc-types].custom` in `product.toml`) are **open**. Product treats them
  identically in mechanics — included in context bundles, run via the
  configured runner, status tracked normally. The type name is a signal to
  agents and humans about the nature of the assertion; Product carries no
  opinion beyond the name.

---

### 1. The four structural types

| Type | Mechanic |
|---|---|
| `exit-criteria` | Phase gate — `phase_gate_satisfied(N)` requires every `exit-criteria` TC linked to phase-N features to be `passing`. Bundle ordering: first. |
| `invariant` | W004 (formal block required); G002 satisfier. |
| `chaos` | W004 alongside `invariant`; G002 satisfier. |
| `absence` | G009/W022 satisfier (ADR-041); routed to `verify --platform`. |

Each mechanic is a function whose only TC-type input is one of these four
strings. The mapping is hard-coded; configuration cannot change it.

### 2. The two built-in descriptive types

| Type | Purpose |
|---|---|
| `scenario` | Default. Narrative given/when/then test case. G002 satisfier (alongside `chaos`/`invariant`). |
| `benchmark` | Performance assertion. Identical mechanics to `scenario`. |

### 3. Custom descriptive types

Teams declare additional descriptive types in `product.toml`:

```toml
[tc-types]
custom = ["contract", "migration", "smoke", "load", "end-to-end", "property"]
```

A custom type passes type validation iff it appears in `[tc-types].custom`. In
every other respect it behaves identically to `scenario`. Product never branches
on a custom type's name beyond the validation lookup.

### 4. New validation codes

| Code | Tier | Severity | Condition |
|---|---|---|---|
| E006 (refined) | Validation | error | TC declares a `type` that is neither a built-in type nor in `[tc-types].custom` |
| E017 | Configuration | error | `[tc-types].custom` contains a reserved structural type name |

E006 surfaces in `product graph check` and `product request validate` with a
hint listing both built-in and configured custom types and showing the
`product request change` invocation that would add the missing custom type.

E017 fires at startup — Product refuses to load a configuration that would
shadow a structural type. This guarantees no custom type can accidentally
hijack a structural mechanic at runtime.

### 5. Context bundle ordering

Within a context bundle, TCs are ordered by type:

```
1. exit-criteria
2. invariant
3. chaos
4. absence
5. scenario
6. benchmark
7. [custom types alphabetical]
```

Built-in types always sort before custom types. Custom types sort
alphabetically among themselves. The ordering is stable: adding or removing a
custom type never reorders existing built-in TCs.

This ordering refines the rule in FT-011 (Context Bundle Format), which
previously sorted as `exit-criteria → scenario → invariant → chaos`. The
revision moves invariant/chaos ahead of scenario so agents see properties and
failure-mode assertions before narrative test cases. Existing bundles remain
valid; the ordering change is forward-compatible because no consumer depends on
a specific position beyond "exit-criteria first".

### 6. AGENT.md / agent-context rendering

`product agent-init` and `product agent-context` render the type vocabulary
with the structural / descriptive partition visible:

```markdown
### Test Criterion (TC-XXX)
type: scenario        # exit-criteria | invariant | chaos | absence  (structural)
                      # scenario | benchmark                        (descriptive)
                      # contract | migration | smoke                (custom — this project)
```

Custom types are listed with the "(custom — this project)" annotation, sourced
from the active `product.toml`. An agent receiving the rendered schema sees the
full type vocabulary without consulting `product.toml` separately.

---

⟦Γ:Invariants⟧{
  structural_types_are_a_fixed_compiled_set_of_four
  structural_type_names_cannot_appear_in_tc_types_custom
  e017_fires_at_startup_before_any_command_runs
  custom_types_drive_no_product_mechanics_beyond_validation
  bundle_ordering_places_built_in_types_before_custom_types
  bundle_ordering_for_custom_types_is_alphabetical
  e006_lists_both_built_in_and_configured_custom_types_in_its_hint
  adding_or_removing_a_custom_type_does_not_reorder_existing_builtin_tcs
}

⟦Ε⟧⟨δ≜1.0;φ≜100;τ≜◊⁺⟩

**Evidence TCs:** TC-601 (exit-criteria → phase gate), TC-602 (invariant W004),
TC-603 (chaos W004), TC-604 (absence G009), TC-605 (custom valid in toml),
TC-606 (custom missing → E006), TC-607 (custom mechanics = scenario), TC-608
(custom in agent-md), TC-609 (custom in bundle after builtins), TC-610 (E017
reserved name in custom), TC-611 (E017 startup), TC-612 (bundle ordering exit
first), TC-613 (custom alphabetical last), TC-614 (request create with custom
validates), TC-615 (request unknown type → E006), TC-616 (consolidated exit
criteria).

---

**Rationale:**

- **Structural / descriptive partition matches what Product actually does.**
  Four type names already drive Product mechanics by exact-string match
  (phase-gate, W004, G002, G009). Two type names (`scenario`, `benchmark`)
  carry no mechanics. Naming the partition makes the existing reality
  legible and gives the type-name extension story a clear safe edge.
- **Custom types as a closed-list opt-in is the cheapest correct shape.** A
  free-form string field would mean any typo silently becomes a new "type"
  and hides validation errors. A schema-defined enum forces a Product release
  for every team-specific naming choice. The middle path — built-in set plus
  a `[tc-types].custom` list in `product.toml` — gives teams freedom while
  preserving "spelt it wrong" detection.
- **E017 fires at startup, not lazily, so a malformed config cannot run the
  wrong mechanics.** A custom type named `invariant` would be a disaster: TCs
  could declare it without a formal block (no W004 because it would now look
  custom), bypassing the gap. Refusing to load the config is the only safe
  behaviour. The cost is small — `product.toml` is read at startup anyway.
- **Bundle ordering revision is a forward-compatible refinement of FT-011.**
  The earlier ordering predates `absence` and didn't account for the
  agent-cognitive value of seeing properties before scenarios. The revision
  is small and the consumers (agents) are tolerant of finer-grained
  ordering. No break.
- **AGENT.md surfaces the vocabulary at the agent's read-point.** Agents
  consult AGENT.md for schema; teaching them to also consult `product.toml`
  for the same answer doubles the surface area and creates skew. Render the
  full vocabulary in AGENT.md and there is one source of truth.

**Rejected alternatives:**

- **Make all types structural — no custom types.** Forces every team-specific
  naming choice through a Product release. Rejected as hostile to the primary
  user (a team curating their own test taxonomy).
- **Make all types descriptive — no reserved set.** Pulls the four mechanic
  anchors out of the type field and forces Product to discover phase-gate /
  W004 / G002 / G009 candidates by some other marker (a separate field, a
  title prefix, a tag list). Rejected because the existing semantics already
  live on `type` and any alternative is a worse fit.
- **Allow custom types to declare mechanics in `product.toml`.** E.g.
  `[tc-types.smoke] phase-gate = true`. Rejected because it lets a config
  file change the meaning of Product's exit-code contract — a security and
  debuggability hazard. The four structural mechanics are too central to be
  configurable.
- **Single open list, structural mechanics by separate flag on each TC.** A
  `phase-gate: true` boolean on each `exit-criteria` TC instead of relying on
  the type. Rejected as it lets two TCs of the same `type` differ in
  mechanics, which violates the "type drives behaviour" invariant and breaks
  every existing TC analysis tool.
- **Validate custom-type lookups warning-only (W instead of E).** Rejected
  because a typo in a `type:` field producing a passing graph check is a
  silent specification bug. E-class is correct: an unknown type either gets
  added to `[tc-types].custom` or corrected to a built-in.
- **Ship a curated catalogue of "blessed" descriptive types** (contract,
  migration, smoke, ...) as built-ins. Rejected because every name added to
  the built-in set is a new Product release dependency for teams. The curated
  catalogue lives in the spec as worked examples; teams add what they need.

**Test coverage:** TC-601 through TC-616 (FT-048). See the standalone spec at
`docs/product-tc-types-spec.md` for the session-test-to-TC mapping (ST-180 →
TC-601, etc.) and full configuration examples.
