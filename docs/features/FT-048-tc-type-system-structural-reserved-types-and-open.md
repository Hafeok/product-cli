---
id: FT-048
title: TC Type System — Structural Reserved Types and Open Descriptive Types
phase: 5
status: complete
depends-on:
- FT-011
- FT-018
- FT-029
- FT-047
adrs:
- ADR-011
- ADR-012
- ADR-013
- ADR-019
- ADR-041
- ADR-042
tests:
- TC-601
- TC-602
- TC-603
- TC-604
- TC-605
- TC-606
- TC-607
- TC-608
- TC-609
- TC-610
- TC-611
- TC-612
- TC-613
- TC-614
- TC-615
- TC-616
domains:
- api
- data-model
- error-handling
domains-acknowledged:
  ADR-048: Predates ADR-048; this feature does not author the canonical .product/ folder layout. FT-057 owns the migration command and discovery fallback.
  ADR-043: Predates ADR-043; feature command adapters were written before the slice+adapter pattern was formalised and are not retroactively refactored.
  ADR-018: Predates the 2026-04-22 scope promotion of ADR-018 to cross-cutting. Test coverage reflects the property/session/benchmark strategy as it existed when this feature shipped; not retroactively reclassified.
  ADR-047: Predates ADR-047; this feature does not author the functional-spec body convention. FT-055 owns the structural validator and W030 mechanics.
  ADR-040: TC type system is a data-model concern; verify pipeline's stage-6 already discovers absence TCs by tc-type (owned by FT-047) without hooks from this feature.
---

## Description

Product's `type:` field on a TC drives four hard-wired mechanics: phase gate
evaluation (`exit-criteria`), formal block requirements (`invariant` and
`chaos`), and the new removal-tracking enforcement (`absence`, FT-047).
Everything else — `scenario`, `benchmark`, and the team-specific names a
project really wants to use (`contract`, `migration`, `smoke`, `load`,
`end-to-end`, `property`) — is descriptive metadata. This feature partitions
the type vocabulary cleanly along that line: four reserved structural types
compiled into Product, two built-in descriptive types, and a
`[tc-types].custom` list in `product.toml` for everything else.

The full design lives in `docs/product-tc-types-spec.md`. This feature
implements that spec.

---

## Depends on

- **FT-011** — Context Bundle Format. The bundle ordering convention is
  revised to a six-position built-in sequence followed by alphabetical custom
  types. Forward-compatible refinement, not a break.
- **FT-018** — Validation and Graph Health. E006 is sharpened to enumerate
  both built-in and custom types in its hint; E017 surfaces through the
  existing E-code stream.
- **FT-029** — Gap Analysis. G002 (formal block on linked invariant/chaos)
  and G009 (absence TC linked) both query the type field. The structural /
  descriptive partition makes their lookup correct by construction.
- **FT-047** — Removal & Deprecation Tracking. `absence` is one of the four
  structural types catalogued by this feature; G009 is one of its mechanics.

---

## Scope of this feature

### In

1. **`[tc-types]` section in `product.toml`** with a single key `custom: [String]`.
   Default is empty. Validated at startup.
2. **Type validation refresh.** TC type values are valid iff in
   `{exit-criteria, invariant, chaos, absence, scenario, benchmark} ∪
   [tc-types].custom`. E006 wording updated to enumerate both sets.
3. **E017** at config-load time. Reserved structural names in
   `[tc-types].custom` cause Product to refuse to start, with the offending
   names listed.
4. **Bundle ordering** updated to the six-position built-in sequence followed
   by alphabetical custom-type sort. The ordering is implemented as a single
   comparator function used by `product context` and any other bundle
   emitter.
5. **AGENT.md / agent-context rendering.** The TC schema render groups types
   as structural / built-in descriptive / custom, with the custom list pulled
   from `product.toml`.
6. **Request validator update.** `product request validate` and
   `product request apply` reject TC artifacts whose `tc-type` is not in the
   valid set, with the same E006 hint as the graph check.
7. **Schema introspection.** `product_schema` includes the structural /
   descriptive distinction in its TC schema render.

### Out

- **Per-custom-type mechanics** (e.g. "make `smoke` skip in CI"). Custom
  types carry no Product mechanics. If a team wants a mechanic, the path is
  a configuration filter on top, not a structural property of the type.
- **Migration of existing TCs to new type names.** This feature does not
  rename existing TCs. Teams adopt custom types as they author new TCs.
- **A curated catalogue of "blessed" custom types** shipped with Product.
  The spec lists worked examples; teams pick what fits their project.
- **The `level:` field referenced in spec examples** (`level: integration`,
  `level: unit`, etc.) is not introduced or specified by this feature. It
  appears in spec illustrations as forward-compatible context only; if and
  when `level:` becomes a first-class field it is a separate ADR.
- **Renaming any of the four reserved structural types.** They are immutable
  identifiers in the codebase by design.

---

## Commands

No new CLI subcommands. The feature surfaces through:

- `product graph check` — E006 (unknown type), E017 (reserved name in
  custom), and the per-TC type validation pass.
- `product request {validate,apply}` — same type validation, same hints.
- `product context` (and `product agent-context`) — bundle ordering and
  schema rendering.
- Startup of any `product` command — config load runs E017.

---

## Implementation notes

- **`src/types.rs`** — define `TcType` as an enum with four `Structural`
  variants (`ExitCriteria`, `Invariant`, `Chaos`, `Absence`) and two
  `BuiltinDescriptive` variants (`Scenario`, `Benchmark`), plus a
  `Custom(String)` variant. Provide `is_structural()`, `is_descriptive()`,
  `bundle_sort_key()`, and `Display`/`FromStr` that round-trip the canonical
  spelling.
- **`src/config.rs`** — add `TcTypesConfig { custom: Vec<String> }`. Validate
  at load: reject reserved names with E017 (terminate the process before any
  command runs). Emit `&'static [&'static str]` for the four reserved names so
  the check is unambiguous.
- **`src/parser.rs`** — when parsing a TC, look up the `type` value against
  the union of built-in and configured custom types. Unknown → E006. The
  parser does not distinguish reserved-vs-built-in-descriptive at this
  layer; that distinction matters only for mechanics.
- **`src/graph.rs`** — `check_unknown_tc_types` rule iterates every TC and
  emits E006 for any unknown type. (Same code as parser; emitted via the
  graph-check stream for visibility.)
- **`src/gap.rs`** — no change. G002 and G009 already match against
  structural type names by exact string compare; this feature codifies that
  contract.
- **`src/context.rs`** — replace the existing TC sort with
  `bundle_sort_key()` from `types.rs`. The sort key returns
  `(category, position, name)` where category is `0` for built-in and `1`
  for custom, position is the six-element fixed sequence, and name is the
  type string for alphabetical custom ordering.
- **`src/agent_context.rs`** (or wherever `agent-init` / `agent-context`
  render the schema) — emit the structural / built-in descriptive / custom
  groups, sourcing the custom list from the loaded `TcTypesConfig`.
- **`src/request.rs`** — extend the request validator to call the same type
  lookup. Rejection is E006 with the same hint structure as parser/graph
  check.
- **`src/main.rs`** — config-load failure (E017) exits 1 before any command
  runs. The error path uses the same `ProductError` mapping as other
  config-time errors.
- **Tests.** Each TC is implemented as an integration or session test paired
  with `runner: cargo-test` and `runner-args: tc_NNN_snake_case` per
  CLAUDE.md. Add the runner config at the same time as the test.

---

## Acceptance criteria

A developer running on a clean repository can:

1. Configure `[tc-types].custom = ["contract"]` in `product.toml`, declare a
   TC with `type: contract`, and observe the graph builds with no E006
   (TC-605).
2. Configure `[tc-types].custom = ["regression"]`, declare a TC with
   `type: smoke` (not configured), and observe E006 in `product graph check`
   output with a hint listing both built-in and `["regression"]` custom types
   (TC-606).
3. Configure `[tc-types].custom = ["exit-criteria"]` and observe Product
   refuses to start with E017 naming the offending entry (TC-610).
   Confirm E017 fires before any subcommand executes (TC-611).
4. Observe an `exit-criteria` TC's `passing` status enables the phase gate
   for the next phase (TC-601).
5. Observe an `invariant` TC without a `⟦Γ:Invariants⟧` block triggers W004
   (TC-602). Same for `chaos` (TC-603).
6. Observe an ADR with non-empty `removes:` and a linked `absence` TC clears
   G009 (TC-604).
7. Run `product context FT-XXX` against a feature with TCs of every type
   category and observe the bundle order is exit-criteria → invariant →
   chaos → absence → scenario → benchmark → custom-alphabetical (TC-612).
   Add a custom type and observe it sorts last alphabetically (TC-613).
   Confirm two scenarios with no other state difference are not reordered
   when a custom type is added (covered by TC-613 invariant).
8. Observe a custom-type TC behaves identically to a `scenario` TC in all
   mechanics: it appears in bundles, runs via the configured runner, has
   status tracked in front-matter (TC-607). Observe it appears in the
   AGENT.md schema render (TC-608).
9. Submit `product request apply` with a TC of a configured custom type and
   observe success (TC-614). Submit one with an unknown type and observe E006
   with the configurable hint (TC-615).
10. Run `cargo test`, `cargo clippy -- -D warnings -D clippy::unwrap_used`,
    and `cargo build` and observe all pass.

See TC-616 (exit criteria) for the consolidated check-list.

---

## Follow-on work

- **`level:` field** for execution depth (unit / component / integration /
  system / acceptance). Referenced in spec examples; not implemented here.
  Separate ADR if pursued.
- **Mechanics-by-tag overlay.** A future feature could allow projects to map
  custom types to advisory metadata (e.g. "smoke runs on every deploy")
  without touching Product mechanics. Out of scope; user can wrap with
  external tooling today.
- **Lint for stale custom types.** A custom type declared in `product.toml`
  but used by zero TCs could be a W-class warning. Useful, not required;
  deferred.

---

## Functional Specification

### Inputs

- `product.toml` — `[tc-types]` section with a `custom: [String]` array; default is empty. Read at config-load time before any subcommand executes.
- TC YAML front-matter with a `type:` field — validated against the union of built-in types and configured custom types.
- `product graph check` — reads the graph and config; evaluates E006 (unknown type) and E017 (reserved name in custom list).
- `product request {validate,apply}` — validates TC `tc-type` fields in the request document against the same type set.
- `product context` / `product agent-context` — reads the graph, config, and TC type metadata for bundle ordering and schema rendering.
- `product_schema` — reads config for the custom type list; renders the grouped schema.

### Outputs

- **Config load (startup):** E017 if `[tc-types].custom` contains any reserved structural type name (`exit-criteria`, `invariant`, `chaos`, `absence`); Product refuses to start and lists the offending names.
- **`product graph check`:** E006 for any TC whose `type` value is not in the valid set (built-in + configured custom); the E006 hint enumerates both the built-in types and the configured custom types.
- **`product request validate` / `product request apply`:** E006 with the same hint for TC artifacts with an unknown type in the request document.
- **`product context FT-XXX`:** TCs ordered in the bundle by the six-position built-in sequence (`exit-criteria` → `invariant` → `chaos` → `absence` → `scenario` → `benchmark`) followed by alphabetical custom-type order.
- **`product agent-context` / `product_schema`:** schema grouped as structural (4 types with mechanics documented) / built-in descriptive (2 types) / custom (list sourced from `product.toml`). TC schema includes a cross-reference line after the `type:` field pointing at formal block requirements.
- **Mechanics (unchanged):** `exit-criteria` enables phase gate; `invariant` and `chaos` require formal blocks (W004 when absent); `absence` enforces `validates.features` empty + `validates.adrs` non-empty (FT-047). Custom types carry no Product mechanics.

### State

- **`product.toml`** — `[tc-types].custom` list is read-only from Product's perspective; the user edits it. Validated at startup; startup fails on reserved-name collision (E017).
- **TC front-matter `type:` field** — the `TcType` enum in `src/types.rs` carries four structural variants (`ExitCriteria`, `Invariant`, `Chaos`, `Absence`), two built-in descriptive variants (`Scenario`, `Benchmark`), and a `Custom(String)` variant. Custom values are validated against the configured list at parse time and graph-check time.
- No persistent state beyond front-matter and config. The type vocabulary is an in-memory union of built-ins and the `product.toml` custom list, rebuilt on every invocation.

### Behaviour

1. **Config-load validation.** On every Product invocation, before any subcommand executes, `product.toml` is parsed. If `[tc-types].custom` contains any of `["exit-criteria", "invariant", "chaos", "absence"]`, Product exits 1 with E017 listing the offending names. The subcommand does not run.
2. **Type validation at parse time.** When the parser reads a TC front-matter `type:` value, it looks it up in the union of built-in types and `[tc-types].custom`. Unknown values produce an E006 finding; the artifact is still included in the graph but marked with an error.
3. **Type validation in `product graph check`.** The `check_unknown_tc_types` rule iterates every TC and emits E006 for any type not in the valid set. The E006 message enumerates both built-in types and the configured custom list.
4. **Type validation in `product request validate` / `product request apply`.** The request validator runs the same type lookup on every TC artifact in the request. E006 with the same hint structure as graph check.
5. **Bundle ordering.** `product context` sorts TCs using a comparator keyed on `(category, position, name)` where `category=0` for built-ins (ordered by the six-position fixed sequence) and `category=1` for custom types (sorted alphabetically by type name). The sort key is implemented in `TcType::bundle_sort_key()` in `src/types.rs`.
6. **Schema rendering.** `product agent-context` / `product_schema` render the TC schema with three groups: structural types (with their mechanics documented), built-in descriptive types, and custom types (sourced from `product.toml`). A cross-reference line after `type:` in the TC schema points at formal block requirements for `invariant` / `chaos` / `exit-criteria`.
7. **Custom types carry no mechanics.** A TC with a custom type participates in bundles, runner execution, and status tracking identically to a `scenario` TC. No Product-level mechanic is triggered by custom type names.

### Invariants

- A reserved structural type name in `[tc-types].custom` always causes E017 at startup; Product never starts with a colliding custom type.
- The four structural type names are immutable identifiers in the codebase; they cannot be renamed without a separate ADR.
- Custom types carry no Product mechanics; all mechanics (phase gate, formal block requirement, absence validation) are triggered only by the four structural type names.
- Bundle ordering is deterministic: the six-position built-in sequence followed by alphabetical custom types, with a stable secondary sort by TC id within each type bucket.
- `product request apply` and `product graph check` use the same type validation logic (same code path or same function call); they produce identical E006 findings for the same invalid type value.

### Error handling

| Code | Condition |
|---|---|
| E017 | Reserved structural type name in `[tc-types].custom` — Product refuses to start; lists offending names |
| E006 | TC `type` value not in the valid set (built-in + configured custom); hint enumerates both sets |
| W004 | `invariant` or `chaos` TC without a `⟦Γ:Invariants⟧` or equivalent formal block in its body |

E017 fires before any subcommand executes, ensuring the invalid config never reaches runtime validation.

### Boundaries

- Per-custom-type mechanics (e.g. "make `smoke` skip in CI") are not supported; custom types carry no Product mechanics.
- Migration of existing TCs to new type names is not performed by this feature; teams adopt custom types for new TCs.
- A curated catalogue of "blessed" custom types is not shipped; the spec lists worked examples and teams choose what fits their project.
- The `level:` field (`level: integration`, `level: unit`, etc.) referenced in spec illustrations is not introduced or specified by this feature.
- Renaming any of the four reserved structural types is not in scope; they are immutable identifiers by design.

## Out of scope

- Per-custom-type mechanics: custom types are descriptive metadata only; no Product behaviour is attached to them.
- Migration of existing TCs to new type names: teams adopt custom types for new TCs at their own pace.
- A curated "blessed" custom-type catalogue shipped with Product: the spec provides worked examples; the choice of custom types is project-specific.
- The `level:` field referenced in spec examples: not implemented here; a separate ADR is needed if pursued.
- Renaming any of the four reserved structural types: they are immutable identifiers in the codebase.
