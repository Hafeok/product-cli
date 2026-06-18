---
id: ADR-058
title: Work units are validated as frozen SPMC manifests cross-checked against What and How
status: accepted
features:
- FT-116
supersedes: []
superseded-by: []
domains:
- api
- data-model
scope: feature-specific
content-hash: sha256:8e5d28ce985dcb93f55f6a6df1e581619b101ba4a309939754fa61f2b303ea27
source-files:
- product-core/src/pf/work_unit.rs
- product-core/src/pf/work_unit_validate.rs
- product-cli/src/commands/work_unit.rs
---

## Context

The §5 work unit is the smallest reproducible unit of realisation (SPMC): one
bounded transformation with a frozen, declared input (Schema, Prompt, Model,
Context) producing exactly one artifact and carrying a rationale trace. Cells
inside a task type are the template form; a standalone `work-unit.yaml` is the
concrete, dispatchable form whose `context.derived_from` names real domain
concepts (`domain:Task`). Nothing validated these manifests or checked that
their inputs and applied principles actually exist.

## Decision

Implement a typed `WorkUnit` model (`pf::work_unit`) and a native checker
(`pf::work_unit_validate`):

1. **Structural (violations):** a non-empty schema and prompt; a **frozen**
   context (reproducibility depends on it); a non-empty `context.derived_from`;
   exactly one produced artifact.
2. **Domain cross-check (warnings):** each `domain:X` input and `trace.what`
   resolve to a real node in the captured What graph.
3. **How cross-check (warnings):** `applies` and `trace.why` name real How
   patterns/principles; an applied principle the How does not record as
   `enforced_by` a verification surfaces a trace-truth warning (the crown rule,
   a warning here because the verification graph is a separate artifact).

The CLI (`product work-unit validate/show/init`) loads the manifest, the
default product's What graph, and the How contract best-effort, reporting which
were cross-checked.

## Rationale

- A frozen context is the load-bearing SPMC invariant; making it a violation
  (not a warning) is what protects reproducibility.
- The work unit is the concrete form where `domain:` pointers are real entity
  ids (unlike a task-type's slot-bound template), so cross-checking them
  against the What graph is precise and valuable.
- Trace-truth at the work-unit level is a warning because enforcement lives in
  the verification graph; `how validate` / `cell validate` already carry the
  blocking forms within their own scopes.

## Rejected alternatives

- **Structural-only (JSON Schema) validation.** Rejected: it cannot catch a
  `domain:Ghost` input or an `applies: [no-such-pattern]` — the drift a
  realisation manifest is most prone to.
- **Treat an unfrozen context as a warning.** Rejected: reproducibility is the
  defining SPMC guarantee; an unfrozen context is a real defect, not a hint.

## Test coverage

- TC-960 — validate passes on the bundled example.
- TC-961 — an unfrozen context is a blocking violation.
- TC-962 — a `domain:` input is cross-checked against the What graph.
- TC-963 — show / init.
- TC-964 — validate without a file is a clear error.
