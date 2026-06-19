---
id: ADR-082
title: Content is carried by reference and resolved against a locale-parameterised content store
status: accepted
features:
- FT-138
supersedes: []
superseded-by: []
domains:
- api
- data-model
scope: feature-specific
content-hash: sha256:c38ceeec6bd55378b92671f9068abcfbb4fd02c2dda0dbe3ddbe941d595cc55f
source-files:
- product-core/src/pf/ids.rs
- product-core/src/pf/model.rs
- product-core/src/pf/content.rs
- product-core/src/pf/turtle.rs
- product-core/src/pf/rules_ui.rs
---

## Context

§3.2.1 and §4.6 of the framework specify how a screen carries its **standing
authored words** — a heading, explanatory body copy, the prose of an empty or
error state, help text, legal text. These are *not* projected data (they come
from no read model) and *not* a control label (they belong to no command). The
framework requires they be referenced by **content key with a declared role**
(heading / body / empty-message / error-message / help / legal / …), **never**
written as literal strings in the UI step. The role and the obligation are What
("this page needs a heading that states its purpose; its empty state needs a
message conveying the way forward"); the words themselves are resolved by the
How against a **content store**:

```
resolve(content_key, locale) -> string
```

This is the **fourth instance of the framework's one UI move** — *do not bake the
concrete thing into the What; reference an abstraction the How resolves*:
widgets became AIOs (§3.2.2), styles became tokens (§4.5), components became CIOs
(§4.5), and now **copy becomes content references resolved against a store**. The
payoff is identical each time: a keyed reference can be translated, swapped, and
verified, where a literal cannot.

The `pf/` engine has no representation of content today: a `UiStep` (ADR-078) can
surface projections and offer commands, but it has nowhere to carry its words,
and there is no store to resolve them against — so copy would have to be baked in
as literals, the precise fusion §4.6 forbids.

## Decision

Model content as graph structure, parallel to the design system's reification:

1. **New node kinds.** Add `ContentKey` (an authored-word reference carrying a
   declared `role`), `ContentStore` (the swappable provider of words), and
   `Locale` to `NodeKind` (`pf/ids.rs`/`ALL_KINDS`) and the `pf:` ontology. A
   `ContentStore` declares the locales it covers and resolves a key to a string
   per locale (a `ContentEntry` value).

2. **Content references on the UiStep.** A `UiStep` (ADR-078) gains
   `references_content` edges — each naming a `ContentKey` **with a role** — for
   the standing words it carries. No literal copy string may appear on a step;
   the reference is the only legal form. A page may consist **entirely** of
   content references and no projection or command — a **pure-content page** (an
   "about" or informational page) is simply a `UiStep` whose interaction is
   "read", needing no new page kind.

3. **Locale is the store's context dimension.** Resolution is parameterised by
   **locale** exactly as reification is parameterised by context of use:
   `resolve(key, locale)` mirrors `reify(AIO, context)` (ADR-083). A localized
   application is one resolved against a store covering its target locales, with
   the *same* content keys — the What does not change per language, just as it
   does not change per device.

4. **Two checks** (`rules_ui`, the content rules):
   - **Content coverage** — the store must `resolve` every `(content key,
     locale)` the application's UI steps reference, or some screen has missing
     words in some language. This is the central obligation, parallel to a design
     system's reification coverage over `(AIO, context)` and a Decider's command
     coverage. Expressed as a graph rule and consumed by the seam (ADR-084).
   - **Role conformance** — because each key carries a role, resolution is
     checked for more than mere existence: an `error-message` or `empty-message`
     role that resolves to empty, or a `heading` longer than its role admits, is
     catchable. The role is the What-side meaning; the string is the How-side
     value; the check confirms the value satisfies the role.

The predicates are `references_content` (UI step → content key, with role),
`resolves` (content store → string), and `in_locale` (the locale a resolution
holds in).

## Rationale

- Carrying words by keyed reference rather than literal is the only form that can
  be translated (one What, many locales), swapped (editorial ownership), and
  verified (empty error messages caught at check time) — identical to why styles
  are tokens not values.
- Making locale the store's context dimension keeps content resolution
  structurally symmetric to reification, so the seam verification (ADR-084) can
  compose the two coverage obligations the same way (`resolve` and `reify` are
  the two resolutions of one surface, §4.6).
- Roles turn content from an opaque blob into a checkable obligation, so the
  store's value is held to the meaning the What declared — the move that lets the
  check find missing legal text and overflowing headings before production.

## Rejected alternatives

- **Literal copy strings on the UiStep.** Rejected: a literal baked into the What
  cannot be translated, swapped, or verified — the same drift reason tokens
  replace literal style values and interface contracts may not be hand-written.
- **A separate page kind for pure-content pages.** Rejected: an "about" page is
  just a page whose interaction is "read"; it needs only content references, not
  a new node kind. Adding one would fork every UI-step rule for no gain.
- **Resolve content in code, outside the graph.** Rejected: then coverage and
  role conformance could not be checked from the model, and the words would drift
  from the screens that reference them — the store must be a declared provider.

## Test coverage

- TC-1006 — a UI step references a heading and an empty-message by key + role,
  with no literal strings; a literal baked into the step is rejected.
- TC-1007 — content coverage over (key, locale): a store covering {en, es}
  passes; a key missing its `es` value fails, naming the (key, locale) gap.
- TC-1008 — role conformance catches an `error-message` that resolves to empty
  at check time, rather than in production.
- `pf::content` + `pf::rules_ui` unit tests cover the (key, locale) coverage rule
  and each role-conformance failure shape.
