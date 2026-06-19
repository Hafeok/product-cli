---
id: FT-138
title: Content references and the content store
phase: 7
status: planned
depends-on:
- FT-134
adrs:
- ADR-082
tests:
- TC-1006
- TC-1007
- TC-1008
domains:
- api
- data-model
domains-acknowledged:
  ADR-041: Additive — adds the ContentKey/ContentStore/Locale node kinds and the references_content/resolves/in_locale edges; nothing is removed or deprecated, so no absence TC is required this increment.
  ADR-047: The functional specification lives in this feature's body, not a separate artifact.
  ADR-042: TCs use the reserved `scenario` type only; no new TC type is introduced.
  ADR-050: PAT-001 (slice + adapter) governs the `pf` slice + CLI adapter; no new implementation pattern is introduced.
  ADR-049: Not a context-bundle/template command; no template surface changes.
  ADR-043: Content references, the store model, and the (key,locale) coverage and role-conformance rules live in the pure `pf` slice; the CLI is a thin adapter.
  ADR-048: Reads/writes the captured What graph and the declared content store only; no other side effects.
  ADR-051: Every TC declares `observes:` and asserts on those surfaces (graph, exit-code).
  ADR-018: Scenario TCs drive the binary through assert_cmd; `pf::content`/`pf::rules_ui` carry unit tests. No property or session dimension for content references and coverage.
  ADR-040: Content references are What-side; the store is the How-side resolver; the coverage rule composes with the existing What-side rules; the verify pipeline is untouched.
patterns:
- PAT-001
---

## Description

§3.2.1 (the *Content references* bullet) and §4.6 specify how a screen carries
its **standing authored words** — heading, body copy, empty-state prose,
error-state prose, help text, legal text. These are neither projected data nor a
control label, and the framework forbids writing them as literal strings on the
UI step: they are referenced by **content key with a declared role**, and
resolved by the How against a **content store** — `resolve(content_key, locale)
-> string`.

This feature (ADR-082) gives the `UiStep` (FT-134) a place to carry its words by
reference, models the content store and its locales, and adds the two checks that
make copy a checkable obligation rather than a literal baked into the What.

## Functional Specification

### Inputs

- The captured What graph for a product (the domain session; `--product` to
  override the default), including its `UiStep` nodes.
- Content references (key + role) attached to a UI step.
- A declared **content store**: the locales it covers and, per key per locale,
  the resolved string.

### Behaviour

- **Attach content references to a UiStep.** A `UiStep` gains
  `references_content` edges, each naming a `ContentKey` **with a role**
  (`heading`, `body`, `empty-message`, `error-message`, `help`, `legal`, …). No
  literal copy string may appear on a step; the reference is the only legal form
  (a literal is rejected, like a non-AIO control in FT-134).
- **Declare a content store over locales.** A `ContentStore` declares the
  `Locale`s it covers and `resolves` each referenced `ContentKey` to a string
  `in_locale` each covered locale. The same content keys resolve through stores
  covering different locales — the What does not change per language.
- **Content coverage** (`pf::rules_ui`). A graph rule: the store must resolve
  every `(content key, locale)` the application's UI steps reference. A missing
  resolution fails the check, naming the `(key, locale)` gap — the central
  obligation, parallel to reification coverage over `(AIO, context)`.
- **Role conformance.** Because each key carries a role, resolution is checked for
  more than existence: an `error-message`/`empty-message` that resolves to empty,
  or a `heading` longer than its role admits, is caught at check time.
- **Pure-content pages.** A `UiStep` with only content references and no
  projection or command is a valid **pure-content page** (an "about" page) whose
  interaction is "read" — no new page kind is introduced.

### Error handling

- A literal copy string on a UI step is rejected, pointing the author at the
  key + role form.
- Content coverage reports each missing `(key, locale)` pair, not a bare fail.
- Role conformance names the offending key, role, and locale (e.g. "error-message
  `cart.failed.message` resolves to empty in `de`").

## Out of scope

- **The §12 Content Store Conformance Profile** — the `content-store.manifest.yaml`
  shape, its internal-wholeness validator, and the coupling check from a manifest
  to the What — is FT-142; this feature establishes the in-graph model and rules
  the profile is a packaging of.
- **The seam composition** that folds content coverage in with reification, state,
  and accessibility coverage into one verdict is FT-140.

## Acceptance

- TC-1006 — a UI step references a heading and an empty-message by key + role
  (no literals); a literal baked into the step is rejected.
- TC-1007 — content coverage over (key, locale): a store covering {en, es}
  passes; a key missing its `es` value fails, naming the gap.
- TC-1008 — role conformance catches an `error-message` that resolves to empty at
  check time, not in production.
