# Worked example: product-cli describes and verifies its own How

This is a dogfood of the Product-Framework toolchain (FT-109–FT-117) on
`product-cli` itself: the repo's *What* graph, an *archetype* (How + layout +
cells) describing how `product-cli` is actually built, and a dispatch that
produces frozen SPMC work units — every layer validated by the same tools.

The archetype lives at [`.product/archetypes/product-cli/`](../../.product/archetypes/product-cli/).

## The four-layer verified chain

```console
$ product domain validate                 # What — §3 (Described)
conformant — 152 node(s), 0 violations

$ product archetype validate product-cli  # How + layout + cells — §4/§5 (Realised)
conformant — archetype 'product-cli': how present, layout present, 1 cell(s) [domain: cross-checked]

$ product cell dispatch \
    --file .product/archetypes/product-cli/cells/implement-slice.yaml \
    --bind concept=e-entity --bind operations=validate,show \
    --out .product/archetypes/product-cli/work-units
Dispatched core-slice-e-entity -> …/core-slice-e-entity.yaml
Dispatched cli-adapter-e-entity -> …/cli-adapter-e-entity.yaml
Dispatched integration-tests-e-entity -> …/integration-tests-e-entity.yaml

$ product work-unit validate --file …/integration-tests-e-entity.yaml
conformant — work unit 'integration-tests-e-entity' produces … [domain: cross-checked]

$ product conformance check                # the FT/ADR/TC graph — Two Pillars
Verdict: conforms to Level 3 (spec-driven) — checkable subset
```

## What it captures

The archetype encodes `product-cli`'s real architecture as a conformant How:

- **Top decisions** — *slice + adapter* (pure `pf::` domain slices in
  product-core, thin CLI adapters in product-cli) and *native checkers*
  (conformance checkers mirror the vendored SHACL/JSON schemas, cross-checked
  against `pyshacl`).
- **Principles** — `slice-cohesion`, `pure-core`, `conformance-fidelity`,
  `zero-unwrap`; each enforced by a named verification.
- **Patterns** — `slice-plus-adapter` (applied by the `implement-slice` cell),
  `native-checker`.
- **Contracts** — an application contract (Rust, `core`→`cli` layering,
  statements: no `unwrap()`, files under 400 lines) and an infrastructure
  contract (cargo-dist release), with the layout model placing slices in
  `product-core/src/pf/**` and adapters in `product-cli/src/commands/**`.

## The realisation step

`implement-slice` is a task type whose `concept` slot binds to a real entity in
the What graph. Dispatch resolves the cell's `domain:concept` input to the
bound entity (`domain:e-entity`), freezes and content-hashes the context, and
emits one SPMC work unit per cell (core slice → CLI adapter → integration
tests). Binding to a value that is **not** an entity in the What graph is
rejected — a cell can only ever be realised against concepts that exist.

> The dispatched work units under `work-units/` are generated output
> (gitignored); regenerate them with the `cell dispatch` command above.
