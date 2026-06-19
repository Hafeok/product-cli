---
id: FT-141
title: Design System Conformance Profile (preview)
phase: 7
status: planned
depends-on:
- FT-139
adrs:
- ADR-085
tests:
- TC-1015
- TC-1016
domains:
- api
- data-model
domains-acknowledged:
  ADR-041: Additive — adds a preview manifest format + validator; nothing existing is removed or deprecated, so no absence TC is required.
  ADR-047: The functional specification lives in this feature's body, not a separate artifact.
  ADR-042: TCs use the reserved `scenario` type only; no new TC type is introduced.
  ADR-050: PAT-001 (slice + adapter) governs the `pf` slice + CLI adapter; no new implementation pattern is introduced.
  ADR-049: Not a context-bundle/template command; no template surface changes.
  ADR-043: Manifest validation + the coupling check live in the pure `pf` slice (manifest/design_system); the CLI is a thin adapter.
  ADR-048: Reads a design-system manifest and the captured What graph only; no other side effects.
  ADR-051: Every TC declares `observes:` and asserts on those surfaces (graph, exit-code).
  ADR-018: Scenario TCs drive the binary through assert_cmd; `pf::manifest`/`pf::design_system` carry unit tests. No property or session dimension for a manifest validator.
  ADR-040: The profile is a How-side provider contract at the What/How boundary; the coupling check composes with the existing seam rules; the verify pipeline is untouched.
patterns:
- PAT-001
---

## Description

§11 of the framework — **🅿 PREVIEW**, non-normative — states what a design system
must provide to plug in as the Concrete-UI layer: a *conforming design system*
against which any conformant What can be realised and pass the seam verification
(§6.3). This feature reads a design-system **manifest** (the §11.3 schema),
validates it, and confirms it couples to the What. It introduces no new
requirement; it is a derived view of §3.2.2/§3.2.3/§4.5 from the design system's
side of the seam (ADR-085).

## Functional Specification

### Inputs

- A design-system manifest file (the §11.3 PREVIEW schema): `design_system`
  id/version/`wcag_target`; `contexts_supported` (form factor, modality);
  `components` (the CIO catalog), each with its `tokens` and its WCAG `satisfies`
  guarantees (criterion + level + verification type); `reification` rules
  (`reify(aio, when) → cio` with rationale); and the `tokens` surface.
- The captured What graph for a product (its UI steps, the core AIO set, and the
  declared contexts of use), for the coupling check.

### Behaviour

- **Validate internal wholeness.** Every `cio` named in `reification` exists in
  `components`; every `criterion` a component claims is a real WCAG 2.2 entity;
  every token a component references is declared in `tokens`. Reports each
  violation; exits non-zero on any.
- **Run the coupling check.** Confirm the manifest's `reification` covers every
  core AIO across every context in `contexts_supported` — the design-system
  analogue of a Decider's command coverage. A core AIO with no reifying CIO in a
  claimed context makes the design system **non-conforming for that context**;
  the check names the missing (AIO, context) pair.
- Surface the manifest: list components, the reification table, and the token
  surface; show, per component, the WCAG criteria it discharges by construction.

### Error handling

- A reification rule naming a non-catalog `cio`, a component referencing an
  undeclared token, or a `satisfies` entry citing a non-WCAG criterion are clear,
  individually-named validation failures — never a bare fail.
- A manifest that does not parse against the §11.3 schema points the user at the
  expected shape.

## Out of scope

- The **normative reification model** (the `Cio`/`ReificationRule` node kinds,
  `reify(AIO, context) → CIO`, tokens-not-literals) is FT-139/ADR-083; this
  feature consumes a manifest, it does not define that model.
- The **content-store profile** (§12) is FT-142.
- The **seam verification** that uses a conforming manifest to discharge a
  screen's reification coverage end to end is FT-140.

## Acceptance

- TC-1015 — a design-system manifest whose reification cios all exist, whose
  tokens are all declared, and whose criteria are all real WCAG 2.2 entities
  validates; a reification naming a non-catalog cio fails.
- TC-1016 — the coupling check passes when `reification` covers every core AIO ×
  supported context; a missing (AIO, context) makes the design system
  non-conforming for that context, naming the gap.
