---
id: FT-149
title: Trigger block and the four event-model patterns
phase: 1
status: complete
depends-on: []
adrs:
- ADR-091
tests:
- TC-1032
- TC-1033
domains: []
domains-acknowledged: {}
---

## Description

Framework §3.2.0 names the event model's four building blocks (Trigger,
Command, Event, View) and the four patterns they compose into (Command, View,
Automation, Translation). This feature adds the missing block — the **Trigger**
— as a captured domain node (the 26th `NodeKind`): *what initiates a command*,
with a `source` of exactly one of `user`, `external`, or `automated`. An
automated trigger expresses the **Automation** pattern by `watches`-ing a View
and issuing a command (observe, then act); a **Translation** additionally reads
from one source `System`. Generalising "what issues a command" into one block is
what makes automation and cross-system communication checkable shapes rather
than special cases.

Triggers serialize to Turtle under `pf:Trigger` with `pf:source`/`pf:issues`/
`pf:watches`/`pf:translatesFrom`, round-trip through the seed parser, and are
validated for the Automation and Translation pattern shapes.

## Functional Specification

### Inputs

- `product domain new trigger <id> --label <name> --trigger-source <user|external|automated> --issues <command-id> [--watches <read-model-id>] [--translates-from <system-id>]`

### Behaviour

- A `Trigger` is captured with its source and the command it issues, and appears
  in `domain list trigger`, `domain show`, and the Turtle export.
- An automated trigger that `watches` a declared read model is a well-formed
  Automation pattern; a trigger that `translates_from` a declared system is a
  well-formed Translation pattern.
- The block round-trips through `domain export` → `seed::from_turtle`.

### Error handling

- A trigger missing its source, or whose source is not user/external/automated,
  is rejected (`§3.2.0`).
- A trigger whose issued command does not resolve to a declared Command is rejected.
- An automated trigger that watches no View is rejected (Automation observes,
  then acts); a watched View or a `translates_from` system that does not resolve
  is rejected.

## Out of scope

- Command-pattern completeness — flagging *every* command that no trigger issues.
  Deferred (like flow→system ownership in FT-148) to avoid invalidating existing
  What graphs; the integrity of declared triggers is validated here.
- Enforcing that an automated trigger's logic is empty ("no business logic") —
  the framework states it; mechanically the shape (automated ⇒ watches a View) is
  what is checked.

## Acceptance

- TC-1032, TC-1033 pass; the block round-trips through Turtle; `cargo t` + clippy green.
