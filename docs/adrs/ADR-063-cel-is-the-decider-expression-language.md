---
id: ADR-063
title: CEL is the Decider expression language for guards and assignments
status: accepted
features:
- FT-122
supersedes: []
superseded-by: []
domains:
- api
- data-model
scope: feature-specific
source-files:
- product-core/src/pf/decider_cel.rs
- product-core/src/pf/decider_logic.rs
- product-core/src/pf/decider_sim.rs
---

## Context

Stage 1 (ADR-062) gave Deciders a lifecycle state machine with structured
predicates (`eq`/`ne`/`in`/`exists`) over state. Real decision logic also needs
to read command/event **payloads** and compute values (a charge ≤ a limit; an
emitted event carrying an amount). That requires an expression language. §4.4 of
the framework is explicit: where a standard exists, use it — reinventing forfeits
the ecosystem. Guard/value expressions are exactly such a surface.

## Decision

Adopt **CEL (Common Expression Language)** via the `cel-interpreter` crate as the
expression layer, rather than growing a bespoke predicate DSL:

- A guard is either a structured `when` predicate (Stage 1) **or** a CEL `expr`
  (a boolean over `state`, `command`, `event` maps).
- An assignment value (`evolve.set`, an emitted event's `with`) is a literal
  scalar, **or** a CEL expression marked by a leading `=` (e.g.
  `amount: "=command.amount"`). The `=` sigil keeps literals literal and is
  backward-compatible with Stage 1 logic.
- Events and commands carry payloads (`{event|command, with}`), available to
  expressions; scenarios assert emitted payloads, not just event ids.

CEL values are built directly from our `Scalar` (integers stay `Int`, not the
`UInt` that serde-sourcing produces — which would break arithmetic against `Int`
literals) and converted back. Evaluation is **total**: a guard that fails to
compile or does not return a boolean is `false` (the command is rejected); a
value expression error surfaces as a scenario finding. CEL is non-Turing-complete
and side-effect-free, so this preserves the determinism the simulation gate
depends on.

## Rationale

- CEL is the industry standard for safe, embeddable policy/guard expressions
  (used across Kubernetes, Envoy, etc.); using it is the framework practising its
  own §4.4 "use the standard" rule.
- Non-Turing-completeness is the key property: unlike a general scripting engine
  (rhai, Lua), CEL cannot loop or diverge, so a guard cannot hang the gate or
  vary run-to-run — exactly what ADR-062 requires of the interpreter.
- The `=` sigil avoids the unsolvable ambiguity of distinguishing a literal
  string from an expression string in YAML, with zero ceremony for the common
  literal case.

## Rejected alternatives

- **A bespoke expression DSL.** Rejected by §4.4: it would reinvent CEL's
  grammar, evaluation, and tooling, and drift from a known standard.
- **A Turing-complete scripting engine.** Rejected: side effects and unbounded
  loops destroy totality/determinism.
- **Sourcing CEL values through serde_json.** Rejected: positive integers become
  CEL `UInt`, which cannot be combined with `Int` literals in arithmetic; building
  `Value` from `Scalar` directly keeps the numeric type predictable.

## Test coverage

- TC-955 — a Decider with a CEL guard over command payload and an emitted event
  carrying a computed payload simulates sound + complete.
- `pf::decider_cel` unit tests cover boolean guards, integer arithmetic (Int not
  UInt), the literal-vs-`=`-expression split, and graceful failure of bad
  expressions.
