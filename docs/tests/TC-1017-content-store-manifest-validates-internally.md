---
id: TC-1017
title: content store manifest validates internally
type: scenario
status: unimplemented
validates:
  features:
  - FT-142
  adrs:
  - ADR-085
phase: 7
observes:
- graph
- exit-code
runner: cargo-test
runner-args: tc_1017_content_store_manifest_validates_internally
---

## Scenario — a whole content-store manifest validates; a missing locale or empty error role fails

**Given** a content-store manifest (the §12.2 schema) in which every entry
carries a `role`, every locale in `locales_supported` has a value for every key,
and every error-message and empty-message role resolves to non-empty, actionable
text,
**When** the user validates the manifest,
**Then** the process exits 0 and the validator reports the manifest internally
whole.

**And given** a second manifest in which one key lacks a value for a claimed
locale, **or** an `error-message` role resolves to an empty string, **when** the
user validates it, **then** the process exits non-zero and the validator emits a
finding naming the offending (key, locale) or the empty role — content is checked
for more than mere presence because each key carries a role.

## Validates

- FT-142 — Content Store Conformance Profile (preview)
- ADR-085 — Preview conformance profiles for the design system and the content store
