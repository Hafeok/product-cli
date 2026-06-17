---
id: ADR-068
title: Done is computed from existing verifications plus recorded acceptance
status: accepted
features:
- FT-127
supersedes: []
superseded-by: []
domains:
- api
- data-model
scope: feature-specific
source-files:
- product-core/src/pf/done.rs
- product-core/src/pf/bundle.rs
- product-cli/src/commands/deliverable.rs
- product-cli/src/commands/release.rs
---

## Context

§7.2 says "done" is a verifiable predicate and "progress is computed, not
estimated … the fraction of in-scope elements that pass their verifications,"
and that "done is exactly as honest as the verifications are strong." The
delivery layer (FT-126) had the hierarchy but no predicate. A done predicate
needs verification status over a feature's in-scope What elements — but a full
verification ledger does not yet exist.

## Decision

Compute `done` from the verifications the toolchain can already run, rather than
inventing a ledger:

- **feature_done** (`pf::done`) over a deliverable's slice scope
  (`bundle::covered`):
  - **domain conformance** — each in-scope element has no blocking
    `validate_graph` violation;
  - **behavioural conformance** — each Decider over an in-scope aggregate is
    sound + complete (`decider_sim::simulate`, the §3.3 gate);
  - **acceptance** — each criterion's recorded `status` is `passing`
    (set with `product deliverable accept`).
  `done` = all checks pass; progress = fraction passing.
- **cut_closed** — every in-scope node's directed `dependencies` that exist in
  the graph are also in scope (`bundle::dependencies`).
- **release_done** — all member features done AND the cut is closed.

`done` is a gate: the CLI exits non-zero when not done. Acceptance status is the
only new recorded state (on the deliverable); everything else is recomputed from
the graph each run.

## Rationale

- Reusing the existing checks (domain conformance + Decider simulation) makes
  done meaningful today without a speculative verification subsystem, and keeps
  it "as honest as the verifications are strong" — it claims exactly what was
  checked.
- The closed-cut check is pure graph and needs nothing else; it is the part of
  §7.2 that is fully determinable now, so it is computed exactly.
- Recording acceptance as explicit status (not inferring it) keeps done a
  predicate over recorded verdicts, never a judgement.

## Rejected alternatives

- **A full verification ledger / realisation tracker now.** Deferred: large, and
  not needed to make done meaningful for what is currently verifiable.
- **Fold `decider conform` (post-realisation) verdicts into done.** Deferred: it
  needs persisted verdicts + a runner; the pre-realisation simulation is the
  behavioural gate for now, and the docs say so.
- **Estimate progress.** Rejected by §7.2 — progress is computed from passing
  verifications.

## Test coverage

- TC-969 — pending acceptance blocks done; recording it unblocks.
- TC-975 — release done requires members done + closed cut.
- `pf::done` units: pending vs passing acceptance, an unsound Decider blocking
  done, closed vs open cut, and release_done.
