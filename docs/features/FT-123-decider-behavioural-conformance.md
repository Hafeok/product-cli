---
id: FT-123
title: product decider conform — check realised code against the Decider oracle
phase: 6
status: complete
depends-on:
- FT-122
adrs:
- ADR-064
tests:
- TC-956
- TC-957
domains:
- api
- data-model
domains-acknowledged:
  ADR-041: Additive — a new `decider conform` subcommand; nothing existing is removed or deprecated, so no absence TC is required.
  ADR-047: The functional specification lives in this feature's body, not a separate artifact.
  ADR-042: TCs use the reserved `scenario` type only; no new TC type is introduced.
  ADR-050: PAT-001 (slice + adapter) is cited via `patterns:`; no new implementation pattern is introduced.
  ADR-049: Not a context-bundle/template command; no template surface changes.
  ADR-043: The comparison lives in pure `pf::decider_conform`; the CLI adapter spawns the runner and handles I/O.
  ADR-048: Reads the decider file + invokes an external runner; writes nothing.
  ADR-051: Every TC declares `observes:` (exit-code, stdout, stderr) and asserts on those surfaces.
  ADR-018: Two scenario TCs drive the binary through assert_cmd with a shell runner; `pf::decider_conform` carries unit tests over the comparison. No property or session dimension.
  ADR-040: Behavioural conformance is a §6.3 verification kind; its oracle is the Decider, derived from the model — it does not touch the verify pipeline's other gates.
patterns:
- PAT-001
---

## Description

§3.3 says the Decider's scenarios are "authored once, in the What, and consumed
twice": before realisation as a simulation (FT-122), and *after* realisation as
the §6.3 behavioural-conformance check. This feature is the second consumption.
`product decider conform` replays the same scenarios against realised code and
requires the realised behaviour to produce outcomes **identical to the Decider's
simulated outcomes** — turning "looks complete" into "computes the same thing."

## Functional Specification

### The runner protocol

The realised code is reached through a **pluggable runner** — any command, in any
language — so the check is language-agnostic. The contract is a one-shot JSON
exchange:

- **stdin** — a JSON array of requests, one per scenario, in order:
  `[{ "given": [<event>...], "when": <command> }, ...]` where an event/command is
  a bare id string or `{event|command, with: {...}}`.
- **stdout** — a JSON array of outcomes, same length and order:
  `[{ "emit": [<event>...] } | { "reject": "<invariant>" }, ...]`.

### Behaviour

`product decider conform <name> --runner "<cmd>"`:

1. Builds the requests from the Decider's scenarios and sends them to the runner.
2. Parses the runner's outcomes.
3. For each scenario, computes the Decider's own simulated outcome (the oracle,
   via `replay`/`decide`) and compares it to the realised outcome — event ids
   **and** payloads, or the rejection invariant, must match exactly.

Exits 0 with `behaviourally conformant` when every scenario matches; exits 1
listing each divergence. A runner that fails, returns the wrong number of
outcomes, or emits non-JSON is a clear error.

## Out of scope

- Building the runner — that is the adopter's realised code; the framework
  defines only the protocol and the comparison.

## Acceptance

- TC-956 — a runner whose outputs match the oracle is behaviourally conformant.
- TC-957 — a runner that diverges on a scenario fails, naming the scenario.
