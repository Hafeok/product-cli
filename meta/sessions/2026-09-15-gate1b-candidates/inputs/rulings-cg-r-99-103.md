# Rulings CG-R-99 … CG-R-103 — run 8

**Issued by Emil, 2026-09-14.**

---

## CG-R-99 — Both falsifiers fired against me. The table cannot move the numerator.

**A came back at 50.7 exact.** No table entry named an in-solution constructed type, so the graph did not expand and the session's reading was right. CG-R-97's stated falsifier fired.

**And the unscored bet was wrong for a better reason than being wrong.** B's CG-R-96 figure is 88.8, not the 70s. The cause is structural:

> **Every table entry names a framework type with no production implementor.** A framework registration registers framework types — that is what makes it a framework registration. So under CG-R-96 the table can only move edges *out* of the denominator, never into the numerator.

I predicted a denominator shrinkage lifting coverage into the 70s. The shrinkage was total: with every table-known edge leaving as `boundary`, the CG-R-96 denominator reduces to resolved + unresolved, and **the figure collapses to the run-6 form exactly**. 88.8 is not a new measurement; it is B's run-6 coverage arriving by another route.

Six-of-six inside bands on both predictions, and the only two claims that could have been wrong were wrong. That is the design working.

---

## CG-R-100 — CG-R-96 is inert on A and B. The table's worth is the error bound.

CG-R-96 is **not withdrawn** — its resolved branch is correct and would fire on a solution where a framework call registers an in-solution type. `AddDbContext<AppContext>` remains the shape. It simply does not occur in either codebase.

**Ruled:**

- CG-R-96 stays in force from run 9, **recorded as inert on A and B**, with the identity stated: on a solution whose table entries are all framework-typed, the CG-R-96 figure and the run-6 form are the same number. Run 9 reports it once and says so, rather than printing two identical columns.
- **The table's contribution is measured by the error bound, not by coverage.** A moved 8.2% → 5.2%, B 7.1% → 6.4%. That is what CG-R-95's work bought, and it is the figure to report against it.

The session identified this before I did and it is the correct reading.

---

## CG-R-101 — The table is the last unvalidated component and needs ground truth

**744 edges in B and 59 in A take their verdict from the table being right about what a framework call registers — written from documentation, checked against nothing.** That is 28% of B's composition edges resting on recollection.

The reader has ground truth. The walk has ground truth, at 65/67 with precision holding at 100% across every run. The table has a fixture entry.

**Ruled: the table is validated the way CG-R-71 validated the reader.** Take the entries by edge count — `Configure` at 220, `AddLogging` at 170, and down the ranking until the covered edges pass a stated share — and verify each against the framework's actual registrations rather than against documentation. Source, or a runtime service-collection dump, not a second reading of the same docs.

Report the table's precision and recall the way the reader's are reported, beside every figure that depends on it.

**This is the last validation the instrument needs.** The criterion is sound at 100% precision, the reader and walk are measured, the proxies carry incidences, the limits carry incidences. The table is the one component whose correctness is asserted.

---

## CG-R-102 — Gates run before measurement

`cargo t` caught the 44-statement-line breach **after** run 8 was measured. The repair was a pure extraction with no figure change and the outputs stand as produced by the binary at `4023741`, which is recorded — so nothing is wrong with run 8.

But the order is wrong, and it is the same class on its sixth appearance: a check that did not run when it mattered.

**Ruled: the run script runs the full gate set before the first measurement, not after.** A measurement produced by a binary that has not passed its gates is provisional until it has, whatever the gate is about. Recording which binary produced the outputs is the mitigation, not the fix.

---

## CG-R-103 — Instrument work closes after table ground truth

**Ruled: after CG-R-101's table validation, no further instrument work is undertaken.**

`AddHttpClient<T>` as a knowledge row is **ratified** and lands with the table validation. `unknown = 0` on both is CG-R-95 discharged against everything reached — correctly stated as an artefact of N's definition, since a third solution will reach calls the table does not know. That is not a reason to keep extending it.

Eight runs have produced a validated instrument and **no finding about either codebase**. The delta has never run. Every remaining question — §12.1's disposition, the reachable/isolated split, whether any of this assesses a codebase — waits on an act vocabulary that is mine and has been outstanding since the PRD.

Continuing to sharpen the instrument past the point where its components are measured would be work I can authorise indefinitely without ever supplying the input that makes it mean anything. The scope closes so that it does not.

---

Register debt: CG-R-17 … CG-R-103. Mine, with the act vocabulary.
