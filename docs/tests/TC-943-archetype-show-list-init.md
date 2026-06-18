---
id: TC-943
title: archetype show list and init
type: scenario
status: passing
validates:
  features:
  - FT-114
  adrs:
  - ADR-056
phase: 6
observes:
- exit-code
- stdout
runner: cargo-test
runner-args: tc_943_archetype_show_list_and_init
---

## Scenario — scaffold, then inspect

**Given** a repo with no archetypes,
**When** the user runs `product archetype init rest-api`,
**Then** the process exits 0 and writes `how-contract.yaml`, `layout.yaml`,
and `cells/example-task.yaml`; a second `init` without `--force` exits 1;
`archetype validate rest-api` exits 0; and `list`/`show` stdout report the
archetype name.

## Validates

- FT-114 — product archetype — assemble and validate How, layout, and cells as one
- ADR-056 — An archetype assembles How, layout, and cells from a directory and validates the whole
