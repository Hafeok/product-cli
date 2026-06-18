---
id: ADR-064
title: Behavioural conformance replays the Decider scenarios against a pluggable runner
status: accepted
features:
- FT-123
supersedes: []
superseded-by: []
domains:
- api
- data-model
scope: feature-specific
source-files:
- product-core/src/pf/decider_conform.rs
- product-cli/src/commands/decider.rs
---

## Context

§6.3 requires a behavioural-conformance verification: realised behaviour must
produce identical outputs to the Decider across the same scenarios. §6.2 fixes
the mechanism — `verify(artifact, oracle, criteria)` with the oracle *derived
from the model, never authored in the check*. Here the artifact is realised code,
the oracle is the Decider (§3.3), and the criteria is output identity. The
toolchain must run scenarios against realised code without knowing its language.

## Decision

Add `pf::decider_conform` (pure) and `product decider conform <name> --runner`:

- The realised code is reached through a **pluggable runner** — any command —
  over a one-shot JSON protocol: stdin is a JSON array of `{given, when}`
  requests (one per scenario, in order); stdout is a JSON array of
  `{emit|reject}` outcomes of the same length and order.
- For each scenario the **oracle is recomputed from the Decider** via
  `replay`/`decide` (not read from the authored `then`), and compared to the
  realised outcome — emitted event ids and payloads, or the rejection invariant,
  must match exactly.
- The pure module builds the requests and does the comparison; the CLI adapter
  owns the subprocess I/O (spawn, write stdin, read stdout, parse).

A runner that exits non-zero, returns the wrong count, or emits non-JSON is a
clear error rather than a silent pass.

## Rationale

- A subprocess + JSON protocol is the lingua franca for crossing a
  language boundary; it lets a Rust toolchain check a realised aggregate written
  in any stack, which is what a framework-level check must do.
- Recomputing the oracle from the Decider (rather than trusting the authored
  `then`) is the §6.2 oracle-derivation rule applied literally: the check
  compares realised code to the *model*, so it stays honest even if a scenario's
  `then` were mis-authored (FT-122's simulation is what guarantees `then` matches
  the model; this check does not depend on it).
- Reusing the same scenarios as FT-122 is the "authored once, consumed twice"
  property from §3.3 — one oracle, two gates (before and after realisation).

## Rejected alternatives

- **Compare realised output to the authored `then`.** Rejected: that trusts the
  scenario authoring; deriving the oracle from the Decider is the §6.2 rule and
  is strictly safer.
- **An in-process language-specific harness.** Rejected: it would bind the
  framework tool to one runtime; the subprocess protocol keeps it agnostic.

## Test coverage

- TC-956 — a runner matching the oracle is behaviourally conformant.
- TC-957 — a divergent runner fails, naming the scenario.
- `pf::decider_conform` unit tests cover request building, a matching run, a
  divergent run, and a wrong-count response.
