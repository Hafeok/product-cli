# Rulings CG-R-63 … CG-R-67 — Gate 1a measurement

**Issued by Emil, 2026-09-13.** The measurement is accepted as filed. Runs 1 and 2 voided and retained with their READMEs is the right disposition, and stating that the fixes came after numbers were seen — rather than presenting run 3 as the first run — is what makes the result usable.

---

## CG-R-63 — Both predictions were wrong about a quantity neither had defined

Two independent predictions landing wrong **in the same direction by the same amount** is not two estimation errors. It is one shared assumption, and O-9 names it: eighteen of A's thirty unresolved edges are abstract base classes used as data, counted as services.

**Neither prediction was about the measured quantity.** "The fraction of interface-mediated edges the resolver can follow" was never operationally defined — nothing said what counts as an interface-mediated edge, so the denominator admitted base-classes-as-data and both predictors assumed it did not.

This is the endogenous-denominator defect from the notation pre-registration, arriving again in a different guise. It was caught there by design and missed here, and the difference is that there the frame was fixed in advance and here the measure's denominator was not.

**Ruled: any measure whose prediction is to be scored has its denominator operationally defined before the prediction is committed.** Not "resolution coverage" — *which edges enter the denominator, and which do not.*

---

## CG-R-64 — O-9: the narrowing is applied as a defect repair; A's prediction comparison is void

**Ruled: apply it**, and the test is the one used at CG-R-57 and for the category list — it is defended from the definition alone, without citing the number it produces.

It survives that test. Resolution is about determining which implementation satisfies a dependency. An abstract base class used as data has no implementations to choose between; there is nothing to resolve, so it is not an interface-mediated edge. That argument stands without reference to 58.3% or 78%.

So it is a **defect repair, not tuning**, and it is applied.

**But the prediction comparison for A is void and cannot be recovered.** Both predictions were made against the old denominator. Scoring them against the new one scores a quantity nobody predicted.

**Report both numbers**, with the definitions that produced them, and mark A's prediction **unscored** with the reason. Do not reconstruct what we would have predicted for the narrowed measure — a prediction recalled after seeing 78% is not a prediction, and labelling it "reconstructed" does not make it evidence.

The cost is real: the A measurement no longer tests anybody's expectation. That is what a denominator defect costs, and it is cheaper to say so than to salvage it.

---

## CG-R-65 — O-10: module Startups are part of the primary convention

**Ruled: yes**, and it follows directly from CG-R-60 rather than being a new choice.

Orchard's module `Startup` classes **are** the registration mechanism. CG-R-60 says the registrations are the call graph and a solution built with DI is never loaded without resolving them. Excluding the Startups is refusing to read the registrations — the DI-off error in a different form.

**B's primary figure is 53.0%.** The 40.3% number is retained and labelled as *registrations not read*, which is what it measures.

The residue — roughly 700 calls through Orchard's own registration wrappers — is the honest remaining gap and belongs in the unresolved distribution by reason, not in the headline.

**Consequence for the predictions on B:** 53.0% falls inside CG-R-62's 40–65% band and below Emil's 60% floor. B is the measurement that separated the two predictions, as expected, and it separated them.

---

## CG-R-66 — O-11 is blocking, not an open item

A's three handlers are unreached, no unresolved edge lands on their interface, the registrations are explicit and reached, and the cause is not established.

**An edge that is neither followed nor reported unresolved is invisible failure**, and it is worse than either category. Unresolved is honest blindness; isolated is a claim about the codebase. This is a third state the instrument does not know it has, and it produces the exact flattery CG-R-62 named: blindness presenting as *isolated*.

**Ruled: blocking. Gate 1b does not begin until the cause is established.**

Until it is, **no isolated count from either solution is trusted**, because the extent of this failure class is unknown. It may be three handlers or it may be a pattern; nothing in the report distinguishes those.

If the cause turns out to be unfixable, the correct outcome is a fourth reported category — *not followed, cause unknown* — rather than leaving the types in *isolated*.

---

## CG-R-67 — The CI defect: the globs are ratified, the silence is not

`ddd diff-contracts` failing on any branch carrying the C# reader, since the first Gate 1a commit, with gate runs grepping past it.

**The globs are ratified.** A check that needs a Roslyn host cannot run in a workspace without one, and pretending otherwise helps nobody.

**The silence is the defect, and the globs do not fix it.** A check globbed out of existence reports the same green as a check that ran and passed. This is the third instance of the same class: `run_all.sh` exiting green when a schema check crashes, gate runs grepping past a failure, and now an ignore glob.

**Ruled: a check that cannot run in an environment says so.** It reports *skipped, host unavailable*, and a run containing a skip is not reported as fully green. Applies to `ddd diff-contracts` here and to `run_all.sh` in the binding, which carries the same defect and is now overdue.

---

## §12.1 — disposition deferred

It fires on both solutions as fixed. **It is not ruled fired yet**, for one reason and one only: A's firing may be an artefact of the denominator defect, since the narrowing at CG-R-64 takes A to about 78% and above the line.

B's firing at 53.0% looks genuine.

**Ruled: the disposition is read after O-9 is applied and O-11's cause is established.** The ratio is neither dropped nor rescued in the meantime. A measure that fires partly because its denominator was wrong has not fired on the evidence.

---

## Carried

- O-12 is not described in what reached me; it is unruled and remains open.
- The act vocabulary for Gate 1b is mine and is outstanding.
- Register debt: CG-R-17 … CG-R-67. Mine.
