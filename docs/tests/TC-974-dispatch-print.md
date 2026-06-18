---
id: TC-974
title: dispatch print does not write files
type: scenario
status: passing
validates:
  features:
  - FT-117
  adrs:
  - ADR-059
phase: 6
observes:
- exit-code
- stdout
runner: cargo-test
runner-args: tc_974_dispatch_print_does_not_write_files
---

## Scenario — print mode emits without writing

**Given** a dispatchable task type and bindings,
**When** the user runs `product cell dispatch --print …`,
**Then** the process exits 0, stdout contains the work-unit manifests (e.g.
`# contract-order` with `domain:Order`), and no files are written under
`.product/work-units/`.

## Validates

- FT-117 — product cell dispatch — instantiate a task type into frozen SPMC work units
- ADR-059 — Cell dispatch instantiates a task type into frozen work units bound to real entities
