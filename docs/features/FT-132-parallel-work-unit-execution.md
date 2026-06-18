---
id: FT-132
title: parallel work-unit execution — build fans units across workers
phase: 6
status: complete
depends-on:
- FT-131
adrs:
- ADR-073
tests:
- TC-987
domains:
- api
domains-acknowledged:
  ADR-041: Additive — a `--jobs` parallel path in `build` + a concurrency primitive; nothing removed.
  ADR-047: The functional specification lives in this feature's body, not a separate artifact.
  ADR-042: The TC uses the reserved `scenario` type only; no new TC type is introduced.
  ADR-050: PAT-001 (slice + adapter) is cited via `patterns:`; no new implementation pattern is introduced.
  ADR-049: Not a context-bundle/template command; no template surface changes.
  ADR-043: The bounded-concurrency primitive lives in pure `pf::run`; the CLI owns the live worker dispatch.
  ADR-048: Reads `.product/work-units/`; writes the frozen build context; dispatch is live.
  ADR-051: The TC declares `observes:` (exit-code, stdout) and asserts on those surfaces.
  ADR-018: One scenario TC drives `build --jobs --dry-run`; `pf::run` carries unit tests over the concurrency primitive.
  ADR-040: Parallelism composes the §5 work units; the §6.1 coherence bar gates the fan-out afterwards.
patterns:
- PAT-001
---

## Description

The §5 work unit is the parallel unit: frozen input + one artifact each makes
work units independent by construction, so they run concurrently. `build` now
fans a deliverable's work units (from `cell dispatch`) across workers — each its
own capability instance — bounded by `--jobs`, then gates coherence (§6.1) +
`done`.

## Functional Specification

### The concurrency primitive (`pf::run::run_parallel`)

`run_parallel(items, jobs, f)` maps `f` over `items` with at most `jobs`
concurrent invocations (a shared cursor over scoped threads), preserving input
order in the results. Pure and reusable.

### `product build <deliverable> [--role …] [--jobs N]`

- Loads the work units in `.product/work-units/`. When present, they are the
  parallel units; when absent, the deliverable is a single unit of work
  (unchanged behaviour).
- `--dry-run` prints the **parallel run plan** — `N job(s) over K work unit(s)`
  and each unit → its resolved capability — alongside the SPMC context + gate
  status, without dispatching.
- Live, it fans the units out via `run_parallel`, each dispatched to the resolved
  worker (the cell's role → capability), then reports successes/failures and the
  `done` gates. The §6.1 coherence bar is the gate that makes the split safe.

### Behaviour

When work units are present, `build` fans them across at most `--jobs` workers
via `run_parallel`, each unit dispatched to its resolved capability; results
preserve input order and coherence is gated afterwards (§6.1). With no work units
the deliverable is a single unit of work.

### Error handling

- A work unit whose dispatch fails is reported per-unit (which unit, why) and
  does not abort its siblings; the run summarises how many of N succeeded.
- `--jobs` is bounded to a sane minimum of one; the gate status reflects any
  units that did not complete.

## Out of scope

- Per-unit role/escalation (all units in one build currently share the role's
  capability; per-unit triggers are a follow-on).
- Cross-cell pipelines (implement → verify with different workers) — a later
  orchestration layer.

## Acceptance

- TC-987 — `build --jobs N --dry-run` lists the parallel run plan over the
  seeded work units.
