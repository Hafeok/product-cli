---
id: ADR-034
title: Lifecycle Gate — ADR Acceptance Before Feature Completion
status: accepted
features: []
supersedes: []
superseded-by: []
domains: [data-model]
scope: domain
content-hash: sha256:b887a8ff6f3cbf3415d933f6c996a24cecb73b0c7af8767a0d4870d1d9ef29ee
---

**Context:** On 2026-04-14, three ADRs (ADR-031, ADR-032, ADR-033) were accepted after their linked features (FT-033, FT-034, FT-035) were already marked `complete` with all TCs passing. The decisions were rubber-stamped — accepted as a formality after the code shipped, not as a gate before implementation began.

This violates the purpose of an ADR. An architectural decision record exists to guide implementation. If the decision is reviewed after the code is written, the review cannot influence the implementation. Discovering a contradiction or a better rejected alternative at that point means rework, not guidance. The ADR becomes documentation of what happened rather than a decision that shaped what happened.

The problem is structural, not procedural. No validation rule prevents a feature from reaching `complete` while its governing ADRs are still `proposed`. The lifecycle states of features and ADRs are independent — they can advance in any order. A developer or agent working through `product verify` will happily mark a feature complete without checking whether the linked ADRs have been accepted.

This matters more with ADR-032 (Content Hash Immutability) in the system. ADR-032 computes the content-hash at the moment of acceptance. If acceptance happens after implementation, the hash seals a document that was never reviewed before its decision influenced code. The integrity mechanism protects a decision that was never formally made.

Two enforcement points exist in the current architecture:

1. **`product graph check`** — runs validation rules across the entire graph. Currently checks structural integrity (broken links, cycles, orphans) and domain coverage. Does not check lifecycle ordering.

2. **`product verify FT-XXX`** — the one pipeline command Product owns (ADR-021). Runs TCs, updates feature status, regenerates checklist. Currently does not inspect ADR status before promoting a feature to `complete`.

Both are appropriate enforcement points for different reasons: `graph check` catches the problem at any time (CI, pre-commit, manual runs); `verify` catches it at the exact moment of status transition.

**Decision:** Add a lifecycle ordering invariant: a feature cannot be marked `complete` while any of its linked ADRs have `status: proposed`. Enforce this at two points — as a warning in `product graph check` and as a hard gate in `product verify`.

---

### Lifecycle Ordering Invariant

The invariant is:

```
∀f:Feature, ∀a:ADR where a ∈ f.adrs:
  f.status = "complete" → a.status ≠ "proposed"
```

A feature may be `in-progress` while its ADRs are `proposed` — exploration and prototyping during decision review is legitimate. But `complete` means "done, verified, ready to ship." A feature cannot be done if the decisions governing it have not been formally accepted.

ADRs with `status: accepted`, `status: superseded`, or `status: abandoned` all satisfy the invariant:
- **accepted** — the decision is made, the feature can complete.
- **superseded** — the decision was made and later replaced. The feature was implemented under the original decision; the superseding ADR governs future work.
- **abandoned** — the decision was considered and rejected. The feature proceeds without that particular decision, which is a valid outcome if the link is also removed.

Only `proposed` blocks completion — it means the decision has not been reviewed.

---

### Enforcement: `product graph check` — W017

`product graph check` gains a new warning:

**W017 — Feature complete with proposed ADR:** a feature has `status: complete` (or `in-progress`) and a linked ADR has `status: proposed`.

```
warning[W017]: feature complete but governing ADR not yet accepted
  --> docs/features/FT-034-content-hash-immutability.md
   |
 8 | adrs: [ADR-032]
   |        ^^^^^^^ ADR-032 status is 'proposed'
   |
   = hint: accept the ADR with `product adr status ADR-032 accepted`
           or remove the link if the ADR no longer governs this feature
```

W017 is a warning (exit code 2), not an error (exit code 1). This follows the ADR-009 convention: teams running `graph check` in CI can choose whether to fail on warnings. The warning is informational for teams that have a looser process; it becomes a hard gate for teams that configure CI to fail on any non-zero exit.

The warning fires for both `in-progress` and `complete` because both represent active or finished work governed by an unreviewed decision. The `planned` status does not trigger the warning — linking an ADR to a planned feature is forward-planning, not a lifecycle violation.

---

### Enforcement: `product verify` — Hard Gate

`product verify FT-XXX` gains a pre-verification check. Before running any TCs:

1. Load the feature's `adrs` list.
2. For each linked ADR, check its `status`.
3. If any linked ADR has `status: proposed`, emit E016 and exit with code 1. Do not run TCs. Do not update feature status.

**E016 — Verify blocked by proposed ADR:**

```
error[E016]: cannot verify — governing ADR not yet accepted
  --> docs/features/FT-034-content-hash-immutability.md
   |
   = ADR-032 (Content Hash Immutability Enforcement) has status 'proposed'
   = hint: accept the ADR first: `product adr status ADR-032 accepted`
           or remove the link: `product feature link FT-034 --remove-adr ADR-032`
```

This is the hard gate. `product verify` is the transition point from "work done" to "work recorded as done" (ADR-021). It is the correct place to enforce that the governing decisions are finalized.

The gate applies only to the `proposed` → `complete` transition path. If all TCs pass but a linked ADR is proposed, the feature stays at its current status (`planned` or `in-progress`). The developer must accept the ADR and re-run verify.

---

### Bypass: `--skip-adr-check`

For migration scenarios where features are being retroactively linked to ADRs (e.g., adopting Product on an existing codebase), a `--skip-adr-check` flag on `product verify` suppresses E016:

```bash
product verify FT-034 --skip-adr-check
```

The flag is intentionally verbose and undiscoverable — it should not be the default workflow. It does not suppress W017 in `graph check`.

---

### New Error Codes

| Code | Tier | Description |
|---|---|---|
| E016 | Lifecycle | `product verify` blocked — linked ADR has `status: proposed` |
| W017 | Lifecycle | Feature `in-progress` or `complete` with a `proposed` ADR link |

E016 uses exit code 1 (error). W017 uses exit code 2 (warning-only). Both follow the ADR-009 exit code scheme.

---

### Interaction with ADR-032 (Content Hash Immutability)

ADR-032 computes the content-hash when `product adr status ADR-XXX accepted` is run. This ADR ensures acceptance happens before `product verify` marks the feature complete. Together, they create a clean ordering:

1. Author writes ADR (proposed, no hash)
2. ADR is reviewed and accepted → content-hash written (ADR-032)
3. Feature is implemented
4. `product verify FT-XXX` checks ADR status → all accepted → runs TCs → marks complete
5. Any subsequent tampering with the accepted ADR body is caught by E014 (ADR-032)

Without this ADR, step 2 and step 4 could happen in any order, and the hash would seal an unreviewed document.

---

### Scope

This ADR is `cross-cutting` — it applies to every feature that links to any ADR. The `product verify` gate fires regardless of domain, phase, or feature type. Any feature with a `proposed` ADR link is blocked from completion.

---

**Rationale:**
- The invariant is minimal: only `proposed` blocks completion. `accepted`, `superseded`, and `abandoned` all satisfy it. This avoids false positives from ADRs that were legitimately superseded or abandoned after a feature was linked.
- Warning in `graph check` + hard gate in `verify` provides two levels of enforcement. Teams that run `graph check` in CI catch the violation early. Teams that rely on `verify` as their workflow endpoint catch it at the transition point. Both paths are covered.
- The `in-progress` warning (W017) is advisory because prototyping during decision review is legitimate — you might implement a proof-of-concept to validate an ADR's feasibility. But the warning makes the state visible: the team knows work is proceeding under an unreviewed decision.
- `--skip-adr-check` exists for migration, not for workflow. Adopting Product on a mature codebase means many features are already complete and ADRs may be written retroactively. The flag prevents the migration from being blocked by a lifecycle invariant that didn't exist when the work was done.
- E016 stops verify before running TCs, not after. Running tests and then refusing to record the results would waste CI time and confuse developers who see passing tests but no status update.

**Rejected alternatives:**
- **Hard error in `graph check` instead of warning** — `graph check` runs across the entire graph. Making this a hard error would mean a single proposed ADR on any feature blocks the entire graph check with exit code 1. This is too aggressive for teams with mixed workflow maturity. The warning (exit 2) surfaces the issue; CI policy decides whether to fail on it.
- **Block `in-progress` as well as `complete`** — preventing a feature from starting implementation until all ADRs are accepted. Rejected because exploration during decision review is valuable. A developer may prototype to validate that an ADR's decision is feasible. Blocking `in-progress` would force sequential work: finish all decisions, then start all implementation. Real work is more interleaved than that.
- **Automatic ADR acceptance when verify passes** — if all TCs pass, auto-accept linked proposed ADRs. Rejected because acceptance is a human decision, not a test outcome. Tests validate implementation correctness; acceptance validates decision quality. A perfectly implemented bad decision should not be auto-accepted.
- **Lint rule only, no verify gate** — rely on `graph check` W017 alone with no hard gate in verify. Rejected because warnings are ignorable. The whole point of this ADR is that the process violation happened because nothing prevented it. A warning that can be silently ignored does not prevent it — it only documents it after the fact.
- **Block at `product feature status` instead of `product verify`** — prevent manual status changes to `complete`. Rejected because `product verify` is the canonical status transition path (ADR-021). Manual `feature status complete` is already an escape hatch. Adding a gate to both would be redundant; adding it only to `feature status` would miss the primary path.

**Test coverage:**
- TC-440: `product verify` exits E016 when linked ADR is `proposed` — create a feature linked to a proposed ADR with a passing TC. Run `product verify`. Assert exit code 1, E016 in stderr, feature status unchanged.
- TC-441: `product verify` succeeds when all linked ADRs are `accepted` — same setup but accept the ADR first. Run `product verify`. Assert exit code 0, feature status `complete`.
- TC-442: `product graph check` emits W017 for complete feature with proposed ADR — create a feature with `status: complete` linked to a proposed ADR. Run `product graph check`. Assert W017 in output.
- TC-443: W017 does not fire for `planned` feature with proposed ADR — create a feature with `status: planned` linked to a proposed ADR. Run `product graph check`. Assert no W017.
- TC-444: `--skip-adr-check` bypasses E016 — create a feature linked to a proposed ADR with passing TCs. Run `product verify --skip-adr-check`. Assert feature status updates normally.
- TC-445: Superseded and abandoned ADRs satisfy the invariant — create a feature linked to ADRs with `status: superseded` and `status: abandoned`. Run `product verify`. Assert no E016.
- TC-446: E016 names all proposed ADRs, not just the first — create a feature linked to two proposed ADRs. Run `product verify`. Assert both are named in the E016 output.
- TC-447: Lifecycle gate exit criteria — TC-440 through TC-446 all pass.