---
id: ADR-071
title: build assembles the SPMC context; conformance is recorded into done
status: accepted
features:
- FT-130
supersedes: []
superseded-by: []
domains:
- api
- data-model
scope: feature-specific
source-files:
- product-core/src/pf/build.rs
- product-core/src/pf/done.rs
- product-cli/src/commands/build.rs
- product-cli/src/commands/decider.rs
---

## Context

The framework had all the pieces of the build loop — slice context, the How,
the Decider oracle, the §7.2 `done` predicate — but no single orchestrator
tying them together (the new-flow analog of `implement`). And `done` folded in
the Decider *simulation* (pre-realisation) but not the post-realisation
`decider conform` verdict, because nothing recorded it — so `done` could be true
before the realised behaviour was ever checked against the oracle.

## Decision

Two changes:

1. **`pf::build::assemble`** builds the SPMC frozen context for a deliverable —
   the What slice (bundle closure), the How (principles/patterns/contracts to
   apply by pointer), the Decider oracle (scenarios for in-scope aggregates),
   and the acceptance criteria. `product build <deliverable>` prints it under
   `--dry-run` (with the gate status), or persists it + spawns `claude -p` +
   reports the gates live. `build` is CLI-only (it spawns a process).

2. **Conformance is recorded into `done`.** `product decider conform` writes its
   verdict to `<name>.conform.json`. `feature_done` gains a `conformed` set: an
   in-scope Decider must now both simulate sound + complete (§3.3) *and* be
   recorded conformant (§6.3) — two checks (`behavioural-sim`,
   `behavioural-conform`). A Decider-bearing deliverable is therefore not done
   until its realised behaviour has been conformed.

## Rationale

- One orchestrator makes the loop legible: assemble → (agent) → gates, in a
  single command, mirroring the old `implement` ergonomics.
- Recording the conform verdict is the honest completion of §7.2 — "done is
  exactly as honest as the verifications are strong." Before, done over-claimed
  (spec proven, code unchecked); now it claims only what was verified, including
  realised behaviour.
- Keeping assembly pure (`pf::build`) and the spawn in the adapter follows the
  slice/adapter split; the verdict is a tiny sidecar file, recomputed into `done`
  rather than mutating the deliverable.

## Rejected alternatives

- **Reuse the legacy `implement` for deliverables.** Rejected: it is FT-XXX /
  legacy-graph-centric; the new flow's context is the What slice + How + Decider.
- **Leave `done` simulation-only.** Rejected: that over-claims done before the
  realised code is checked against the oracle — the gap this feature exists to
  close.
- **Run `decider conform` inside `build`.** Deferred: conform needs a per-Decider
  runner; keeping it a separate recorded gate is simpler and composes cleanly.

## Test coverage

- TC-984 — `build --dry-run` assembles the SPMC context + gate status.
- TC-983 — recording a conformance verdict flips a Decider-bearing deliverable to
  done.
- `pf::build` unit (SPMC sections); `pf::done` unit (an in-scope Decider must be
  conformed for done).
