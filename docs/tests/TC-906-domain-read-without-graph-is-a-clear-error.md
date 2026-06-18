---
id: TC-906
title: domain read without graph is a clear error
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
runner-args: tc_906_domain_read_without_graph_is_a_clear_error
---

## Scenario — reading a non-existent graph is a clear error

**Given** a repo with no captured domain graph yet,
**When** the user runs `product domain list`,
**Then** the process exits 1 and stderr explains that no domain graph exists
yet, pointing at `domain new` / `author domain`.

## Validates

- FT-110 — product domain — CLI list, show, and CRUD over the captured What graph
- ADR-053 — Domain authoring is a separate What graph with native in-loop conformance
