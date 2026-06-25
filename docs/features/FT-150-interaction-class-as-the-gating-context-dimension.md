---
id: FT-150
title: Interaction class as the gating context dimension
phase: 1
status: complete
depends-on: []
adrs:
- ADR-092
tests:
- TC-1034
domains: []
domains-acknowledged: {}
---

## Description

Framework §3.2.2 makes the **interaction class** (GUI / TUI) the *senior*
context-of-use dimension: it is chosen first and determines which other
dimensions are even meaningful (GUI brings form factor + pointer/touch; TUI
brings terminal capabilities and rules form factor out). This feature recognises
the closed-core class set (`gui`, `tui`) and validates it as a context
dimension: a `System` declares the interaction classes it targets (the
`target_classes` field from FT-148), and a `ContextOfUse` may declare the
`interaction-class` dimension naming one of the recognised classes. Platform
stays an open dimension (ios/android/web).

This establishes the dimension the How's reification rules are written against
and sets up the per-class reification coverage that the `unreifiable_in` escape
hatch (a later feature, §4.5) completes.

## Functional Specification

### Inputs

- `product domain new system <id> … --target-classes gui,tui`
- `product domain new context-of-use <id> --dimension interaction-class --value <gui|tui>`

### Behaviour

- A system's `target_classes` and a context-of-use declaring the
  `interaction-class` dimension are accepted when they name a recognised class.
- The classes are emitted as `pf:targetsClass` on the system in the export.

### Error handling

- A system target class, or an `interaction-class` context value, that is not a
  recognised core class (gui/tui) is rejected (`§3.2.2`).

## Out of scope

- A registration mechanism for adopter-defined classes beyond the core (the spec
  notes voice-only as a future class); the core is treated as closed for now,
  mirroring how `CORE_AIOS` is closed pending an `Aio` registration node.
- Per-class reification coverage and the `unreifiable_in` recorded-gap rule —
  deferred to the §4.5 feature, which consumes this dimension.

## Acceptance

- TC-1034 passes; `cargo t` + clippy green.
