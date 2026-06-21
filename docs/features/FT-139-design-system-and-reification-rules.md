---
id: FT-139
title: Design system and reification rules
phase: 7
status: complete
depends-on:
- FT-134
adrs:
- ADR-083
tests:
- TC-1009
- TC-1010
- TC-1011
domains:
- api
- data-model
domains-acknowledged:
  ADR-041: Additive — adds the DesignSystem/Cio/Token/ReificationRule node kinds and the reifies/in_context/realizes_step/composes/binds edges; this is the first How-side material in the UI system, so nothing existing is removed or deprecated and no absence TC is required.
  ADR-047: The functional specification lives in this feature's body, not a separate artifact.
  ADR-048: Reads/writes the captured graph only (the How contract for an archetype); no other side effects.
  ADR-049: Not a context-bundle/template command; no template surface changes.
  ADR-050: PAT-001 (slice + adapter) governs the `pf` slice + CLI adapter; no new implementation pattern is introduced.
  ADR-042: TCs use the reserved `scenario` type only; no new TC type is introduced.
  ADR-018: Scenario TCs drive the binary through assert_cmd; `pf::reify`/`pf::rules_how` carry unit tests. No property or session dimension for a reification model and its rules.
  ADR-043: The reification model and the coverage/closed-vocabulary/token rules live in the pure `pf` slice (`reify`, `rules_how`); the CLI is a thin adapter. This populates the How side, previously reported as "(none)".
  ADR-040: The design system and reification are How-side (Concrete UI) material at the What/How seam; the coverage/closed-vocabulary rules compose with the existing How-side rules; the verify pipeline is untouched.
  ADR-051: Every TC declares `observes:` and asserts on those surfaces (graph, exit-code, stdout).
patterns:
- PAT-001
---

## Description

§4.5 of the framework specifies the **screen-composition contract** — the
Concrete UI layer. Where a UI step (FT-134) is typed against Abstract
Interaction Objects, the How realises it by **binding the screen to a design
system**: each AIO is **reified** into a concrete control (a CIO) by a declared
rule `reify(AIO, context) → CIO`, and pages compose only on-system components,
structured by Atomic Design. This feature adds the first How-side UI material to
the `pf/` graph (ADR-083).

## Functional Specification

### Inputs

- The captured graph for a product (its What — AIOs, contexts of use, UI steps —
  and its How contract for an archetype; `--product` to override the default).
- A design system: a closed `Cio` catalog, a `Token` surface, and a set of
  `ReificationRule`s.

### Behaviour

- **Declare a design system.** A `DesignSystem` node owns a **closed `Cio`
  catalog** (its component vocabulary) and a **`Token` surface**. A screen may
  compose only catalog CIOs and may reference style only as tokens.
- **Author reification rules.** A `ReificationRule` declares `reify(AIO, context)
  → CIO` via `reifies` and `in_context`, and carries **rationale**. One AIO
  reifies to different CIOs per context (e.g. `single-select` → `segmented-
  control` on a tablet with few options, → `searchable-list` on a phone with
  many) — the AIO in the What is unchanged.
- **Realise a UI step.** A page `realizes_step` a `UiStep`, `composes` catalog
  CIOs (atoms → molecules → organisms → templates → pages), and `binds` controls
  to the commands the step `offers` and fields to the projections it `surfaces`.
- **The reification-coverage check.** For every (AIO, context) pair the
  product's UI steps can encounter, there must be a reifying rule; the check
  reports any uncovered pair (a screen left unspecified for some device) — the
  design-system analogue of Decider command coverage.
- **The closed-vocabulary check.** A reification rule whose CIO is not in the
  design system's catalog fails, naming the rule and the off-system component.
- **Tokens, not literals.** A screen carrying a literal style value instead of a
  token reference is non-conformant.
- **Root navigation as chrome.** The `navigate` AIO at the application root
  (FT-135) reifies per context by the same rules (phone → drawer, tablet → rail,
  desktop → sidebar); the chrome is reified root-navigation, not separately
  authored.

### Error handling

- A reification rule referencing an unknown AIO, context, or CIO is a clear
  error pointing at the recognised set / the catalog.
- Coverage and closed-vocabulary failures report each offending pair/rule, not a
  bare fail.

## Out of scope

- **The §11 Design System Conformance Profile** — the design-system *manifest*
  format and its validator (internal wholeness + coupling check) — is FT-141;
  this feature establishes the in-graph reification model and its checks.
- **The full seam verification** that composes reification coverage with state
  coverage, content coverage, and accessibility discharge into one verdict is
  FT-140.
- **WCAG accessibility guarantees** a CIO carries (so a screen inherits its
  discharge) are FT-137; here a CIO is a component with an identifier and tokens.

## Acceptance

- TC-1009 — one AIO reifies to different CIOs by context, via rationale-carrying
  rules; the What is unchanged.
- TC-1010 — reification coverage over (AIO, context): full coverage passes; a
  missing pair fails, naming the gap.
- TC-1011 — an off-system component fails the closed-vocabulary check; a literal
  style value instead of a token is non-conformant.
