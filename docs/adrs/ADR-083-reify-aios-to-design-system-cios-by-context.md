---
id: ADR-083
title: Screens bind to a design system; AIOs reify to CIOs by context of use
status: accepted
features:
- FT-139
supersedes: []
superseded-by: []
domains:
- api
- data-model
scope: feature-specific
content-hash: sha256:9f2fc415ba8a260b3cc38b8febab6f2c31df91f13fe791e007386bc8ef204f40
source-files:
- product-core/src/pf/ids.rs
- product-core/src/pf/model.rs
- product-core/src/pf/how_turtle.rs
- product-core/src/pf/reify.rs
- product-core/src/pf/rules_how.rs
---

## Context

§4.5 of the framework specifies the **Concrete UI layer**: where the What carries
a screen as a UI step typed against **Abstract Interaction Objects** (AIOs,
ADR-078), the How realises it by **binding the screen to a design system**, not
by inventing a bespoke UI-description language. The design system's components
are the **Concrete Interaction Objects (CIOs)** that AIOs reify into. The bridge
from the AUI to the Concrete UI is a declared set of **reification rules**, each
mapping an AIO *in a given context of use* to a concrete control:
`reify(AIO, context) → CIO`. This is where the phone-vs-tablet decision lives —
below the What, as a traceable rule rather than a choice buried in code.

The `pf/` engine has the What-side AIO vocabulary and the typed `UiStep`
(ADR-078) but **no How-side UI material at all** — no design system, no CIO
catalog, no tokens, no reification. `product status` reports `How: (none)`. The
screen-composition contract of §4.5 has no representation, so a UI step has
nothing to be realised against and the seam (ADR-084) has no How half to check.

## Decision

Introduce the screen-composition contract as graph structure, in four parts:

1. **New node kinds.** Add `DesignSystem`, `Cio`, `Token`, and `ReificationRule`
   to `NodeKind` (`pf/ids.rs`) and the `pf:` ontology. A `DesignSystem` owns a
   **closed `Cio` catalog** (its component vocabulary) and a **`Token` surface**
   (colour, spacing, typography, …). Atomic Design is the normative
   compositional structure: every screen is a composition over atoms → molecules
   → organisms → templates → pages, and nothing in a screen may exist outside
   those levels.

2. **A page is the realised form of a UI step.** Each `UiStep` is realised by a
   page via `realizes_step`; the page `composes` only catalog CIOs, and `binds`
   its controls to the commands the step `offers` and its fields to the
   projections the step `surfaces`. The screen's data and controls are therefore
   *derived* from the UI step, not authored on the screen.

3. **Reification rules.** A `ReificationRule` declares `reify(AIO, context) →
   CIO` via `reifies` (to the CIO) and `in_context` (the context of use,
   ADR-078), and **carries rationale** — the UX reasoning that is otherwise lost.
   One AIO reifies to *many* CIOs by context; the What (the AIO) is unchanged,
   which is the entire reason the AIO layer exists.

4. **Three How-side checks** (`rules_how`/`reify`):
   - **Reification coverage** — a conformant instance must provide a rule for
     *every* (AIO, context) pair its UI steps can encounter, or some screen is
     left unspecified for some device. This is the design-system analogue of a
     Decider's command coverage (ADR-061).
   - **Closed vocabulary** — a rule's CIO must be a component the design system
     defines; a rule targeting a non-catalog component fails. Reification chooses
     *among* the system's components; it never invents one.
   - **Tokens, not literals** — a screen carrying a literal style value instead
     of a `Token` reference is non-conformant.

**Root navigation reifies like any AIO.** The `navigate` AIO at the application
root (ADR-079) reifies per context by these same rules — phone → drawer/burger,
tablet → rail, desktop → persistent sidebar. The application chrome a renderer
draws is nothing more than reified root-navigation; it is not separately authored
content, so it needs no separate model.

## Rationale

- Binding to a design system rather than describing screens bespoke buys the
  system's components, tokens, accessibility, and tooling for free — exactly as
  §4.4 binds interfaces to industry standards rather than hand descriptions.
- Reification rules as first-class, rationale-carrying nodes make the
  phone-vs-tablet decision a *traceable artifact* that participates in the
  rationale trace, rather than a decision buried in code.
- Coverage, closed-vocabulary, and tokens-not-literals expressed as graph rules
  keep every cross-reference check in the graph (`rules_how`), consistent with
  the project standard and reusing the oxigraph runner.
- Treating chrome as reified root-navigation keeps one source of truth for
  navigation (the page graph, ADR-079) instead of a second shell model that can
  drift.

## Rejected alternatives

- **A bespoke screen-description format.** Rejected: it forfeits the design
  system's components, tokens, accessibility, and tooling — non-conformant where
  a design system exists (§4.5).
- **Literal style values on screens.** Rejected: literals drift exactly as
  hand-written interface contracts do; styling must be `Token` references.
- **A separately-authored application shell above the screens.** Rejected: the
  chrome is the reified root-navigation of the one page graph; a second shell
  model duplicates the transition machinery and creates a second place
  navigation can drift (ADR-079).

## Test coverage

- TC-1009 — one AIO reifies to different CIOs by context, via rationale-carrying
  rules; the What is unchanged.
- TC-1010 — reification coverage over (AIO, context): full coverage passes; a
  missing pair fails, naming the gap.
- TC-1011 — a rule targeting an off-system component fails the closed-vocabulary
  check; a literal style value instead of a token is non-conformant.
- `pf::reify` + `pf::rules_how` unit tests cover the coverage rule, the
  closed-vocabulary rule, and tokens-not-literals (pass and each failure shape).
