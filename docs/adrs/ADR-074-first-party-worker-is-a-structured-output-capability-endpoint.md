---
id: ADR-074
title: The first-party worker is a structured-output capability endpoint
status: accepted
features:
- FT-133
supersedes: []
superseded-by: []
domains:
- api
scope: feature-specific
source-files:
- product-core/src/pf/worker.rs
- product-cli/src/commands/worker.rs
---

## Context

The runner abstraction (ADR-072) gave `build` two endpoints: `claude` (a
subprocess agent) and `litellm` (a raw provider completion). The raw completion
prints text; it does not know the SPMC contract or where the artifact goes. We
want our own worker — graph-aware, owning the structured-output contract and the
apply step — as a first-class capability.

## Decision

Add a third endpoint, `worker`, backed by a native SPMC executor:

- **Pure core (`pf::worker`)**: `build_request` (a litellm request forcing a
  JSON file contract), `parse_files` (read `{files:[{path,content}]}`),
  `stub_files` (a deterministic offline artifact), and `apply_files` (write under
  the repo root, refusing absolute/`..` paths).
- **Dispatch**: `endpoint: worker` calls the model when
  `LITELLM_BASE_URL`/`LITELLM_API_KEY` are set and applies the returned files;
  otherwise it writes the stub. This offline stub is what makes the worker
  testable without a live model (decision-cli's `_stub_runner` pattern).
- **Surface**: `product worker run <role> --prompt <ctx>` dispatches directly;
  `build`'s parallel fan-out dispatches `endpoint: worker` units identically, so
  the first-party worker is one capability among `claude`/`litellm`.

## Rationale

- A first-party worker that owns the structured contract + the apply step is the
  native, graph-aware executor — it produces files in the right place against the
  cell's schema, rather than emitting prose we then have to place.
- Making it a capability endpoint (not a special case) means it composes with the
  catalog, role resolution, escalation, and the parallel runner for free.
- The deterministic offline stub keeps the whole path testable and gives a
  no-cost dry mode, while the live path reuses the same litellm seam as ADR-072.

## Rejected alternatives

- **Extend the `litellm` endpoint to write files.** Rejected: that conflates a
  raw completion runner with the SPMC executor; a distinct `worker` endpoint
  keeps each runner's contract sharp.
- **Build a full agentic loop now.** Deferred: a single structured-output pass is
  the useful first worker; agentic read/edit/test cycles can come later (or stay
  on the `claude` endpoint).

## Test coverage

- TC-988 — `worker run coder` writes a stub artifact offline.
- `pf::worker` units: request shape, parse (happy + missing array), deterministic
  stub, apply-under-root, and path-escape refusal.
