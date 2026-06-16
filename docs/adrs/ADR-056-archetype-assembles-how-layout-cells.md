---
id: ADR-056
title: An archetype assembles How, layout, and cells from a directory and validates the whole
status: accepted
features:
- FT-114
supersedes: []
superseded-by: []
domains:
- api
- data-model
scope: feature-specific
content-hash: sha256:f5189a6865b17aa2b89eb0a1839a6887874ba49f27a1b5ccf64e5a10f34f2b0e
source-files:
- product-core/src/pf/archetype.rs
- product-core/src/pf/layout.rs
- product-cli/src/commands/archetype.rs
---

## Context

An archetype is "a reusable, pre-filled How for a recurring system shape; a
product realises one or more." Concretely it is three §4/§5 parts for one
shape: the **How contract** (FT-111), the **repository layout model** (§4.3,
not previously implemented), and the **task-type cells** (FT-113). Each part
already validates in isolation, but nothing assembled them or checked the
cross-part coherence: that cells belong to the archetype, that the How's
`layout_model` reference resolves, that a layout's rules cite real principles.

## Decision

Represent an archetype as a directory `.product/archetypes/<name>/` containing
`how-contract.yaml`, `layout.yaml`, and `cells/*.yaml`, and implement an
`Archetype` aggregate (`pf::archetype`) that loads the parts and validates the
whole:

1. **Each part against its own shapes** — `validate_how`, `validate_layout`
   (new, `pf::layout`, mirroring `layout-model.schema.json` incl. the two
   guards), and `validate_cell` per cell. Every finding is re-tagged with its
   part (`how/…`, `layout/…`, `<cell>/…`) so the source is unambiguous.
2. **Cross-part coherence (warnings)** — each cell's `archetype` field matches
   the archetype name; the How's `layout_model` reference has a backing
   `layout.yaml`; a layout's declared archetype matches.
3. **Assembly requirement (violation)** — an archetype must declare a How
   contract.

Cells' `domain:X` inputs are cross-checked against the captured What graph when
available (threaded through `validate_cell`), so the whole realisation chain —
What → How → cells → layout — is validated from one command,
`product archetype validate <name>`.

A new §4.3 layout model (`pf::layout`) is introduced as part of this work,
since the layout is a required archetype part: rules must each cite what they
`enforces` (Guard 1), `must_exist` rules need a cardinality, and prohibitions
(`must_not_exist`) need rationale (Guard 2), with exactly one rule-kind per
rule.

## Rationale

- A directory per archetype is the natural multi-archetype layout (a product
  realises one or more) and keeps each archetype's parts together and
  diff-reviewable, while the single-file `how`/`cell` commands remain for
  ad-hoc work.
- Part-tagged findings make a whole-archetype report actionable — you see
  immediately whether a violation is in the How, the layout, or a specific
  cell.
- Cross-part checks are warnings (a template archetype may be authored
  incrementally) except the one structural requirement (a How must exist).

## Rejected alternatives

- **Reuse the single-file `.product/how-contract.yaml` / `.product/cell.yaml`
  for archetypes.** Rejected: it cannot express more than one archetype, and an
  archetype is intrinsically a multi-part bundle.
- **A flat validator that concatenates all findings untagged.** Rejected: in a
  multi-part assembly the source of a violation is essential; tagging by part
  is what makes the report usable.

## Test coverage

- TC-940 — validate a full assembled archetype (How + layout + cells).
- TC-941 — a part violation is reported tagged with its part.
- TC-942 — an archetype with no How contract is non-conformant.
- TC-943 — show / list / init.
- TC-944 — cells' `domain:` inputs are cross-checked against the What graph.
