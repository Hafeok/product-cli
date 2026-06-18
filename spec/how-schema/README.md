# The Product Framework — Schemas

Machine-readable schemas for every artifact the [Product Framework](../SPEC.md) defines.
The framework's reference encoding is **RDF** for the connected graph (the What, the How,
the derivation links) and **JSON/YAML** for the file-based operational artifacts (the layout
model, task-type definitions, work units, delivery). This package provides both, plus a
validator and conformant examples.

## What's here

```
schema/
  ontology/
    product-framework.ttl          # the vocabulary: every artifact class + the derivation-contract links
  shapes/
    shapes.shacl.ttl               # SHACL — the What, work units, verification, delivery
    how.shacl.ttl                  # SHACL — the complete How layer (incl. the trace-truth rule)
  json/
    layout-model.schema.json       # §4.3 repository layout model (globs, allowlist, the two guards)
    how-contract.schema.json       # §4 an archetype's How: decisions/principles/patterns + the two contracts + interfaces
    task-type-definition.schema.json  # §5 the dual-read task-type definition
    work-unit.schema.json          # §5 the SPMC work-unit manifest
    delivery.schema.json           # §7 features & releases, with the done predicates
  examples/
    todo-product.ttl               # a tiny conformant product graph (the What/How/Delivery as RDF)
    *.example.yaml                 # one conformant instance per JSON schema
  validate.py                      # runs SHACL + JSON Schema validation over any artifact
```

## Artifact → framework section → schema

| Artifact | Framework § | Schema | Encoding |
|---|---|---|---|
| Bounded context, entity, relation, value object, invariant, context mapping | §3.1 Domain model | `ontology` + `shapes` | RDF/SHACL |
| Event, command, read model, wireframe step, flow | §3.2 Event model | `ontology` + `shapes` | RDF/SHACL |
| Top decision, principle, pattern | §4.1 The Why | `ontology` + `how.shacl` + `how-contract.schema.json` | RDF/SHACL + YAML |
| Application & infrastructure contracts | §4.2 | `ontology` + `how.shacl` + `how-contract.schema.json` | RDF/SHACL + YAML |
| **Repository layout model** | §4.3 | `layout-model.schema.json` | YAML/JSON |
| Interface contract (OpenAPI/AsyncAPI/…) | §4.4 | *use the standard's own schema* (+ recorded in `how-contract`) | — |
| **Work unit (SPMC)** | §5 | `work-unit.schema.json` | YAML/JSON |
| **Task-type definition** (dual-read) | §5 | `task-type-definition.schema.json` | YAML/JSON |
| Verification + verdict | §6 | `ontology` + `shapes` | RDF/SHACL |
| **Feature & release** | §7 | `delivery.schema.json` | YAML/JSON |
| The derivation contract (`derived_from`, `applies`, `enforces`, …) | §9 | `ontology` | RDF |

Interface contracts deliberately have **no schema here** — they use the industry standard's
own schema (OpenAPI, AsyncAPI, Protobuf), per framework §4.4. Do not reinvent them.

## Two encodings, one graph

The RDF artifacts (domain, event, why, contracts, verifications, delivery, and the links
between them) form **one connected graph** — that is what makes "describe this system" a
query and impact analysis a traversal. The JSON/YAML artifacts are the **authoring surface**
for the parts that live as files in a repo (a layout model, a task-type definition, a work
unit). A toolchain typically authors the YAML and projects it into the graph; the two are not
in competition.

## The conformance checker is real, not vacuous

`shapes.shacl.ttl` enforces the framework's load-bearing rules, each with a message naming the
section it comes from — so a validation report reads as a conformance report. For example:

- every **event** must `changes` a real entity (§3.2 — the load-bearing relation);
- every **top decision** must carry rationale (§4.1);
- a **principle** must be applied by a work unit *or* enforced by a verification (§4.1 earn-their-place);
- a **pattern** must `realizes` a principle (§4.1);
- an **infrastructure contract** must `conformsTo` the application contract (§4.2);
- every **verification** must `enforces` something (§6.1 — name what it protects);
- **the trace must be true** (§5/§4.1): every principle a work unit `applies` must be `enforces`d by some verification — a cross-node SPARQL rule in `how.shacl.ttl`, the crown rule of the How.

A malformed graph (an orphan event, a why-less decision, an applied-but-unenforced principle) fails with those messages.

## What the framework keeps open vs. closed

These schemas describe the **shapes** — they ship no proprietary content. In particular,
`shapes.shacl.ttl` validates that verifications *exist and are well-formed*; it does not (and
cannot) contain the verification **content** — the actual checks are the adopter's, per the
framework's open/closed line. Likewise the examples are a toy to-do domain, not any real
archetype.

## Validating

```bash
pip install rdflib pyshacl jsonschema pyyaml

# validate all bundled examples
python schema/validate.py

# SHACL-validate your own product graph
python schema/validate.py path/to/your-product.ttl

# validate a file-based artifact
python schema/validate.py path/to/layout.yaml --as layout
python schema/validate.py path/to/how.yaml     --as how-contract
python schema/validate.py path/to/task.yaml   --as task-type
python schema/validate.py path/to/unit.yaml   --as work-unit
python schema/validate.py path/to/plan.yaml   --as delivery
```

## Conformance levels

An instance claims the highest cumulative level it satisfies (framework §8.1):
**Described** (a conformant What graph) → **Realised** (a conformant How + work units) →
**Verified** (verifications of all required kinds, meeting the coherence bar) →
**Delivered** (features/releases as graph partitions with computed `done`).

## Versioning

Schemas are versioned with the specification. Breaking changes follow the framework's
deprecation policy so existing conformant instances are never silently invalidated.
