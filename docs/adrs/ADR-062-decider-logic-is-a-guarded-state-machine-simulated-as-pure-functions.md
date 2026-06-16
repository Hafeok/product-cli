---
id: ADR-062
title: Decider logic is a declarative guarded state machine simulated as pure functions
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
- product-core/src/pf/decider_logic.rs
- product-core/src/pf/decider_sim.rs
- product-cli/src/commands/decider.rs
---

## Context

§3.3 requires a Decider to be simulated "sound and complete before any code
exists," and to be the independent oracle the realised code is later checked
against (§6.3). Two consequences follow. First, the decision logic must be
executable by the tool with no realised code present — so a *pointer to realised
functions* is excluded (you cannot call code that does not exist, and an oracle
that is the code cannot judge the code). Second, the logic must be total and
deterministic, or "sound and complete" is undecidable. The logic must therefore
be declarative data the tool interprets.

## Decision

Author the logic as a **guarded state machine**, interpreted by pure functions:

- **State** is a small map of named scalar fields. **`evolve(state, event)`**
  folds an event's `set` assignments into state; **`replay`** folds a sequence.
- **`decide(state, command)`** finds the command's rule, evaluates its `guards`
  in order — each a structured predicate (`eq`/`ne`/`in`/`exists`) paired with
  the invariant it `else_reject`s — and rejects at the first failure, else emits
  the rule's events. The rejection reason *is* an aggregate invariant id, making
  §3.3's "invariants, now executable" a real edge.
- **Scenarios** (`given`/`when`/`then`) are the oracle. `simulate` checks
  soundness (every scenario matches) and completeness (every handled command is
  exercised).

The interpreter is total: a missing field makes a comparison false; an unknown
command rejects. No loops, no I/O, no user functions — it cannot diverge, which
is what makes the gate deterministic.

Guard predicates use the same one-of-optional-fields idiom as the layout model's
rule kinds. The expectation is a struct with optional `emit`/`reject` (not an
untagged enum, which mis-binds `{reject: …}` to an `emit` variant by ignoring the
unknown key).

## Rationale

- A guarded state machine is what an aggregate *is* (placed → paid → shipped;
  you cannot ship an unpaid order); it covers the overwhelming majority of real
  decision logic while staying trivially total and analyzable.
- Totality + determinism are not nice-to-haves: they are what let the simulation
  *prove* soundness/completeness rather than sample it, and what make the same
  scenarios a stable oracle for the post-realisation check (FT-123).
- Keeping payloads and a full expression language out of this increment (YAGNI)
  bounds it to a dependency-free, fully-tested core; richer expressions adopt the
  CEL standard later rather than a bespoke DSL.

## Rejected alternatives

- **Pointer to realised pure functions.** Rejected: breaks the before-realisation
  gate and makes the oracle non-independent.
- **A Turing-complete scripting engine (e.g. rhai) for logic.** Rejected:
  divergence and side effects destroy totality/determinism — the gate could hang
  or differ run-to-run.
- **Untagged enum for the expectation.** Rejected: serde binds `{reject: …}` to
  the first variant, silently ignoring the key; the optional-fields struct is
  unambiguous.

## Test coverage

- TC-948 — a sound, complete Decider simulates clean.
- TC-949 — a contradicting scenario is caught.
- `pf::decider_sim` unit tests cover replay, decide (accept + reject-with-
  invariant), soundness failure, and completeness gaps.
