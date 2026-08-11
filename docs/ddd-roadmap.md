# DDD Roadmap — M8 and the filed follow-ups

Split out of the PRD per `dec/ddd/prd-split`. History (M1–M7, shipped)
lives in the umbrella's history note and the experiment reports; this
document holds only what is still ahead.

## M8 — Enforcement closure

Scope per `dec/ddd/m8-enforcement-closure` (based on `DDD-arch-08`,
`DDD-arch-09`): the Decision Ledger integration **plus the chain that
makes enforcement close over the repository**, not only the governed
path. The acceptance for M8 is the chain, end to end:

> **change → detected contract event → justified obligation →
> declaration signing the change → durable discharge → authoritative
> CI result**

Interception governs the governed path; CI governs the repository — M8
is where the second half becomes true.

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

### Open at M8 planning (name them before writing the M8 prompt)

- **Signing semantics:** whether the signed subject covers the
  post-change content only or the before/after pair, and what
  `base_revision` pins against in a dirty working tree (index? HEAD?
  the interception-time overlay?).
- **Classifier sharing mechanism:** library call (ddd-core grows a
  diff-over-two-revisions entry point) vs. re-serving through the MCP
  surface; CI needs the hostless path or committed host output.
- **Session vs. signature:** whether same-session matching survives as a
  low-latency convenience over the signed form, or is retired.

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
