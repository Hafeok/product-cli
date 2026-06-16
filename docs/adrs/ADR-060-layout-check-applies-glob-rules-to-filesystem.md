---
id: ADR-060
title: Layout conformance applies glob rules to the filesystem with allowlist semantics
status: accepted
features:
- FT-120
supersedes: []
superseded-by: []
domains:
- api
- data-model
scope: feature-specific
source-files:
- product-core/src/pf/layout_check.rs
- product-cli/src/commands/archetype.rs
---

## Context

FT-114 introduced the §4.3 layout model and a checker that validated the model
is *well-formed* (exactly one rule-kind per rule; every rule cites what it
enforces; prohibitions carry rationale). It deliberately did not apply the model
to a real repository tree — "applying layout rules to files is future work."
§6.2 names layout conformance the cheapest gate, to run first; without a
filesystem check the model is documentation, not a gate.

## Decision

Implement `pf::layout_check::check_layout(model, root)` — a pure function that
walks the tree under `root` and applies each rule via the `glob` crate,
returning `Vec<Violation>`. Surface it as `product archetype check <name>`.

Rule semantics:

1. **must_exist** — the glob must match, honouring `cardinality`
   (`exactly 1`/`at least 1`) and the `for_each` "1 per scope" form (exactly one
   match per directory matched by the scope, with `{dir}` substituted).
2. **must_not_exist** — a prohibition: the glob must match nothing.
3. **must_co_exist** — within each directory matching `when`, every `require`
   sibling must exist.
4. **no_orphans** — allowlist semantics: every file under the scope must be
   matched by some `may_exist_here`/`must_exist` glob, so the *unanticipated*
   file is the failure case.

A trailing `/**` is normalised to `/**/*` so the glob engine matches files.

`check` is separate from `validate`: `validate` checks the model is well-formed,
`check` checks the tree conforms to it. The pure walk lives in `product-core`;
the CLI is a thin adapter (PAT-001).

## Rationale

- The `glob` crate is a small, well-understood dependency; path globbing is
  exactly the operation §4.3 rules describe, so the rules map directly to
  queries with no interpretation layer.
- Allowlist semantics (the unanticipated file fails) is the property that makes
  a layout a real boundary rather than a list of known-bad patterns.
- Keeping `check` (filesystem) distinct from `validate` (model) keeps each
  command's contract sharp and lets `check` run as the cheap pre-flight gate.

## Rejected alternatives

- **Fold filesystem enforcement into `validate`.** Rejected: well-formedness of
  the model and conformance of the tree are different questions with different
  inputs; conflating them muddies both exit-code contracts.
- **A bespoke recursive walker instead of `glob`.** Rejected: reinvents a solved
  problem and would drift from familiar glob semantics authors already expect.

## Test coverage

- TC-945 — apply a layout model to the tree: a conformant tree passes; a
  forbidden file fails with the `must_not_exist` violation.
- `pf::layout_check` unit tests cover each rule-kind (must_exist + cardinality +
  1-per-scope, must_not_exist, must_co_exist, no_orphans allowlist) over
  tempdir trees.
