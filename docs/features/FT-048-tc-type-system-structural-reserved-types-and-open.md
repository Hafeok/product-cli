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
  ADR-018: Predates the 2026-04-22 scope promotion of ADR-018 to cross-cutting. Test coverage reflects the property/session/benchmark strategy as it existed when this feature shipped; not retroactively reclassified.
  ADR-043: Predates ADR-043; feature command adapters were written before the slice+adapter pattern was formalised and are not retroactively refactored.
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
