---
id: TC-1007
title: content coverage over key and locale
type: scenario
status: unimplemented
validates:
  features:
  - FT-138
  adrs:
  - ADR-082
phase: 7
observes:
- graph
- exit-code
runner: cargo-test
runner-args: tc_1007_content_coverage_over_key_and_locale
---

## Scenario — the store must resolve every referenced key in every claimed locale

**Given** a What graph whose UI steps reference the content keys
`checkout.review.heading` and `cart.empty.message`,
**And** a `ContentStore` covering locales {`en`, `es`} that `resolves` both keys
`in_locale` `en` and `in_locale` `es`,
**When** the user runs the content-coverage check,
**Then** the process exits 0 — every `(content key, locale)` the application
references is resolved.

**And when** the `es` value for `cart.empty.message` is removed, **then** the
check exits non-zero and names the missing pair (`cart.empty.message`, `es`) — a
screen with missing words in some language is caught, the same coverage
obligation reification has over `(AIO, context)`.

## Validates

- FT-138 — Content references and the content store
- ADR-082 — Content is carried by reference and resolved against a locale-parameterised content store
