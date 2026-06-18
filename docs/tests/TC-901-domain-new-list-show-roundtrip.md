---
id: TC-901
title: domain new list show roundtrip
type: scenario
status: passing
validates:
  features:
  - FT-110
  adrs:
  - ADR-053
phase: 6
observes:
- exit-code
- stdout
runner: cargo-test
runner-args: tc_901_domain_new_list_show_roundtrip
---

## Scenario — build, list, and show a graph from the CLI

**Given** a repo whose configured product is `test`,
**When** the user runs `product domain new` for a context, an entity, an
event, and a command, then `product domain list`, `domain list entity`, and
`domain show <id>`,
**Then** each command exits 0; `list` stdout shows the entity and command
rows, the filtered `list entity` stdout omits the command, and `show`
stdout includes the node fields plus its `changedByEvents` links.

## Validates

- FT-110 — product domain — CLI list, show, and CRUD over the captured What graph
- ADR-053 — Domain authoring is a separate What graph with native in-loop conformance
