---
id: ADR-089
title: The domain model has a data side — reference data is What, production data is the oracle
status: accepted
features:
- FT-145
supersedes: []
superseded-by: []
domains:
- api
- data-model
scope: feature-specific
content-hash: sha256:d413664d4fb47fcbf82ae32367ff7bc88d73f583211fb011c738b3577ea0eba2
source-files:
- product-core/src/pf/model_data.rs
- product-core/src/pf/data_check.rs
- product-core/src/pf/rules_data.rs
- product-cli/src/commands/domain.rs
---

## Context

The `pf` What-graph captured the §3.1 **structure** side of the domain model —
entities, relations, invariants, bounded contexts — but not the **data** side.
§3.1 is explicit that a model trusted only from the structure side is asserted
top-down and never reconciled with ground truth, which is exactly why data
practitioners distrust developer domain models. The framework makes both sides
first-class and distinguishes two kinds of data by their relationship to the
model: constitutive **reference data** (part of the What) and **production
data** (the oracle the structure is validated against, §6.3).

## Decision

Add a data side to the `pf` graph as three node kinds and a conformance engine:

1. **`ReferenceSet`** — constitutive reference data. It declares the `concept`
   it is `reference_data_for` and its closed set of `values`. It lives in the
   What because the model and behaviour reference it.
2. **`DataShape`** — the structure made machine-checkable. It targets an entity
   (`conforms_to_shape`) and declares `required` fields and `enums` constraints
   (membership in a declared reference set) — the SHACL-property side of §3.1.
3. **`ProductionDataset`** — the oracle pointer. It names the `shape` it
   conforms to and a `source` of populated records; it is not specification.

`product domain data <dataset>` runs **data conformance**: it reads the source,
validates each record against the shape, and reports the **data-divergence
rate** — the fraction of records that fail — exiting 1 on any divergence. The
verdict is deliberately bidirectional: a failure means either the data is wrong
*or* the spec is stale, and the message says so. The pure engine lives in
`pf::data_check`; the data-side rules (presence + cross-reference) in
`pf::rules_data`; the model in `pf::model_data`; the CLI is a thin adapter
(`domain data`), per the slice + adapter pattern (PAT-001).

## Rationale

- **Both sides first-class.** Modelling reference data and a production oracle
  reconciles the structure with ground truth — the data practitioner's lens
  given teeth.
- **The two motivating defects are caught.** A field null/absent in production
  rows (`missing-required`) and an enum value the schema never declared
  (`not-in-reference-set`) are exactly the §3.1 examples the structure side
  cannot see.
- **Failures read both ways.** Data conformance is the first check whose failure
  can indict the specification, so the verdict never presumes the data is wrong.
- **Authoring stays ergonomic.** Cross-references are checked by `validate`
  (whole-graph), not in-loop, so a shape or dataset can be authored before the
  concept it points at — matching the reification model's posture.

## Rejected alternatives

- **Full RDF + SHACL shape expression up front.** §3.1 names RDF+SHACL as the
  reference but admits an equivalent; required-field + reference-set membership
  cover the motivating defects with a CLI-authorable surface. Richer SHACL is
  future work.
- **Treating production data as specification.** Rejected by §3.1's
  constitutive/populated test: production rows are an oracle, never the What.
- **A separate top-level `data` command.** Folded under `domain` to keep the
  What-graph surface together and avoid a new top-level parity obligation.

## Test coverage

- TC-1021 — author the structure/data split; validate + export.
- TC-1022 — clean data is conformant at a 0.0% divergence rate.
- TC-1023 — divergence caught per record, rate reported, verdict reads both ways.
- TC-1024 — `validate` catches a dangling data cross-reference.
