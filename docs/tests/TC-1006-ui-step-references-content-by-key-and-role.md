---
id: TC-1006
title: UI step references content by key and role
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
runner-args: tc_1006_ui_step_references_content_by_key_and_role
---

## Scenario — standing words are carried by keyed reference, never as literals

**Given** a captured What graph with a `UiStep` `ReviewOrder`,
**When** the user attaches a heading content reference (`checkout.review.heading`,
role `heading`) and an empty-state reference (`cart.empty.message`, role
`empty-message`) to the step,
**Then** the graph records two `references_content` edges from `ReviewOrder`,
each naming a `ContentKey` with its declared role, and **no literal copy string**
is stored on the step.

**And when** an author instead bakes a literal heading string onto the step,
**then** the process exits non-zero and the content rule rejects it, pointing at
the key + role form — copy on the What is a reference, exactly as a control is an
AIO and a style is a token.

## Validates

- FT-138 — Content references and the content store
- ADR-082 — Content is carried by reference and resolved against a locale-parameterised content store
