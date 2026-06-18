---
id: ADR-055
title: Task-types (cells) are cross-validated against the captured What graph and How contract
status: accepted
features:
- FT-113
supersedes: []
superseded-by: []
domains:
- api
- data-model
scope: feature-specific
content-hash: sha256:4afb4cc9e930cc72147d517fae495cbb66fdd5b882c32eb034378dafc409bfec
source-files:
- product-core/src/pf/cell.rs
- product-core/src/pf/cell_validate.rs
- product-cli/src/commands/cell.rs
---

## Context

The framework's §5 realisation layer is the **task-type definition** (e.g.
`add-crud-resource`): the dual-read unit declaring `slots`
(dispatch/capture/audit), the **cells** (SPMC work units) it dispatches, and
the **audits** that back them. A cell's defining property is that its frozen
input is *drawn from the rest of the graph* — `derived_from: ["domain:Order",
"app-contract:slice"]` — and it `applies` patterns from the archetype's How
contract. A task type that references a domain concept which does not exist, or
a pattern the How never defines, is a latent lie.

The vendored `task-type-definition.schema.json` validates structure only (it
is JSON Schema); it cannot check that `domain:Order` is a real entity or that
`applies: [result-type]` names a real pattern.

## Decision

Implement task types as a typed model (`pf::cell`) with a native checker
(`pf::cell_validate`) that **cross-validates against the other two graphs**:

1. **Structural (violations):** at least one slot and one audit; every slot
   carries a non-empty inline `audit` (no slot without a backing audit); every
   audit names what it `protects`; every cell declares `derived_from`.
2. **Domain cross-check (warnings):** each cell `derived_from` pointer of the
   form `domain:X` must resolve to a declared domain slot *or*, when the
   captured What graph is available, a real node in it. A dangling `domain:X`
   is surfaced as a warning.
3. **How cross-check (warnings):** each cell `applies` pointer should name a
   pattern or principle in the archetype's How contract.
4. **Reference coherence (warnings):** a bare `derived_from` pointer must name
   a sibling cell or a slot; `slot:`/`behaviour:` pointers must name a slot.

Cross-graph and template/instance-ambiguous checks are **warnings**, not
violations: a task type is a template whose `domain:slot` pointers bind to
concrete entities only at dispatch, and a How contract may be authored
separately. The structural load-bearing rules are violations (exit ≠0).

The CLI (`product cell validate`) loads the task-type file, the default
product's What graph (`.product/author-domain/<product>/`), and the How
contract (`.product/how-contract.yaml`) best-effort, reporting which were
cross-checked.

## Rationale

- The whole point of a cell is that it is built *from the domain model*;
  validating its `domain:` pointers against the captured What graph is what
  makes "cells use the entities" enforceable rather than aspirational.
- Warnings (not hard failures) for cross-graph references keep a task type
  authorable before its What/How are complete, and respect the
  template-vs-instance duality, while still surfacing every dangling pointer.
- A native checker matches the zero-Python-runtime, typed-violation approach
  of the What (ADR-053) and How (ADR-054) layers.

## Rejected alternatives

- **Structural JSON-Schema validation only.** Rejected: it cannot catch a
  `domain:Ghost` pointer or an `applies: [no-such-pattern]` — exactly the
  errors that make a realisation layer drift from the model.
- **Hard-fail on dangling domain/pattern pointers.** Rejected: a task type is
  a reusable template; its `domain:slot` pointers resolve at dispatch, and its
  How may be authored elsewhere. Warnings inform without blocking authoring.

## Test coverage

- TC-930 — validate passes on the conformant example.
- TC-931 — a slot with no inline audit is a blocking violation.
- TC-932 — a dangling `domain:` pointer is cross-checked against the What graph.
- TC-933 — `applies` is cross-checked against the How contract.
- TC-934 — show/list/init.
- TC-935 — validate without a file is a clear error.
