# Rulings CG-R-71 … CG-R-75 — run 5

**Issued by Emil, 2026-09-14.**

---

## CG-R-71 — The loop breaks here. Ground truth before any further prediction.

Five runs. Four changes to the denominator or the emission rules, each found *after* numbers existed, each voiding what came before. No prediction has been scorable yet, and run 5's figures are again mostly instrument: 32 of A's 36 unresolved edges are O-14, and 19 of A's 91 and 274 of B's 1,450 edges are excluded by a proxy O-13 shows to be wrong at every occurrence.

Another ruling on another defect continues the loop. The cause is structural and the report states it exactly:

> the fixture's `IDomainEvent` is the only kind of marker the proxy was tested on. A proxy whose test case is the one shape the field never shows is not a proxy for the field.

**That generalises to every emission rule and every proxy in the instrument.** All of them were tested against a fixture authored by the same party that wrote the rule. It is the independence defect of the category list, in the reader, and a fixture cannot catch an emission gap because the fixture was written to the reader's behaviour.

**Ruled: reader recall and precision are measured against hand-verified ground truth before any further prediction is committed.**

- One project, small, real. A's `Web` project.
- Composition edges enumerated **by reading the source**, before looking at the reader's output. Both directions counted: edges the reader missed, and edges it emitted that are not composition edges.
- The result is two figures — recall and precision — reported with every coverage number thereafter.

**Until reader recall is known, coverage is uninterpretable.** A resolver following 80% of the edges it was given says nothing if the reader gave it half of them.

---

## CG-R-72 — CG-R-70 is void. No prediction until ground truth exists.

Both predictions were wrong in opposite directions, and neither was scorable: A's 41.9% is dominated by O-13 and O-14, B's 79.5% by O-13 and by the partial exclusion at CG-R-73.

**My prediction's premise was also factually wrong.** I named MediatR's assembly scanning as A's expected residue. A uses the source-generated `Mediator` library; assembly scanning is 0, and the tracked labels showed it from run 2. I asserted a fact about a codebase I had not inspected.

**That is the second time.** CG-R-64 carried a hand estimate of 78% that computed to 85.7%. Both were inference presented as ground, in rulings, by me. Recorded rather than noted.

**Ruled: no prediction is committed until CG-R-71's ground truth exists and P-1 to P-4 are applied.** Run 6 is scored against a fresh prediction made after the emission rules are restated, per P-1's own question. CG-R-70 is void and is not partially salvaged.

---

## CG-R-73 — A coverage fraction is reported with its population fraction

A's denominator is resolved + unresolved: 26/62 = 41.9%, over 62 of 91 composition edges. B's is 504/634 = 79.5%, over **634 of 1,450** — less than half the population, with 333 partial, 456 role-excluded and 27 boundary outside it.

**79.5% reads as "the instrument follows four edges in five". It follows 504 of 1,450, which is 35%.**

**Ruled:**

- Every coverage figure is reported as *x/y of the denominator, y/z of composition edges*. Never the percentage alone.
- **B's table is internally inconsistent** and is corrected: the *in denominator* column reads 967, which includes partial, while the coverage fraction uses 634, which does not. One of the two is wrong; state which and fix it.
- **Partial's disposition is ruled explicitly**, not left implied by a column. A third of B's composition edges being neither resolved nor unresolved is the largest single fact in the run and it is currently invisible in the headline.

---

## CG-R-74 — P-1 and P-4 ratified

**P-1 (O-13).** Ratified. The reader emits base interfaces for external types and notes each as external; the marker and data-contract proxies count members over the interface chain. This is an emission-rule change under CG-R-69 and its answer is given at CG-R-72: a fresh prediction.

The declared divergence never occurred and the undeclared one occurred at 293 of 293 marker edges. **Every remaining proxy is re-examined against the same question before run 6** — not *is its divergence declared*, but *does the declared divergence describe the field, or only the fixture*. The role distribution in run 5 is the evidence available for that.

**P-4 (O-16).** Ratified without qualification. `implements:<T>` walking the inherit chain through in-solution bases is a plain defect repair.

---

## CG-R-75 — P-2 and P-3 amended

**P-2 (O-14) — accepted in principle, with a carve-out that prevents laundering.**

Reading *abstraction* as interface-or-abstract-class was wrong: the criterion is whether a dependency is satisfied by a chosen implementation at composition time, and an external concrete class resolved from the container is a composition edge. Shape is not the test.

**But `UserManager<T>` is not boundary.** It is registered by `AddIdentity`, which the reader ignores. Classifying it boundary would relabel a reader gap as a legitimate category, and the number would improve for the wrong reason.

**Ruled: a fifth state, `registration-not-read`.** An external type named by a registration call the reader does not parse is reported there, with the call. Boundary is reserved for types genuinely provided with no registration naming them. The distribution of ignored calls — 58 in A, 1,554 in B — is the map of what the registration reader would have to learn, and it belongs in the report.

**P-3 (O-15) — amended.** Test projects are classified by project, not by reachability. A project referencing a test framework is a test project; the classification is stated and mechanical. The reachability-based rule proposed is more elegant and risks circularity, since reachability is the thing being measured.

Production composition and test composition are different graphs. Test projects are excluded from the primary convention with the exclusion stated, and reported separately.

**Consequence:** B's "module configuration first" was 86 edges whose sole implementor and registrations live in `OrchardCore.Tests`. Under this ruling that finding disappears from the primary figure, which is the correct outcome — it was never about startup ordering.

---

## Order

1. **CG-R-71's ground truth.** It gates everything and it is the one thing not yet attempted.
2. P-4, then P-1 with the proxy re-examination, then P-2 and P-3.
3. Restate the emission rules in full.
4. Both predictions, committed against the restated rules.
5. Run 6.
6. §12.1, on run 6.

Gate 1b's act vocabulary remains mine. Register debt: CG-R-17 … CG-R-75.
