---
id: FT-130
title: product build — the SPMC build orchestrator that records conformance into done
phase: 6
status: complete
depends-on:
- FT-127
adrs:
- ADR-071
tests:
- TC-983
- TC-984
domains:
- api
- data-model
domains-acknowledged:
  ADR-041: Additive — a new `build` command + a recorded conformance verdict; nothing removed, so no absence TC is required.
  ADR-047: The functional specification lives in this feature's body, not a separate artifact.
  ADR-042: TCs use the reserved `scenario` type only; no new TC type is introduced.
  ADR-050: PAT-001 (slice + adapter) is cited via `patterns:`; no new implementation pattern is introduced.
  ADR-049: Not a context-bundle/template command for the FT/ADR graph; it assembles a What-graph SPMC context.
  ADR-043: Assembly lives in pure `pf::build`; the CLI adapter spawns the agent + does I/O.
  ADR-048: Reads the deliverable/slice/How/deciders; writes the frozen context + the conformance verdict.
  ADR-051: Every TC declares `observes:` (exit-code, stdout) and asserts on those surfaces.
  ADR-018: Two scenario TCs drive the binary via assert_cmd (`--dry-run` for assembly; a runner for the conformance loop); `pf::build`/`pf::done` carry unit tests.
  ADR-040: "`build` composes existing slices + gates; the conformance verdict feeds the §7.2 predicate."
patterns:
- PAT-001
---

## Description

This is the new-flow analog of `implement`. Where the old loop spawned an agent
on a prose feature and ran TC runners, the new loop assembles the **SPMC frozen
context** for a delivery feature, spawns an agent to produce the artifact, and
reports the §7.2 `done` verdict — and it closes the honesty gap by recording
behavioural conformance so `done` reflects realised code, not just the spec.

## Functional Specification

### `product build <deliverable> [--dry-run] [--product]`

Assembles the SPMC context (`pf::build::assemble`) from:

- **What** — the deliverable's slice subgraph (the bundle closure);
- **How** — the principles/patterns/contracts to apply by pointer;
- **Behaviour** — the Decider oracle (scenarios) for in-scope aggregates;
- **Acceptance** — the deliverable's criteria.

`--dry-run` prints the context + the gate status (the `deliverable done`
breakdown) without spawning. Live, it persists the context to
`.product/build/<id>.md`, spawns the agent (`claude -p`), then reports the gates.

### Done records behavioural conformance (the honesty fix)

`product decider conform` now **persists** its verdict to
`<name>.conform.json`. `feature_done` reads the set of conformed deciders: an
in-scope Decider must both **simulate** sound + complete (§3.3, pre-realisation)
*and* have a recorded passing **conformance** verdict (§6.3, realised == oracle).
So a deliverable with a Decider is not done until its realised behaviour has been
conformed — done is now exactly as honest as the verifications run.

## Out of scope

- Driving the agent loop to completion / retrying — `build` spawns once and
  reports; orchestrating multiple work units per deliverable is future work.

## Acceptance

- TC-984 — `build --dry-run` assembles the SPMC context (What/How/Behaviour/
  Acceptance) and shows the gate status.
- TC-983 — recording a conformance verdict (`decider conform`) flips a
  Decider-bearing deliverable from not-done to done.
