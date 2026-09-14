# Rulings CG-R-76 … CG-R-80 — ground truth and the proxy repairs

**Issued by Emil, 2026-09-14.** P-5, P-6 and P-7 are ruled on the principle they embody; I have their descriptions and extents, not their full text. Where a specific repair diverges from the principle below, it is proposed again rather than applied.

---

## CG-R-76 — Ground truth accepted. Precision validates the criterion; recall is the problem.

**Precision 46 of 46 is the most consequential figure yet produced.** Nothing the walk called a composition edge failed the criterion, so the criterion itself is sound and every remaining defect is recall. That is a better position than the reverse and it should be said plainly rather than buried under the recall misses.

The three named recall causes — container-constructed read too narrowly, chained-builder registrations unread, framework activation conventions absent from the roots — are repairs, and the v4 work addresses them.

**Recall figures are scoped, and the scope is stated wherever they appear.** *Reader recall 55/56* is over C# source. It is not recall over composition sites: Razor views are not in either inventory, so the denominator excluded a whole class of site. Report it as *55/56 over C# source; Razor views not covered (see CG-R-78)*.

---

## CG-R-77 — A declared divergence carries its measured incidence

This is the ruling the proxy re-examination earns, and it generalises past this instrument.

The data-contract proxy's declared divergence is **181 of 181** edges. The factory-provider's is **231 of 346**.

**A proxy whose declared divergence covers its field is not a proxy. It is a misclassification with a caveat attached.** A divergence exists to bound the cases where proxy and predicate part company. If they part company everywhere, the proxy does not approximate the predicate at all, and declaring the divergence does not repair that — it only makes the failure look disciplined.

**Ruled: every declared divergence carries its measured incidence on the field.**

Not *this may misclassify X*. That sentence is compatible with 0 of 181 and with 181 of 181, and those are opposite facts. The form is *this misclassifies X, measured at n of m on this field, at this date*.

**Applies retroactively to every proxy in the programme**, not only in this instrument: the CG-R-52 declarable/unstructured separator, the binding's DP-5 proxy fields, and any proxy in the conformance criterion. Each states its incidence or states that its incidence is unmeasured — and *unmeasured* is a materially weaker claim than a measured small number, which is the distinction that has been missing.

**P-5, P-6 and P-7 are ratified on this principle.** Each repaired proxy ships with its divergence and its measured incidence, and the collection-injection misread (76 edges) is a distinct classification rather than a divergence of the factory-provider proxy — a caveat cannot absorb a different category.

**Predictions correctly deferred.** A repair moving a third of B's composition edges before the prediction would have voided it on arrival.

---

## CG-R-78 — R-1: Razor views are out of scope, with the extent stated

A Razor `@inject` is property injection and therefore a composition edge by the criterion. The instrument cannot see it: no generated view class exists in either inventory, putting 1 edge in A and 342 directives in B outside what the reader reads.

**Ruled: out of scope, declared, not papered over.**

- Razor views are stated as a **known blind spot** with the extent: 1 known miss in A, 342 directives in B.
- **No recall figure is reported without that scope**, per CG-R-76.
- Syntactic `@inject` extraction is **refused for now**: it would produce edges whose source is not a type in the inventory, which breaks the graph model to close a gap that is better closed by compiling views into the inventory. Recorded as the candidate repair.

The honest statement is that B's composition graph is missing a class of site roughly a quarter the size of its scored denominator. That belongs beside B's headline figure, not in a footnote.

---

## CG-R-79 — The registration-knowledge table is measured by its coverage of reached calls

The weakest point is correctly identified and it is the same class as everything else: a table written from documentation memory, field-tested against a fixture the author wrote.

Reporting the reached calls the table does not know is the right instinct and the right half. The missing half is the figure.

**Ruled: the table's coverage over *reached* calls is a first-class reported figure** — *the table knows n of the m registration calls actually reached*, per solution. The table's size is not the measure. A table of two hundred entries that knows three of A's fifty-eight reached calls is a table that does nothing, and only the coverage figure shows that.

The distribution of unknown reached calls, by call, is the map of what the table would have to learn, and it ranks itself by frequency.

---

## CG-R-80 — Predictions after the repairs land, and against restated rules

Unchanged in substance from CG-R-72, restated with what now gates it:

1. P-4, P-1, P-2, P-3 applied (done at v4).
2. P-5, P-6, P-7 applied with measured incidences.
3. The emission rules and the proxy set **restated in full**, including CG-R-78's blind spot and CG-R-79's table coverage.
4. Then both predictions, committed.
5. Then run 6.
6. Then §12.1.

**One thing to hold at step 3.** The restatement is the artefact a prediction is made against, and it is the first time the rules will have been written down complete rather than accumulated across five runs. If writing it out surfaces a rule nobody had stated, that is a finding and it is reported before the predictions — not absorbed into the restatement.

---

Gate 1b's act vocabulary remains mine. Register debt: CG-R-17 … CG-R-80.
