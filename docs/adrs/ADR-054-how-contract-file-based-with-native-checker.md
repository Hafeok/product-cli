---
id: ADR-054
title: How contracts are file-based YAML with a native checker mirroring how.shacl.ttl
status: accepted
features:
- FT-111
supersedes: []
superseded-by: []
domains:
- api
- data-model
scope: feature-specific
content-hash: sha256:58a241511f3d6ea3fa0df511c32d91cc253b0a09ae9bba68742e9edbc8d36086
source-files:
- product-core/src/pf/how.rs
- product-core/src/pf/how_validate.rs
- product-core/src/pf/how_turtle.rs
- product-cli/src/commands/how.rs
---

## Context

The framework's §4 "How" layer (the Why cascade — top decisions → principles →
patterns — the application + infrastructure contracts, the layout reference,
and interface contracts) ships as a schema package: `how-contract.schema.json`
(a JSON-Schema authoring surface) and `how.shacl.ttl` (the RDF conformance
shapes, including the crown **trace-truth** SPARQL rule: every principle a work
unit applies must be enforced by a verification).

Unlike the What (FT-109/110), which is an RDF graph captured through an MCP
session, the How's authoring surface is explicitly a **file** ("a toolchain
typically authors the YAML and projects it into the graph"). There is no How
`tools.json` — no MCP session is specified.

## Decision

The How layer is implemented as a **file-based contract** with a **native
Rust conformance checker**, mirroring the What's `pf` approach:

1. **Typed model + YAML** (`pf::how`): `HowContract` mirrors
   `how-contract.schema.json`; loaded from / written to YAML
   (`.product/how-contract.yaml` by default).
2. **Native checker** (`pf::how_validate`) mirrors `how.shacl.ttl` — decision
   rationale, pattern-realizes, infra-conformsTo, interface-derivedFrom,
   principle earn-their-place, and the **crown trace-truth rule** computed over
   the file (a principle is "applied" when a pattern realising it is
   `applied_by` a cell/task-type; an applied principle must be `enforced_by`).
3. **Turtle projection** (`pf::how_turtle`) emits the §4 nodes plus the
   Verification (from `enforced_by`) and Work Unit (from `applied_by`) nodes
   the cross-node shapes need — so the projection validates against the real
   `how.shacl.ttl` + `shapes.shacl.ttl`, cross-checked with `pyshacl`.
4. **CLI** (`product how`): `validate` (exit ≠0 on blocking violations),
   `show`, `list`, `export`, `init`. CRUD-by-flag is not offered: the How is a
   nested single document edited as a file, not a flat node graph.

Soft cascade pointers (`licenses`, `realized_by`) that are dangling are
**warnings**, not violations, and the projection omits dangling soft edges so
the native verdict and the `pyshacl` verdict agree on conformance.

## Rationale

- A native checker keeps the zero-Python-runtime, in-process, typed-violation
  properties established for the What; the vendored `how.shacl.ttl` remains the
  external source of truth, cross-checked in development.
- File-based YAML matches the framework's stated authoring model for the How
  and is diff-friendly, reviewable, and tool-authorable.
- Projecting to Turtle lets the How join the one connected graph and be
  verified by the same SHACL the framework ships — including the SPARQL crown
  rule, which a native check alone could drift from.

## Rejected alternatives

- **An MCP `author how` session like the What.** Rejected: the spec ships no
  How `tools.json`; the How's authoring surface is a file, and a nested
  contract is awkward to build through atomic node-at-a-time tool calls.
- **Per-item CRUD CLI (`how new principle …`).** Rejected: the How is one
  nested document; editing the YAML is clearer than 6 × N flag-driven verbs.
- **Shell out to `pyshacl`.** Rejected, as for the What (ADR-053): a Python
  runtime dependency in a Rust CLI, per-call latency.

## Test coverage

- TC-910 — validate passes on a conformant contract (with a soft warning).
- TC-911 — validate flags a broken trace (the crown rule).
- TC-912 — show + list render the contract.
- TC-913 — export emits Turtle with synthesised verification/work-unit links.
- TC-914 — init scaffolds a contract that validates; refuses to clobber.
- TC-915 — validate without a contract file is a clear error.
