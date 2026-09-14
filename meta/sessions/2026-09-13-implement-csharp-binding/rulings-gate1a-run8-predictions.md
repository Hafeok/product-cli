# Rulings CG-R-96 … CG-R-98 — run 8

**Issued by Emil, 2026-09-14.**

---

## CG-R-96 — A table-known edge is not `registration-not-read`. Effective run 9.

`registration-not-read` means *a registration exists and the reader cannot read it*. **A table entry is reading it.** Once the table states what a call registers, the edge has a verdict: it resolves to the named type, and where that type is external with no in-solution implementation, it is `boundary`.

As the rules currently stand, CG-R-87's host-provider entries keep table-known edges in `registration-not-read`, which is inside the denominator. **That makes the table incapable of moving coverage at all** — every entry added under CG-R-95 explains an edge without changing its verdict. The work would be bookkeeping, and CG-R-95's ranking exercise would be pointless by construction.

**Ruled:**

- From **run 9**, an edge supplied by a call the table knows leaves `registration-not-read` and takes the verdict the entry implies: resolved to the named type, or `boundary` where that type is external and unimplemented in-solution.
- `registration-not-read` is reserved for calls the reader cannot parse **and** the table does not know.
- **Not applied to run 8.** Predictions are committed and CG-R-83's precedent holds: the denominator does not move after predictions. Run 8 reports both figures — the current rule and this one — with the second unscored.

The second figure is the one that says whether CG-R-95's table work was worth doing. Under the current rule, run 8 cannot answer that question.

---

## CG-R-97 — Prediction committed, against the rules as they stand for run 8

| | Scored fraction | Classified | Coverage, not-read inside |
|---|---|---|---|
| **A** | 50.7–52.5, centre **51.0** | 94–97, centre **95.5** | 47–51, centre **49.0** |
| **B** | 65.0–66.5, centre **65.6** | 93–95.5, centre **94.2** | 61–64, centre **62.5** |

**Where I disagree with the session, and why.**

**A's scored fraction is not exact.** The session says it cannot move because the table relabels boundary edges and changes no walk decision. That holds only if no newly-known entry names an **in-solution** type that the container then constructs. If one does — `AddDbContext<CatalogContext>` is the shape to watch — the constructed type opens its own dependencies and the graph expands exactly as O-18 expanded it in run 7. That is CG-R-92's property, and it is the mechanism I missed last time rather than one I am inventing now.

**Stated falsifier:** if A's scored fraction comes back at 50.7 exact, no entry named an in-solution constructed type and the session's reading was right.

Elsewhere I am close to the session, deliberately. Under the current rule the table explains edges without moving their verdicts, so little should move. **That agreement is itself the evidence for CG-R-96**: two independent predictions both expecting the table to change almost nothing is a statement about the rule, not about the table.

**The figure I would actually bet on is the second one.** Under CG-R-96's rule, B's coverage should rise substantially — `Configure` at 220 edges and `AddLogging` at 170 resolve to framework types with no in-solution implementation and leave the denominator as `boundary`. I expect B's CG-R-96 figure in the **70s**, against 62.4% under the current rule. It is unscored, and it is the number that says whether the table earned its keep.

---

## CG-R-98 — A command that can terminate its own session is not run unscoped

The `pkill` pattern matching its own shell is correctly recorded and correctly classed with CG-R-88: a step that did not run.

It is process hygiene rather than an instrument defect, and the distinction matters — nothing was measured wrongly, a run was interrupted. But the class is the same one now on its fifth appearance, and the rule is cheap:

> **A termination command is scoped to specific process ids obtained by a prior query, never to a pattern that could match the session issuing it.**

Recording it rather than quietly restarting is the reason it can be ruled at all, and that is the fifth time a person re-reading caught what no instrument did.

---

## Also

**N stated as the whole run-7 unknown set** — 14 in A, 43 in B, ranked by frequency, each entry listing what the call registers or explicitly empty — is CG-R-95 discharged properly. Entries that register nothing injectable being *entries* rather than omissions is the part that matters: an empty entry is a verdict, an absent one is ignorance.

**Four registration figures** — parsed, known, opaque, unknown — with `opaque` named for lifetime calls whose arguments are variables the reader cannot type, is better than the two CG-R-94 asked for. Opaque is a distinct state and was previously hidden inside unknown.

**The stated-limits list printing on every report with measured incidences** is CG-R-77 generalised from proxies to limits, which is where it should have been from the start.

---

Gate 1b's act vocabulary remains mine and blocks §12.1. Register debt: CG-R-17 … CG-R-98.
