# DDD Roadmap — M8 and the filed follow-ups

Split out of the PRD per `dec/ddd/prd-split`. History (M1–M7, shipped)
lives in the umbrella's history note and the experiment reports; this
document holds only what is still ahead.

## M8 — Enforcement closure. SHIPPED 2026-08-12.

Delivered per `dec/ddd/m8-enforcement-closure`; the report is
[`ddd-m8-report.md`](ddd-m8-report.md), the invariant table as it
actually stands is the spec §2. The chain, end to end, is the shipped
acceptance:

> **change → detected contract event → justified obligation →
> declaration signing the change → durable discharge → authoritative
> CI result**

Interception governs the governed path; CI governs the repository — as
of M8 both halves are true (spec invariant 5 carries one named
residual: direct pushes to the default branch sit outside the
pull-request gate). The section below records the scope as planned;
deviations are in the report, not silently reconciled here.

### Scope as planned (2026-08-11)

### Components

1. **Ledger integration** (record substrate, `decision-ledger-prd.md`
   OD-2): `.ddd/decisions/` migrates to `dec:` ids (bootstrap form
   retired); manifest entries, seam declarations, and risk acceptances
   become ledger entries with typed `DischargeRef`s (`analyzer:ID`,
   `whatif:assertion`, `otel:metric+expectation`); readiness/completeness
   gates delegate to `ledger verify`; instruments computed by the ledger,
   surfaced through `ddd report`/`ddd render`. Acceptance discipline
   inherited: `accepted-by` never resolves to a model identity.
2. **Content-hash basis pins** (pulled forward into M8 with the chain,
   per the ruling): basis loss becomes pinned-hash ≠ current-hash,
   mechanically exact, retiring the status+date heuristic the spec
   names at invariant 2.
3. **Repository-diff contract-surface validation in CI**, sharing the
   interception classifier (spec invariant 4 — one classifier, two
   consumers). A contract-surface change in a diff with no covering
   declaration is a CI finding regardless of how the edit was made
   (invariant 5).
4. **Declaration signing** — a declaration signs the change it
   discharges: subject symbol, before/after content hash, base revision
   (the ledger's acceptance-signs-hash law applied to seams; spec
   invariant 3, closing `DDD-arch-09`).

### The rulings that settled the open questions (Emil, 2026-08-12)

- **Signing semantics:** the before/after pair plus base revision — a
  declaration discharges a transition, not a state. Dirty working tree
  refuses to bind; pins are against HEAD, never uncommitted state.
- **Classifier sharing mechanism:** a library entry point
  (`ddd_lsp::revdiff`), hostless where the adapters support it — MCP is
  a path, not a boundary; routing the authoritative gate back through
  it would rebuild the flaw (spec invariant 4: one classifier).
- **Session vs. signature:** retired, not layered — a convenience path
  admitting the exact defect the milestone closes is a governed-path
  bypass by design.

## Filed follow-ups (independent of M8)

- **Rule-state / STALE rework** — implement the five-state model
  (available / configured / executed / emitted / governed) the spec
  defines in §6, retiring the M2 conflation of "not installed" with
  "did not fire." Bounded task, no ledger dependency, schedulable
  independently.
- **Suppression expiry** — a suppression's risk-acceptance citation
  gains an expiry; `report escapes` flags expired suppressions like
  cadence violations.
- **Adoption baselines** — `init` on an existing repo produces a
  baseline set rather than flooding `UNGOVERNED`; new findings measure
  against the baseline.

## L4 inheritance bar (owned by the ledger track)

`shared/` promotion stays **import** until the ledger's federation layer
(L4) provides real inheritance. The bar, per the review, verbatim:
origin identity and provenance; pinned upstream version; update
detection; local override rules; divergence handling; basis-loss or
revocation propagation. L4's upstreams manifest (§9.4: SHA pins,
lockfile, basis-cone loading, drift as basis loss) is the design that
meets it.
