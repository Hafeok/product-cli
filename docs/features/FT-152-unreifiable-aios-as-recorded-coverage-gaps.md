---
id: FT-152
title: Unreifiable AIOs as recorded coverage gaps
phase: 1
status: complete
depends-on: []
adrs:
- ADR-094
tests:
- TC-1036
domains: []
domains-acknowledged: {}
---

## Description

Framework §4.5 lets a design system declare an `(AIO, interaction class)` pair
**unreifiable** — some abstractions do not honestly survive a change of substrate
(a `display-collection` of product images has no faithful TUI form). The honest
move is to *record* the gap with a rationale rather than force an awkward
reification or fail silently — the same honesty as tagging a WCAG criterion
*manual* (§3.2.3) or naming the Polanyi floor (§3.5).

This feature adds `UnreifiableRule { aio, class, rationale }` as the 27th captured
domain NodeKind, authorable through `product domain new`, serialized under
`pf:UnreifiableRule` with `pf:reifies`/`pf:unreifiableIn`/`pf:rationale`,
round-tripped by the seed parser, and validated so a recorded gap is never a
silent omission.

## Functional Specification

### Inputs

- `product domain new unreifiable-rule <id> --aio <aio-id> --class <gui|tui> --rationale <why>`

### Behaviour

- A recorded gap is captured naming a real AIO (a core AIO or a declared `Aio`),
  a recognised interaction class, and a rationale; it appears in `domain list
  unreifiable-rule` and the Turtle export.

### Error handling

- A rule whose AIO is not a recognised AIO is rejected (`§4.5`).
- A rule whose class is not a recognised interaction class (gui/tui) is rejected.
- A rule with no rationale is rejected — a recorded gap must say *why*.

## Out of scope

- The seam-verification consumption — treating a UI step that uses an
  unreifiable AIO in a class its system targets as an authoring-time finding, and
  letting a declared-unreifiable pair satisfy reification coverage as a recorded
  gap. This builds on the multi-hop step->flow->system->class link and is
  deferred; this feature lands the recorded-gap node and its integrity.

## Acceptance

- TC-1036 passes; the rule round-trips through Turtle; `cargo t` + clippy green.
