---
id: ADR-084
title: The seam verification confirms a screen and its UI step agree
status: accepted
features:
- FT-140
supersedes: []
superseded-by: []
domains:
- api
- data-model
scope: feature-specific
content-hash: sha256:565d9f8c177257f43511adff09f39f871bdca505cc4512c24a4c09c7538be283
source-files:
- product-core/src/pf/seam.rs
- product-core/src/pf/rules_ui.rs
- product-core/src/pf/rules_how.rs
- product-cli/src/commands/seam.rs
---

## Context

§4.5 ("The seam is verified") and §6.3 (the **Seam** verification kind) require
that a screen and the UI step it realises be checked for agreement. A screen
sits on the What→How seam exactly as application sits on the application↔runtime
seam (§4.2): the What declares what a screen *means* (its UI step — the
projection it surfaces, the commands it offers, the AIOs it is typed against, the
states it must cover, the content it references, the accessibility it must
satisfy), and the How declares how that meaning is *realised* (the reified
components, the resolved strings, the discharged criteria). Nothing yet confirms
the two halves agree.

The `pf:seam` VerificationKind is **already declared** in
`schema/ontology/product-framework.ttl` (around line 191) with no code behind it.
The earlier UI-system phases each define one half of an agreement check —
ADR-078 the AIO typing and the `surfaces`/`offers` edges, ADR-080 state coverage,
ADR-081 accessibility discharge, ADR-082 content coverage, ADR-083 reification
coverage — but no artifact *composes* them into the single seam verdict §6.3
names. This ADR fills that declared slot.

## Decision

Add a `pf::seam` slice and a `product seam` command that runs the **seam
verification** for a screen (a `UiStep`) or a flow, composing — not
re-implementing — the per-phase checks the earlier ADRs define:

1. **Datum projected** — every datum the page displays is `project`ed by a read
   model in its flow; no view needs a field no projection supplies (the
   completeness the Projector proves, §3.4 / ADR's projector rules).
2. **Control maps to command** — every control maps to a command valid at that
   step; no button issues a command the step cannot accept (the `offers` edges of
   ADR-078 checked against the Decider's `handles`, §3.3).
3. **Reification coverage** — every AIO the step references has a reifying CIO for
   each declared context of use (ADR-083).
4. **State coverage** — every state in each surfaced projection's declared state
   space is given a meaning by the step or explicitly waived (ADR-080).
5. **Content coverage** — every content key the step references resolves in the
   content store for each declared locale (ADR-082).
6. **Accessibility discharged** — every *machine* WCAG criterion passes as a
   deterministic gate, and every *assisted*/*manual* criterion has a recorded
   attestation (ADR-081).

The cheaper structural AIO-only rule (ADR-078) runs **first** as a by-construction
gate. The seam verdict then reports the conformance **level and its basis** —
never a bare pass — listing every failing sub-check when it fails.

## Rationale

- The seam is the precise analogue of the application↔runtime seam (§4.2): two
  separately-described parts checked for agreement. Modelling it as one composite
  verification keeps a single source of truth for "does this screen serve its
  flow?", answerable as a graph query.
- Composing the per-phase rules — rather than re-deriving them — means each
  sub-check stays owned by the phase that defines its data (its projection, its
  reification rules, its content store, its criteria), so the seam cannot drift
  from the checks it aggregates.
- Reporting level + basis rather than a bare pass is what makes the verdict
  actionable and auditable, matching the accessibility verdict's discipline
  (§3.2.3) and the framework's "no bare pass" rule.
- It is **required wherever screens are specified**, because nothing else makes
  the screen and the flow agree — the seam is the keystone the UI system's other
  phases are load-bearing for.

## Rejected alternatives

- **A single monolithic UI check authored from scratch.** Rejected: the
  sub-checks belong to the phases that define their data (reification to the
  design system, content to the store, states to the read model). Re-implementing
  them in one place would duplicate and drift from those definitions; the seam
  must *reuse* them.
- **A bare pass/fail verdict.** Rejected: the framework requires the verdict to
  report a conformance level and its basis (§4.5, §3.2.3). A bare boolean hides
  which obligations were met and at what level.

## Test coverage

- TC-1012 — a fully-agreeing screen passes; the verdict reports level + basis.
- TC-1013 — an unprojected datum or a foreign command fails the seam, named.
- TC-1014 — reification, state, and content coverage gaps each fail the seam and
  are listed independently, never collapsed into one opaque fail.
- `pf::seam` unit tests cover the composition (each sub-check pass/fail and the
  level/basis verdict assembly).
