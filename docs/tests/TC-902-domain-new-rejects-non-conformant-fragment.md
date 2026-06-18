---
id: TC-902
title: domain new rejects non conformant fragment
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
- stderr
runner: cargo-test
runner-args: tc_902_domain_new_rejects_non_conformant_fragment
---

## Scenario — a non-conformant create is rejected

**Given** a graph with a bounded context but no `Order` entity,
**When** the user runs `product domain new event … --changes Nope` (an event
changing a non-entity),
**Then** the process exits 1, stderr carries the §3.2 framework message, and a
follow-up `domain list event` shows the fragment was not committed.

## Validates

- FT-110 — product domain — CLI list, show, and CRUD over the captured What graph
- ADR-053 — Domain authoring is a separate What graph with native in-loop conformance
