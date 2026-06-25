---
id: ADR-093
title: Justification findings are advisory model-gap detectors not gates
status: accepted
features:
- FT-151
supersedes: []
superseded-by: []
domains: []
scope: domain
---

**Status:** Accepted

**Context:**

Framework §3.3/§3.4 define state justification (an evolved aggregate field no
Decider reads is a finding) and Decider justification (a Decider with no
reachable rejection is a finding). The repo's own deciders (e-viewgraph,
e-boundedcontext) are deliberately guard-less relabellings, and `decider
validate` reverts/blocks on any blocking violation. Making justification
blocking would fail those deciders and any guard-less authored decider.

**Decision:**

Emit justification as **warnings**, not blocking violations, and scope them to
deciders that have authored `logic` (a signature-only derived decider is a stub
and draws nothing). `decider validate` prints the warnings and still exits 0.
Add a `reads` list to the Decider so an author can declare the state a CEL-guarded
decision consults; state justification treats a field as read if a structured
guard names it, a CEL expression mentions it, or it is in `reads` — erring toward
"read" so a used field is never wrongly flagged dead.

**Rationale:**

The spec frames these as *detectors* and *signals* (peers to the intent-reliance
and data-divergence rates), not as the deterministic gates §6 reserves for
oracle-derived checks. A warning surfaces the model gap — "this field is read by
nothing; you probably have an unmodelled invariant" — without blocking work or
invalidating existing deciders, which is exactly how a measurable signal should
behave. The `reads` escape keeps the check honest against CEL guards the
structured matcher cannot parse.

**Rejected alternatives:**

- **Block the build on a justification finding.** Rejected: it would fail the
  repo's own guard-less deciders and contradict the spec's framing of these as
  signals, not gates.
- **Infer reads purely from CEL by parsing expressions.** Rejected as
  over-engineering for now; a substring heuristic plus an explicit `reads` escape
  is enough and never wrongly flags a used field.

**Test coverage:** TC-1035.
