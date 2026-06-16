---
id: FT-122
title: product decider simulate — prove a Decider sound and complete before realisation
phase: 6
status: complete
depends-on:
- FT-121
adrs:
- ADR-062
- ADR-063
tests:
- TC-948
- TC-949
- TC-955
domains:
- api
- data-model
domains-acknowledged:
  ADR-041: Additive — a new `decider simulate` subcommand plus optional logic/scenarios on the decider artifact; nothing existing is removed or deprecated, so no absence TC is required.
  ADR-047: The functional specification lives in this feature's body, not a separate artifact.
  ADR-042: TCs use the reserved `scenario` type only; no new TC type is introduced.
  ADR-050: PAT-001 (slice + adapter) is cited via `patterns:`; no new implementation pattern is introduced.
  ADR-049: Not a context-bundle/template command; no template surface changes.
  ADR-043: The interpreter + scenario runner live in pure `pf::decider_logic`/`pf::decider_sim`; the CLI is a thin BoxResult adapter.
  ADR-048: Reads the decider file; writes nothing.
  ADR-051: Every TC declares `observes:` (exit-code, stdout, stderr) and asserts on those surfaces.
  ADR-018: Two scenario TCs drive the binary through assert_cmd; `pf::decider_sim` carries unit tests over the interpreter. No property or session dimension for a pure interpreter.
  ADR-040: Simulation is the §3.3 before-realisation gate; it composes the Decider artifact, not the verify pipeline.
patterns:
- PAT-001
---

## Description

§3.3 says a Decider "earns its place twice over": *before* realisation it is
simulated against scenarios drawn from the flows, proving the behaviour is sound
and complete before any code exists. This feature adds that simulation. A
Decider gains one authored part — a **guarded state machine** — plus
**scenarios** (the oracle), and `product decider simulate` runs them as pure,
total, deterministic function calls.

## Functional Specification

### The authored logic (`logic:`)

A guarded state machine over a small aggregate state:

- `initial` — the starting state (named fields).
- `evolve` — per event, the state fields it sets (the fold `evolve(state, event)`).
- `decide` — per command, an ordered list of `guards` (each a structured
  predicate `{field, eq|ne|in|exists}` paired with the invariant it
  `else_reject`s) followed by the events it `emit`s. The first failing guard
  rejects with its invariant; otherwise the events are emitted.

### The scenarios (`scenarios:`) — the oracle

Each scenario is `given` (prior events) → `when` (a command) → `then`
(`emit:` events or `reject:` an invariant). Authored once, they are the oracle
consumed twice: here against the interpreter, and (FT-123) against realised code.

### Behaviour

`product decider simulate <name>` evaluates the Decider:

- **Soundness** — every scenario's actual outcome matches its `then`.
- **Completeness** — every handled command is exercised by at least one
  scenario (else behaviour is unspecified for it).

Exits 0 with `sound + complete` when both hold; exits 1 listing each finding.
The interpreter is total: a missing field makes a comparison false, an unknown
command rejects — it never panics.

### Expressions and payloads (ADR-063)

Beyond the structured lifecycle predicates, a guard may be a **CEL** expression
(`expr:`) over `state`/`command`/`event` maps, and events/commands carry
**payloads** (`{event|command, with}`). Assignment values in `evolve.set` and an
emitted event's `with` are literals, or CEL expressions marked by a leading `=`
(e.g. `amount: "=command.amount"`). CEL is non-Turing-complete, so the gate stays
total and deterministic.

## Out of scope

- Checking realised code against the scenarios — that is FT-123 (§6.3). The same
  scenarios are the oracle there.

## Acceptance

- TC-948 — a sound, complete Decider simulates clean (exit 0).
- TC-949 — a scenario whose expectation contradicts the logic fails (exit 1).
- TC-955 — a CEL guard over a payload, with a computed emitted payload, simulates
  sound + complete.
