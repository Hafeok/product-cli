# Session prompt — C-track, session 1: factoring gate and claim model

**Track:** C (claim register core)
**Milestones in scope:** C0 (factoring gate), C1 (claim model and event log) — C1 scaffolding only if
C0 is ratified in-session.
**Dated:** 2026-08-26
**Commit this prompt to `meta/sessions/` before any substantive work** (`DDD-dec-20`).

---

## Standing rules — these bind the whole session

1. **You propose; Emil ratifies via merge.** Nothing in this session is ratified by you. Do not merge,
   do not force-push, do not close an open item.
2. **Three-register discipline.** Rulings, proposals and open items are always distinguished.
   Proposed content is marked `[PROPOSED]`. Nothing tagged `[PROPOSED]` is milestone-acceptance
   content until it clears the ratification queue.
3. **Falsifier-first.** Every claim you file carries a pre-registered falsifier. A claim without one
   is a canon integrity defect, not a rough edge.
4. **Supersession never rewrites.** Errors are corrected by filing a superseding record with a note
   appended to the existing one.
5. **British spelling throughout.**
6. **No legal content of any kind.** No provisions, penalties, thresholds, limits, statutory
   citations or Danish legal vocabulary — not in code, not in tests, not in comments. This is the
   corpus-independent core and it must be possible to read the whole crate without knowing the
   S-track exists.
7. **No corpus-specific types.** No statutory types, no C# types, no ELI vocabulary. If you find
   yourself reaching for one, that is the finding — record it against FC-1 and stop, do not work
   around it.
8. **No customer-identifying material** in any file or commit message.
9. **PRD is canon; session output is draft** until committed and merged.

---

## Gate 0 — live state, no writes

Before any canonical act, fetch and report actual repository state. Do not work from assumptions
about the workspace, and do not write anything in this gate.

Report:

- `Hafeok/product-cli` workspace layout: crates, their dependency edges, and which are published.
- Rust edition and MSRV; the lint and formatting configuration actually in force.
- Test conventions in use — where tests live, whether fixtures are used, what the CI runs.
- Whether any existing crate already carries part of the claim model, event log or refusal
  machinery. If one does, say so plainly; it changes C0.
- Any existing `meta/sessions/` convention to match.

**Stop and report before Gate 1.**

---

## Gate 1 — commit this prompt

Commit this file to `meta/sessions/` per `DDD-dec-20`. No other changes in this commit.

---

## Gate 2 — C0, the factoring boundary

This is the milestone that matters. The sibling ruling created a shared core and named its failure
mode: whichever corpus is encoded first shapes the core, and the separation becomes nominal. C0 is
where that is prevented or lost.

**Produce a written boundary.** Every candidate element assigned **in-core** or **in-track**, with a
reason for each. Candidates, from PRD §4 and Annex A A-6, none of them ratified:

- the claim model (r2 §7.1 fields)
- the three transcription boundaries and per-boundary ratification
- the event log and its projections
- write-time refusals
- supersession and staleness propagation
- evidence binding and three-state discharge
- coverage and gap queries
- the mutation harness
- the embodiment authority-grade field (C-O-6 — suspect this one; it may be an ELI-ism)

**Method constraint, and it is binding.** Factor **from the claim schema**, not by extraction from
either track. The schema predates examination of either corpus. Extraction from a shipped track is
forbidden as a factoring method — that is how siblings become forks.

**Also in Gate 2: name and sketch the two synthetic corpora** (C-O-2). They must be unlike both
statute and C#, and they are the falsifier for the factoring (FC-1), so a lazy choice disables the
check. Candidates offered, not imposed: a domain with numeric bands and delegation to a subordinate
authority; a domain with prose-only obligations and no executable target. Argue for or against these
and propose your own if better.

**Deliverable:** a `[PROPOSED]` boundary document with a reason per element, the two corpora
sketched, and FC-1 stated in a form that can run on every commit.

**Stop. Do not write code. Emil ratifies the boundary before C1 begins.**

---

## Gate 3 — C1 scaffolding, only if Gate 2 is ratified in-session

If and only if the boundary is ratified:

- Propose crate name and workspace placement (C-O-5).
- Propose the domain types for the ratified in-core elements. Types and signatures; no
  implementation.
- Propose the event set: which acts are events, and what each records.

**Stop and report.**

---

## Gate 4 — C1 refusals

Implement the §4.2 refusals, each with a test that asserts the refusal fires:

- incomplete governance record → refused
- model identity as accepting principal → refused (the L006 pattern)
- one signature discharging two boundaries → refused
- rewrite of a prior record → refused, supersession only
- absent residue → refused (empty is a statement; absent is a gap)

**FC-2 is the falsifier and it is adversarial: a claim must not reach the graph incomplete by *any*
path, including test helpers and builders.** Write at least one test that tries to get round your own
constructors.

---

## Gate 5 — session close

Report, in the three registers:

- what is **proposed** and awaiting ratification;
- what is **open**, including anything newly opened;
- what you **found** that contradicts the PRD — especially any FC-1 hit, which is more valuable than
  a completed milestone and must not be smoothed over.

Do not summarise favourably. If C0 could not be done from the schema without reaching for a corpus,
say so; that is the sibling ruling failing its own test and Emil needs it unvarnished.

---

## What is explicitly out of scope this session

Any corpus. Any ingest normalisation (Boundary 0 is corpus-specific). Any reasoner, DSL, NLG or
retrieval surface. Anything touching the S-track's gates S0.1, S0.2 or S0.3. Any Retsinformation
access. The mutation harness (C5) — it is the point of the track, and it is not this session.

---

## Session note — filing location

`DDD-dec-20` directs this prompt to `meta/sessions/`. That directory does not exist in
`Hafeok/product-cli` and no `meta/` convention is in use here; the local convention for track
session material is `docs/<track>/` (see `docs/g-track/`). Filed to `docs/c-track/` on the
principal's direction, 2026-08-26. Whether `DDD-dec-20` is repo-general — in which case
`docs/g-track/` is already a standing deviation — is an upstream item, not resolved here.
