---
id: FT-142
title: Content Store Conformance Profile (preview)
phase: 7
status: planned
depends-on:
- FT-138
adrs:
- ADR-085
tests:
- TC-1017
- TC-1018
domains:
- api
- data-model
domains-acknowledged:
  ADR-041: Additive — adds a preview content manifest format + validator; nothing existing is removed or deprecated, so no absence TC is required.
  ADR-047: The functional specification lives in this feature's body, not a separate artifact.
  ADR-042: TCs use the reserved `scenario` type only; no new TC type is introduced.
  ADR-050: PAT-001 (slice + adapter) governs the `pf` slice + CLI adapter; no new implementation pattern is introduced.
  ADR-049: Not a context-bundle/template command; no template surface changes.
  ADR-043: Manifest validation + the coupling check live in the pure `pf` slice (manifest/content_store); the CLI is a thin adapter.
  ADR-048: Reads a content-store manifest and the captured What graph only; no other side effects.
  ADR-051: Every TC declares `observes:` and asserts on those surfaces (graph, exit-code).
  ADR-018: Scenario TCs drive the binary through assert_cmd; `pf::manifest`/`pf::content_store` carry unit tests. No property or session dimension for a manifest validator.
  ADR-040: The profile is a How-side provider contract at the What/How boundary; the coupling check composes with the existing seam rules; the verify pipeline is untouched.
patterns:
- PAT-001
---

## Description

§12 of the framework — **🅿 PREVIEW**, non-normative — is the companion to §11 for
the *content* provider rather than the *component* provider: what a **content
store** must provide to plug in so screens referencing content keys can be
realised against it and pass the seam verification (§6.3). This feature reads a
content-store **manifest** (the §12.2 schema), validates it, and confirms it
couples to the What. The store is to words what a design system is to components;
**locale** is its context dimension. It introduces no new requirement; it is a
derived view of §3.2.1/§4.6 from the content store's side of the seam (ADR-085).

## Functional Specification

### Inputs

- A content-store manifest file (the §12.2 PREVIEW schema): `content_store`
  id/version; `locales_supported`; `entries`, each a stable `key` + a `role`
  (heading / body / empty-message / error-message / help / legal / …) + `values`
  (a resolved string per locale).
- The captured What graph for a product (its UI steps' and shell's content
  references), for the coupling check.

### Behaviour

- **Validate internal wholeness.** Every entry carries its `role`; every claimed
  locale has a value for every key; every error/empty role resolves to non-empty,
  actionable text. Reports each violation; exits non-zero on any.
- **Run the coupling check.** Confirm the manifest resolves every (content key,
  locale) the application's UI steps and shell reference (§3.2.1, §3.2.4). A key
  the application references that the store cannot resolve in a claimed locale
  makes the store **non-conforming for that locale**; the check names the missing
  (key, locale) pair — the content analogue of reification coverage over (AIO,
  context).
- Surface the manifest: list entries by role, and show resolution per locale.

### Error handling

- A key missing a value in a claimed locale, an entry without a role, or an
  error-message/empty-message role resolving to an empty string are clear,
  individually-named validation failures — never a bare fail.
- A manifest that does not parse against the §12.2 schema points the user at the
  expected shape.

## Out of scope

- The **normative content-reference model** (content keys + roles on UI steps,
  `resolve(key, locale) → string`, the `ContentKey`/`ContentStore`/`Locale` node
  kinds) is FT-138/ADR-082; this feature consumes a manifest, it does not define
  that model.
- The **design-system profile** (§11) is FT-141.
- The **seam verification** that uses a conforming store to discharge a screen's
  content coverage end to end is FT-140.

## Acceptance

- TC-1017 — a content-store manifest where every key carries a role and every
  claimed locale has a value for every key validates; a key missing a value in a
  claimed locale, or an error-message role resolving to empty, fails.
- TC-1018 — the coupling check passes when the manifest resolves every (content
  key, locale) the application references; an unresolved (key, locale) makes the
  store non-conforming for that locale, naming the gap.
