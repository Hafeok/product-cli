---
id: ADR-085
title: Preview conformance profiles for the design system and the content store
status: accepted
features:
- FT-141
- FT-142
supersedes: []
superseded-by: []
domains:
- api
- data-model
scope: feature-specific
content-hash: sha256:98560f81bda930082607466e9a1302cc633c9ecd9cc4c1f63185891bc0761cc4
source-files:
- product-core/src/pf/manifest.rs
- product-core/src/pf/design_system.rs
- product-core/src/pf/content_store.rs
- product-cli/src/commands/profile.rs
---

## Context

§11 (Design System Conformance Profile) and §12 (Content Store Conformance
Profile) of the framework are marked **🅿 PREVIEW** — published ahead of a worked
reference instance and explicitly subject to change. Each introduces **no new
requirements**: §11 is a derived view of §3.2.2/§3.2.3/§4.5 reorganised from the
*design system's* side of the seam, and §12 is a derived view of §3.2.1/§4.6
reorganised from the *content store's* side. Both state what a swappable provider
must supply to plug into the Concrete-UI layer and pass the seam verification
(§6.3), so a product can swap design systems or content stores without touching
its What.

The two profiles are deliberately **parallel**: the design system is to
components what the content store is to words; both are swappable providers
behind a conformance profile; both carry a context dimension (context of use vs.
locale); both are resolved at render time against a reference the What carries.
Nothing in the toolchain yet reads either manifest or checks it against the What.

## Decision

Adopt the §11.3 and §12.2 **PREVIEW YAML manifest schemas** as the on-disk format
for the two providers, and give each manifest two checks behind a shared
`pf::manifest` machine:

1. **A conforming design-system manifest** (§11.3) declares four things — and the
   validator checks all four are internally whole:
   - a **component catalog** (the CIOs) with a stable id each — the closed
     vocabulary a screen may compose from;
   - **reification coverage** — a `reify(AIO, context) → CIO` rule for every core
     AIO across every context the manifest claims to support;
   - a **token surface** — every styling choice a component exposes is a token
     reference, never a literal;
   - **accessibility guarantees** — per component, which WCAG 2.2 criteria it
     satisfies *by construction* and by which verification type, so a screen
     **inherits** discharge instead of re-attesting per screen.
   The **validator** confirms internal wholeness: every `cio` named in
   `reification` exists in `components`, every `criterion` is a real WCAG 2.2
   entity, every token a component references is declared. A **coupling check**
   confirms the manifest's `reification` covers every core AIO across every
   context in `contexts_supported` — the design-system analogue of a Decider's
   command coverage.

2. **A conforming content-store manifest** (§12.2) declares three things:
   - a **keyed catalog** — each entry a stable key + a role (heading / body /
     empty-message / error-message / help / legal / …);
   - **locale coverage** — a resolved string per claimed locale for every key;
   - **role conformance** — each value satisfies its role's constraints
     (error-message non-empty and actionable; heading within its length;
     legal text present where required).
   The **validator** confirms every key carries its role, every claimed locale
   has a value for every key, and every error/empty role resolves to non-empty
   actionable text. A **coupling check** confirms the manifest resolves every
   (content key, locale) the application's UI steps and shell reference — the
   content analogue of reification coverage over (AIO, context).

Both profiles are marked clearly as **preview / non-normative**; where they and
the normative body appear to differ, the normative body governs. They will be
proposed for normative status together, once a reference instance demonstrates a
conforming design system **and** a conforming content store resolving the *same*
What end to end.

## Rationale

- The manifests are the **input data** for the normative reification (ADR-083)
  and content (ADR-082) models; making them first-class, validated artifacts is
  what lets the seam verification (ADR-084) consume a provider's declarations
  rather than guess them.
- Validator (internal wholeness) and coupling check (covers what the What
  references) are the same two-layer shape both profiles share — so they reuse
  one `pf::manifest` machine, honouring the deliberate parallel the framework
  draws between §11 and §12.
- Shipping tooling now, behind a preview flag, is exactly the coupling
  experiment the profiles were published to invite; it is how the schema earns
  its way to normative status instead of being frozen on paper.

## Rejected alternatives

- **Wait for normative status before building any tooling.** Rejected: the
  profiles are Preview *precisely to be exercised*; a reference instance is the
  evidence that promotes them. Tooling gated behind a preview marker is the
  intended path, not a violation of it.
- **Two unrelated manifest schemas with separate validators.** Rejected: §11 and
  §12 are deliberately parallel (provider + context dimension + render-time
  resolution); a shared validator/coupling machine keeps them from drifting and
  states the parallel in code.
- **Bake provider data directly into the What graph instead of a manifest.**
  Rejected: that re-couples the What to a specific design system / content
  store, destroying the pluggability the AUI layer and content references exist
  to provide.

## Test coverage

- TC-1015 — a design-system manifest validates internally; a reification naming
  a non-catalog cio fails.
- TC-1016 — the design-system coupling check covers every core AIO × supported
  context; a missing pair is non-conforming for that context.
- TC-1017 — a content-store manifest validates internally; a missing locale
  value or an empty error-message role fails.
- TC-1018 — the content-store coupling check resolves every referenced (key,
  locale); an unresolved pair is non-conforming for that locale.
