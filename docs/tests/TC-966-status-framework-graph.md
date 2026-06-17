---
id: TC-966
title: status shows the framework graph
type: scenario
status: passing
validates:
  features:
  - FT-125
  adrs:
  - ADR-066
phase: 6
observes:
- exit-code
- stdout
runner: cargo-test
runner-args: tc_966_status_shows_the_framework_graph
---

## Scenario — product status surfaces the framework What/How/delivery graph

**Given** a captured What graph (one context, entity, event, command) and one
delivery slice,
**When** the user runs `product status`,
**Then** the process exits 0 and stdout contains a `Framework graph` section
reporting `1 contexts`, `1 entities`, `1 commands`, and `1 slices`.

## Validates

- FT-125 — product status surfaces the framework What, How, and delivery graph
- ADR-066 — product status summarises the framework graph alongside the legacy spec
