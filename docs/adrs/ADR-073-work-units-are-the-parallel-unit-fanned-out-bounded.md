---
id: ADR-073
title: Work units are the parallel unit, fanned out bounded with a coherence gate
status: accepted
features:
- FT-132
supersedes: []
superseded-by: []
domains:
- api
scope: feature-specific
source-files:
- product-core/src/pf/run.rs
- product-cli/src/commands/build.rs
---

## Context

`build` dispatched one worker for a whole deliverable. To parallelise, the
question is the unit of parallelism. §5 work units are "single-purpose and
bounded… input explicitly declared and frozen, so the same input yields the same
output." That independence is exactly what makes them safe to run concurrently —
the cell is the template (it picks the worker via its role), but the work unit is
the executable, frozen, parallel job.

## Decision

- Add **`pf::run::run_parallel(items, jobs, f)`** — a bounded concurrent map over
  scoped threads (a shared atomic cursor; at most `jobs` running), results in
  input order. Pure + unit-tested, independent of workers.
- `product build --jobs N` loads the work units from `.product/work-units/` and,
  when present, fans them out via `run_parallel`, each dispatched to the resolved
  capability; absent, the deliverable stays a single unit of work. `--dry-run`
  shows the run plan.
- Coherence is gated **after** the fan-out by the existing `done`/coherence
  gates (§6.1) — the framework's guarantee that a split is at least as coherent
  as an unsplit author.

## Rationale

- The work unit is the right granularity: frozen, independent, one artifact —
  embarrassingly parallel, exactly as §5 intends. Parallelising at the cell or
  deliverable level would conflate the template/scope with the executable job.
- A small reusable concurrency primitive (`run_parallel`) keeps the threading in
  one tested place; the CLI only supplies the per-unit work function (worker
  dispatch), which is the part that does I/O.
- Bounding by `--jobs` gives backpressure without an async runtime; scoped
  threads keep it borrow-safe with no `'static` requirement on the work function.

## Rejected alternatives

- **Parallelise per cell or per deliverable.** Rejected: the cell is a template
  (one cell → many units) and the deliverable is a scope; neither is the frozen,
  independent execution unit.
- **An async runtime (tokio) for the fan-out.** Rejected: the work is
  process/HTTP-bound and coarse; scoped threads + a cursor are simpler and need
  no runtime.

## Test coverage

- TC-987 — `build --jobs N --dry-run` lists the parallel plan over the work units.
- `pf::run` units: order-preserving map, every item run once, empty input,
  jobs clamped above the item count.
