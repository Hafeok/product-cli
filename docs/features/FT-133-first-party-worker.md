---
id: FT-133
title: first-party worker — a native SPMC executor (endpoint worker)
phase: 6
status: complete
depends-on:
- FT-131
adrs:
- ADR-074
tests:
- TC-988
domains:
- api
domains-acknowledged:
  ADR-041: Additive — a new `worker` endpoint + `worker run`; nothing removed.
  ADR-047: The functional specification lives in this feature's body, not a separate artifact.
  ADR-042: The TC uses the reserved `scenario` type only; no new TC type is introduced.
  ADR-050: PAT-001 (slice + adapter) is cited via `patterns:`; no new implementation pattern is introduced.
  ADR-049: Not a context-bundle/template command; no template surface changes.
  ADR-043: The request/parse/stub/apply logic lives in pure `pf::worker`; the CLI owns the live model call + dispatch.
  ADR-048: Reads the catalog; writes the produced artifacts under the repo root.
  ADR-051: The TC declares `observes:` (exit-code, stdout) and asserts on those surfaces.
  ADR-018: One scenario TC drives the offline worker via assert_cmd; `pf::worker` carries unit tests over request/parse/stub/apply.
  ADR-040: The worker is a capability (`endpoint: worker`); it composes the catalog + run primitives, not the verify pipeline.
patterns:
- PAT-001
---

## Description

Our own worker — a capability with `endpoint: worker` — rather than a `claude`
subprocess or a raw `litellm` completion. It is graph-aware: it asks the model
for **structured file output** and applies the files itself, so it knows the SPMC
contract end to end. Offline (no model configured) it writes a deterministic
**stub** artifact, which makes the runner testable without a live model.

## Functional Specification

### The worker (`pf::worker`)

- `build_request(model, user)` — a litellm chat request forcing JSON output
  against the file contract `{ "files": [{path, content}, …] }`.
- `parse_files(obj)` — read that contract into `ArtifactFile`s.
- `stub_files(prompt)` — a deterministic offline artifact (a `STUB-<hash>.md`
  carrying the frozen context).
- `apply_files(files, root)` — write each file under the repo root, refusing
  absolute paths or `..` escapes.

### Dispatch + command

A capability with `endpoint: worker` dispatches to the first-party worker:
when `LITELLM_BASE_URL`/`LITELLM_API_KEY` are set it calls the model and applies
the returned files; otherwise it writes the stub. `product worker run <role>
--prompt <ctx>` resolves the role → capability and dispatches — and `build`'s
parallel fan-out dispatches `endpoint: worker` units the same way, so the
first-party worker is one runner among `claude`/`litellm`. The `worker init`
seed adds a `code-writer` capability (`endpoint: worker`) and a `coder` role.

## Out of scope

- A real agentic tool-use loop (read/edit/test cycles) — this worker is a single
  structured-output pass; the `claude` endpoint remains for agentic work.
- Self-gating inside the worker — `build` runs the §6 gates after dispatch.

## Acceptance

- TC-988 — `worker run coder` with no model configured writes a stub artifact
  offline.
