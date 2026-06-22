---
id: FT-145
title: Domain model structure/data split and data conformance
phase: 7
status: complete
depends-on:
- FT-109
adrs:
- ADR-089
tests:
- TC-1021
- TC-1022
- TC-1023
- TC-1024
- TC-1025
- TC-1026
domains:
- api
- data-model
domains-acknowledged:
  ADR-041: Additive — adds the ReferenceSet/DataShape/ProductionDataset node kinds and the reference_data_for/conforms_to_shape data-side edges; nothing existing is removed or deprecated, so no absence TC is required.
  ADR-047: The functional specification lives in this feature's body, not a separate artifact.
  ADR-048: Reads the captured graph and a declared dataset source file only; no other side effects.
  ADR-049: Not a context-bundle/template command; no template surface changes.
  ADR-050: PAT-001 (slice + adapter) governs the `pf` slice (`model_data`, `rules_data`, `data_check`) + the `domain data` CLI adapter; no new implementation pattern is introduced.
  ADR-042: TCs use the reserved `scenario` type only; no new TC type is introduced.
  ADR-018: Scenario TCs drive the binary through assert_cmd; `pf::data_check` and `pf::rules_data` carry unit tests. No property or session dimension for the data model and its checks.
  ADR-043: The data-side model and the conformance engine live in the pure `pf` slice; the CLI is a thin adapter.
  ADR-040: Data conformance is a §6.3 verification kind that reads against the domain model as oracle; it composes with the existing What-side rules and does not touch the verify pipeline.
  ADR-051: Every TC declares `observes:` and asserts on those surfaces (graph, exit-code, stdout).
patterns:
- PAT-001
---

## Description

§3.1 of the framework splits the domain model into two sides: **structure**
(the entity types, relations, invariants — already captured) and **data**.
This feature adds the data side. Some instance data is **reference data** —
constitutive of the What (the valid shipping methods, the tax categories the
behaviour depends on). The live entities are **production data**, which is *not*
specification but serves as the **oracle** the structure is validated against:
"does real data conform to the declared shapes?" is **data conformance** (§6.3),
and it catches model defects nothing else will (a field null in most rows, an
enum value the schema never declared).

## Functional Specification

### Inputs

- The captured graph for a product (its What — entities, contexts;
  `--product` to override the default).
- Three new data-side node kinds, authored through the generic `domain new`
  path: `reference-set`, `data-shape`, `production-dataset`.
- A production dataset's **source**: a JSON file of record objects.

### Behaviour

- **Reference data is part of the What.** A `ReferenceSet` declares the
  `concept` it is constitutive of (`reference_data_for`) and its closed set of
  `values`. It is versioned and governed like the rest of the What.
- **A shape is the structure made checkable.** A `DataShape` targets an entity
  (`conforms_to_shape`) and declares three kinds of field constraint — the
  SHACL-property side of §3.1: `required` (present + non-null), `enums` (a
  field's value must be a member of a declared reference set), and `types` (a
  field's value must be of a declared datatype: string / integer / number /
  boolean / date).
- **A production dataset is the oracle.** A `ProductionDataset` names the
  `shape` it conforms to and a `source` of populated records; it is not
  specification.
- **Data conformance.** `product domain data <dataset>` (omit the id to check
  every declared dataset) reads the source, validates each record against the
  shape, and reports the **data-divergence rate** — the fraction of records that
  fail. A missing required field, an undeclared enum value, and a wrong datatype
  are each caught, per record. Exit 1 on any divergence.
- **The divergence rate is tracked over time.** Each run is recorded to a
  per-product history, and the report surfaces the **trend** (first run / rising
  / falling / stable) against the previous run — the §3.1 "spec staleness
  becomes measurable" signal, made visible as it happens (§13.3). `--no-record`
  runs the check without writing to the history (for CI/read-only use).
- **The verdict reads both ways.** A divergence is genuinely ambiguous: the data
  may be wrong, or the spec may be stale. The message says so — fix the data, or
  (if the model no longer describes the system) fix the shape.
- **Cross-references are checked.** `domain validate` flags a reference set whose
  concept does not exist, a shape targeting an unknown entity or naming an
  unknown reference set, and a dataset whose shape is undeclared.

### Error handling

- A reference set with no concept or no values, a shape with no target, or a
  dataset with no shape/source is rejected at author time with a clear message.
- A dataset source that is missing, not valid JSON, or not a JSON array is a
  clear error naming the source path.

## Out of scope

- **Scheduled, on-write assertion in production** ("asserted continuously",
  §3.1) — this feature records the divergence rate on each on-demand run and
  reports the trend; wiring it to a scheduler or a write hook is future work.
- **Full RDF + SHACL shape expression** — the shape language here covers
  required-field presence, reference-set membership, and datatype, the §3.1
  motivating defects; richer SHACL constraints (regex, ranges, cardinality) are
  future work.

## Acceptance

- TC-1021 — author the structure/data split (reference data, shape, dataset);
  the graph validates and exports on the data-side predicates.
- TC-1022 — clean production data is conformant with a 0.0% divergence rate.
- TC-1023 — divergent data is caught per record (missing-required +
  not-in-reference-set), the divergence rate is reported, and the verdict reads
  both ways.
- TC-1024 — `domain validate` catches a dangling data cross-reference.
- TC-1025 — a datatype constraint catches type drift (a string where an integer
  is declared).
- TC-1026 — the divergence-rate trend is surfaced across runs (first → rising),
  and `--no-record` leaves the history untouched.
