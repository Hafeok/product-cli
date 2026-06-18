---
id: ADR-075
title: Work units dispatch in dependency order, with upstream artifacts injected as frozen inputs
status: accepted
features:
- FT-132
supersedes: []
superseded-by: []
domains:
- api
scope: feature-specific
content-hash: sha256:5bd3c9f2c23474c839c9203ac982f96bda8cc5ff8de804ec46f50ec2a0e6eef9
source-files:
- product-core/src/pf/schedule.rs
- product-cli/src/commands/build.rs
---

## Context

ADR-073 made the work unit the parallel unit and fanned every unit out at once,
on the premise that units are "embarrassingly parallel". That holds only when the
units are genuinely independent. The moment one unit's output is another unit's
input — the canonical case being a `write-test` unit that produces the acceptance
test a later `implement` unit must satisfy — a flat fan-out is wrong: it can run
the implementer before the test it depends on exists.

Work units already declare their inputs in `context.derived_from` (§5: "input
explicitly declared and frozen"). That declaration is a dependency edge; the build
was simply ignoring it for ordering.

## Decision

- Add **`pf::schedule::layers(units)`** — a pure topological grouping. A unit
  depends on another when its `derived_from` names that unit (by work-unit id or
  originating cell id). `layers` returns the units in dependency layers: parallel
  within a layer, sequenced across layers. Cycles and unknown references release
  into a final layer so a build never deadlocks on a stray pointer.
- `build` dispatches layer by layer. Within a layer it still uses
  `run_parallel` (ADR-073); between layers it (a) **freezes** each oracle artifact
  produced so far (`git add`, so the guard of ADR-076 can restore it) and (b)
  injects a completed unit's **declared artifact into its dependents' prompts as a
  READ-ONLY frozen input**. So an `implement` unit sees the exact test it must
  satisfy without being able to edit it.
- The coherence/`done` gates (§6.1) still run after the whole fan-out, unchanged.

## Rationale

- `derived_from` is already the SPMC frozen-input declaration; honouring it for
  scheduling makes the build do what the model always claimed. No new schema.
- Layering keeps ADR-073's parallelism where it is valid (within a layer) and adds
  ordering only where a real edge demands it — the cost is paid only by units that
  actually depend on each other.
- Injecting the upstream artifact read-only is what lets a *test-first* pipeline
  exist inside the existing machinery: one cell writes the oracle, the next
  satisfies it, separation of duties enforced by the schedule plus ADR-076.

## Rejected alternatives

- **Keep the flat fan-out and require authors to sequence builds by hand.**
  Rejected: the dependency is already declared in the data; making the human
  re-encode it as build ordering is the error the graph exists to remove.
- **A full async dependency executor.** Rejected: layered `run_parallel` over the
  topological grouping is sufficient for the coarse, process/HTTP-bound work and
  needs no runtime (consistent with ADR-073).

## Test coverage

- `pf::schedule` units: an `implement` unit layers after the `write-test` unit it
  derives from; independent units share one layer; a cycle releases instead of
  deadlocking.
- End-to-end: a `write-test -> implement` deliverable built by a single worker
  model produces the test, freezes it, and implements against it read-only — green
  with no escalation.
