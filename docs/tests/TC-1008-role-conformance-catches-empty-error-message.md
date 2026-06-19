---
id: TC-1008
title: role conformance catches empty error message
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
runner-args: tc_1008_role_conformance_catches_empty_error_message
---

## Scenario — a role makes content checkable, not merely present

**Given** a What graph whose `UiStep` references `cart.failed.message` with role
`error-message`,
**And** a `ContentStore` that `resolves` `cart.failed.message` to an **empty**
string in locale `en`,
**When** the user runs the role-conformance check,
**Then** the process exits non-zero and the check reports that the
`error-message` role resolves to empty for (`cart.failed.message`, `en`) — an
empty error message is caught at check time, not discovered in production.

**And** a non-empty, actionable resolution for the same key passes — the role is
the What-side meaning, the string is the How-side value, and the check confirms
the value satisfies the role.

## Validates

- FT-138 — Content references and the content store
- ADR-082 — Content is carried by reference and resolved against a locale-parameterised content store
