---
id: ADR-081
title: Accessibility is ingested WCAG 2.2 criteria with machine, assisted, and manual verification
status: accepted
features:
- FT-137
supersedes: []
superseded-by: []
domains:
- api
- data-model
scope: feature-specific
content-hash: sha256:eb3c181c0f1a63faf17feb86a003ae2ae4f4f2774d02a674858cbd6de1d21a52
source-files:
- product-core/src/pf/ids.rs
- product-core/src/pf/model.rs
- product-core/src/pf/wcag.rs
- product-core/src/pf/turtle.rs
- product-core/src/pf/rules_ui.rs
---

## Context

§3.2.3 of the framework specifies a UI step's **accessibility obligations** not
as prose about what "must be perceivable" — an unverifiable wish — but as a set
of **WCAG 2.2 success criteria** referenced as *ingested entities*. The standard
itself has a graph shape (principle → guideline → success-criterion → level,
A/AA/AAA), and the framework treats each criterion as a first-class node with a
known verification type, so that "accessible" stops being a judgement call and
becomes a checkable obligation with a basis.

Two further facts make this more than a list. First, every criterion carries a
**verification type**: some are *machine* (a deterministic gate can decide them),
some *assisted*, some *manual* (a human must evaluate them). The framework cannot
honestly claim a machine pass for a criterion no machine can decide — so the type
is part of the criterion, and the discharge mechanism differs by type. Second,
obligations are mostly **inherited from the AIOs a step uses** (ADR-078): a
`text-entry` carries its labelling criteria, an image-bearing `display-value`
carries 1.1.1 Non-text Content. The `pf/` engine has no representation of any of
this today; the UI step's accessibility is currently unmodelled.

## Decision

Ingest WCAG 2.2 as graph entities and make a step's accessibility a computed,
typed obligation:

1. **New node kinds.** Add `WcagCriterion` and `Attestation` to `NodeKind`
   (`pf/ids.rs`/`ALL_KINDS`) and the `pf:` ontology. A `WcagCriterion` carries
   its standard identifier (e.g. `1.1.1`), its conformance **level** (A/AA/AAA),
   and its **verification type** — `machine`, `assisted`, or `manual`. The
   criterion hierarchy (principle → guideline → criterion) is ingested as
   reference data (`pf/wcag.rs`).

2. **New predicates.** `must_satisfy` — an `Aio` *or* a `UiStep` must satisfy a
   given criterion. `attests` — a **dated, attributed** record that a non-machine
   criterion was evaluated and met for a step. Attestations enter through a
   frozen boundary and are immutable once recorded, consistent with content-hash
   immutability ([[project-graph-conformance]], ADR-034).

3. **Inheritance + extension, computed.** The seed AIO set (ADR-078) declares the
   criteria each AIO `must_satisfy`. A `UiStep`'s full obligation is the
   **computed union** of the criteria of every AIO it references, plus any
   screen-specific criteria the step adds directly. Adding or removing an AIO
   adds or removes its criteria automatically — there is **no hand-maintained
   per-screen checklist** to drift.

4. **The verdict reports level and basis.** Discharge is by verification type:
   *machine* criteria are deterministic gates; *assisted* and *manual* criteria
   are discharged by a recorded attestation. The accessibility verdict reports a
   **conformance level and its basis** (which criteria, discharged how), never a
   bare pass. The machine-gate and attestation-coverage rules live with the other
   What-side UI rules (`pf/rules_ui.rs`); their composition into the full screen
   verdict is the seam verification (ADR-084).

## Rationale

- A criterion-as-entity with a known verification type is checkable where prose
  is not: it has an identity, a level, and a defined way to be discharged.
- Computing the obligation as the union over a step's AIOs makes accessibility
  *inherited*, so it is discharged once at the AIO/design-system level and only
  the genuinely screen-specific criteria need step-level work — the same leverage
  the AIO layer gives the rest of the UI.
- Splitting discharge by verification type keeps the framework honest: it never
  claims a machine pass for a criterion only a human can decide; the attestation
  is the auditable record that the human check happened, dated and attributed.
- Reporting level + basis rather than a bare pass makes the verdict reviewable
  and traceable, matching how every other verification names what it protects.

## Rejected alternatives

- **Free-text "accessibility intent" on the step.** Rejected: prose has no known
  verification type and cannot be gated — it is the unverifiable wish §3.2.3
  replaces. A criterion is a checkable entity; a sentence is not.
- **A hand-maintained per-screen list of criteria.** Rejected: it drifts the
  moment an AIO is added or removed. The obligation must be the *computed* union
  over the step's AIOs, not a list someone remembers to update.
- **One undifferentiated "is accessible" boolean.** Rejected: it erases the
  level and the basis, and pretends a uniform check exists where machine,
  assisted, and manual criteria demand different discharge.

## Test coverage

- TC-1003 — a step inherits its AIOs' criteria as a computed union; adding/removing
  an AIO changes the union with no hand-maintained list.
- TC-1004 — an unsatisfied *machine* criterion fails the gate; the verdict reports
  level and basis, not a bare pass.
- TC-1005 — an *assisted*/*manual* criterion is undischarged without an
  attestation and discharged by a dated, attributed one.
- `pf::wcag` + `pf::rules_ui` unit tests cover ingestion, the union computation,
  the machine gate, and attestation coverage.
