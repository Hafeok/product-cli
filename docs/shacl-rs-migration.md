# Adopting `shacl-rs`: native SHACL validation for the What/How graphs

> Status: proposal / implementation guide.
> Audience: anyone (human or agent) implementing the migration.
> Scope: `product-core/src/pf/` validation; the `Violation` surface stays, its internals change.

## Why

The framework's conformance rules are **already authored as SHACL** — `schema/shapes/shapes.shacl.ttl`
(What) and `schema/shapes/how.shacl.ttl` (How). Today we enforce them three ways:

1. **The shapes themselves** (`.ttl`) — the source of truth.
2. **`spec/.../validate.py`** — the reference validator, using `pyshacl` + `rdflib` (Python).
3. **A hand-rolled Rust re-implementation** that *mirrors* the shapes so the binary needs no Python:
   `pf/validate.rs` (presence/cardinality), `pf/rules_what.rs` + `pf/rules_how.rs` + `pf/sparql_rules.rs`
   (the cross-reference rules as embedded SPARQL run over an oxigraph `Store`), plus the per-artifact
   checkers `pf/cell_validate.rs`, `pf/how_validate.rs`, `pf/work_unit_validate.rs`.

That is **triple maintenance with built-in drift**: every shape edit must be mirrored in Rust (and
ideally in the Python reference). Each mirror file's own doc comment admits it — *"mirrors
`schema/shapes/shapes.shacl.ttl`"*.

[`shacl-rs`](../../rust-shacl) is a native Rust SHACL 1.2 engine (138/141 of the W3C 1.2 core suite,
plus SHACL-SPARQL §8.1). Adopting it lets us validate the existing `.ttl` shapes **directly in Rust**,
deleting the hand-rolled mirrors and collapsing to one source of truth — no Python, no drift.

## What we already have that makes this easy

- `pf::turtle::to_turtle(graph, product) -> String` already projects the What graph to Turtle.
  (`how_turtle` / `decider_turtle` do the same for the other layers.) **This is the SHACL data graph.**
- `pf::sparql_rules::run_rules` already loads that Turtle into an `oxigraph::Store` and runs SPARQL —
  i.e. we already have the oxigraph backend `shacl-rs` validates over.
- `pf::validate::Violation { focus, path, message, severity }` is the one diagnostic type every
  validator returns. It maps 1:1 onto a `shacl-rs` result. **Keep this type and the `Vec<Violation>`
  function signatures unchanged** so callers (`conformance/check.rs`, `product-mcp`
  `framework_read_handlers`, and the `how`/`cell`/`work_unit`/`archetype`/`decider` CLI commands)
  don't change.

## The dependency

`shacl-rs` is consumed as an external crate (like `decision-cli` consumes `product-core`) — its own
files are **not** subject to our 400-line / no-`and` / `deny(unwrap_used)` conventions. Its library
code *is* already unwrap/expect/panic-free, so it won't trip our lints transitively.

```toml
# product-core/Cargo.toml
[dependencies]
shacl-oxigraph = { git = "https://github.com/Hafeok/rust-shacl", tag = "v0.1.0" }
# pulls in shacl-core / shacl-model / shacl-sparql transitively.
# shacl-core has NO oxigraph dependency; shacl-oxigraph is the only crate that does
# (we already depend on oxigraph — ADR-008).
```

The `v0.1.0` tag is the first release (all four crates are versioned together at `0.1.0`). Embed the
shapes in the binary (we already embed the SPARQL rule strings), so there's no
runtime file dependency:

```rust
const WHAT_SHAPES: &str = include_str!("../../schema/shapes/shapes.shacl.ttl");
const HOW_SHAPES:  &str = include_str!("../../schema/shapes/how.shacl.ttl");
```

## The one call

`shacl-rs` exposes a high-level entry point that runs **Core (§7) and `sh:sparql` (§8.1)** in one pass
and returns one report with `sh:message`s populated:

```rust
use shacl_oxigraph::validate_turtle;

let ttl = pf::turtle::to_turtle(graph, product);     // existing projection
let report = shacl_oxigraph::validate_turtle(WHAT_SHAPES, &ttl)
    .map_err(ProductError::from)?;                    // Result<ValidationReport, String>
```

If you'd rather not re-parse the shapes each call, parse once and reuse, and validate over a `Store`
you already built:

```rust
use shacl_oxigraph::{ingest::parse_shapes, store::OxiStore, validate_store};

let shapes = parse_shapes(WHAT_SHAPES).map_err(ProductError::from)?;   // cache this
let store  = OxiStore::new(existing_oxigraph_store.clone());           // clone is cheap (shared storage)
let report = validate_store(&store, &shapes);
```

## The mapping: `ValidationResult` → `pf::validate::Violation`

```rust
use shacl_core::ValidationResult;
use shacl_model::shape::Severity;

fn to_violation(r: &ValidationResult) -> pf::validate::Violation {
    pf::validate::Violation {
        // SHACL focus node → our `focus`. Strip to a local id if our Violation uses bare ids.
        focus:   local_id(&r.focus_node.to_string()),
        // sh:resultPath is the SPARQL string "<iri>"; reduce to the local name we already use.
        path:    r.result_path.as_deref().map(path_local_name).unwrap_or_default(),
        // sh:message → our framework-rule string ("§3.1 An entity must …").
        message: r.messages.first().cloned().unwrap_or_default(),
        severity: match r.severity {
            Severity::Violation => "violation",
            Severity::Warning   => "warning",
            Severity::Info      => "info",
            Severity::Debug | Severity::Trace => "info",
        }.to_string(),
    }
}

// All our framework shapes are blocking, so:
fn to_violations(report: &shacl_core::ValidationReport) -> Vec<pf::validate::Violation> {
    report.results.iter().map(to_violation).collect()
}
```

`r.source_shape` (`ShapeId`) and `r.source_constraint_component` (`NamedNode`) are also available if a
richer report is ever wanted. `report.conforms()` gives the boolean for the `{ ok, … }` MCP shape.

## Migration plan (keep the surface, swap the internals)

Do it one validator at a time; each step is independently shippable and verified by the **existing**
`*_validate_tests.rs` suites (they assert on `Violation`s, which don't change).

1. **Add the dependency + embed the shapes** (above). Land a thin internal helper:
   `pf::shacl::validate_ttl(shapes: &str, ttl: &str) -> Vec<Violation>` that calls `validate_turtle`
   and maps the report. One place owns the `shacl-rs` types.

2. **What graph** — reimplement `pf::validate::validate_graph` (and `validate_node`, by filtering the
   report to one focus) on top of `pf::shacl::validate_ttl(WHAT_SHAPES, to_turtle(graph))`. Run
   `cargo test -p product-core pf::validate` until green. **Delete** `pf/rules_what.rs` once the
   cross-reference rules pass via SHACL.

3. **How layer** — reimplement `pf::how_validate::validate_how` on `HOW_SHAPES` + `how_turtle`.
   The `sh:sparql` trace-truth rule (`§5/§4.1`) runs natively now. **Delete** `pf/rules_how.rs`.

4. **Cells / work units / decider** — point `cell_validate`, `work_unit_validate`, and the
   `decider_conform` path at the same helper with the relevant shapes/projection. **Delete** the
   bespoke presence/cardinality logic in `pf/validate.rs` that the shapes already cover.

5. **Retire the SPARQL-rule scaffolding** — once steps 2–4 are green, `pf/sparql_rules.rs`,
   `pf/rules_what.rs`, `pf/rules_how.rs`, and `pf/rules_decider.rs` have no callers. Delete them.

6. **Spec reference** — `spec/.../validate.py` + `pyshacl`/`rdflib` become an optional external
   cross-check, no longer mirrored by the binary. Keep it for differential testing or drop it.

### Files this touches

| Keep (surface) | Reimplement internals | Delete after migration |
|---|---|---|
| `pf/validate.rs::Violation` | `pf/validate.rs::validate_graph` / `validate_node` | `pf/rules_what.rs` |
| `pf/how_validate.rs::validate_how` (sig) | its body | `pf/rules_how.rs` |
| `pf/cell_validate.rs` / `work_unit_validate.rs` (sigs) | their bodies | `pf/rules_decider.rs` |
| `pf/turtle.rs` / `how_turtle.rs` (projections) | — | `pf/sparql_rules.rs` |
| `schema/shapes/*.ttl` (now the only source of truth) | — | (optionally) `spec/.../validate.py` |

## Verification

- The existing `pf/*_validate_tests.rs` are the acceptance gate — they assert on `Violation`s and must
  stay green through every step (that's the point of keeping the surface).
- Cross-check a few graphs against `spec/.../validate.py` (pyshacl) during the cut-over to confirm the
  native engine agrees, then retire the comparison.
- `shacl-rs` itself is conformance-tested against the W3C SHACL 1.2 core suite (138/141) and ships a
  fixture test that validates **our** `schema/shapes/*.ttl` (messages + the trace-truth `sh:sparql`
  rule) — see `rust-shacl/shacl-oxigraph/tests/product_framework.rs`.

## Caveats (none block our current shapes)

- **`sh:prefixes` isn't collected** — `sh:select` queries must use full IRIs or carry their own
  `PREFIX` lines. Our `how.shacl.ttl` already uses full IRIs, so this is a non-issue today; keep it in
  mind if future shapes use prefixed names in `sh:select`.
- **SPARQL-based *components* (§8.2)** (custom `sh:ConstraintComponent` with `sh:validator`) and
  **pre-binding restriction checks** (REQ-SPQ-15) are not implemented. We don't use them.
- **Complex-path `sh:resultPath`** (sequence/alternative/`*`) isn't serialized to RDF; the in-memory
  `result_path` SPARQL string is always present, which is all the `Violation` mapping needs.

## Recommended graph artifacts

Because this is an architectural change, record it the Product way before implementing:

- An **ADR** — "Adopt `shacl-rs` for native SHACL conformance; retire the hand-rolled Rust mirrors and
  the pyshacl reference path." Supersedes or amends the decisions behind `pf/rules_*` and `sparql_rules`.
- A **feature** (FT) linked to that ADR, with **TCs** that are exactly the existing
  `pf/*_validate_tests.rs` (re-pointed runners) plus one new TC asserting a `sh:message` reaches a
  `Violation.message`.

`product author adr` then `product author feature --feature <id>` will scaffold these against the
existing graph so the decision and its tests are first-class.
