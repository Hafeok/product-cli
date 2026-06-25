---
id: FT-151
title: State justification and Decider justification
phase: 1
status: complete
depends-on: []
adrs:
- ADR-093
tests:
- TC-1035
domains: []
domains-acknowledged: {}
---

## Description

Framework §3.3/§3.4 add two model-gap *detectors* over an authored Decider:

- **State justification** — a field belongs on an aggregate iff some Decider
  reads it. An evolved field that no decision reads is therefore a finding: dead
  state, or (more often) an unmodelled invariant — the rule that *would* read it
  has not been written yet.
- **Decider justification** — the mirror: every Decider must *decide*, i.e. have
  at least one reachable rejection (a guard). A Decider that can never reject is
  a trivial command->event relabelling that should have no Decider, or it is
  missing an invariant.

Both are peers to the intent-reliance and data-divergence signals, so they are
emitted as **advisory warnings**, never blocking gates. A new `reads` field on
the Decider lets an author declare which state fields a CEL-guarded decision
consults, so state justification sees them. The `pf:reads`/`pf:rejects` edges are
added to the Decider's Turtle projection (§9).

## Functional Specification

### Inputs

- `product decider validate <id>` — over an authored Decider with `logic`.

### Behaviour

- For a Decider with authored logic, each aggregate field it evolves is checked
  for a reader (a structured guard, a CEL expression, or the `reads` list);
  unread fields are reported.
- A Decider whose logic has no guard anywhere is reported as never rejecting.
- A signature-only Decider (no logic) is a stub and draws no findings.

### Error handling

- The findings are warnings: `decider validate` prints them and still exits 0
  (conformant). Only the §3.3 drift rules (foreign commands, coverage,
  output-alphabet) remain blocking.

## Out of scope

- The projected-field mirror ("every projected field is consumed by a UI step or
  report") — the model projects whole entities, not fields, so field-level
  consumption is not yet representable. Deferred.
- Promoting these from advisory warnings to blocking gates.

## Acceptance

- TC-1035 passes; `cargo t` + clippy green.
