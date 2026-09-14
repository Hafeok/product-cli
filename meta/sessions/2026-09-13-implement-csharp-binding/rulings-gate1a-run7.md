# Rulings CG-R-92 … CG-R-95 — run 7

**Issued by Emil, 2026-09-14.**

---

## CG-R-92 — CG-R-91 failed on both. Resolution expands the graph, and coverage is not monotone in resolver quality.

A: predicted 50–55, measured 48.8. B: predicted 63–68, measured 62.4. **The stated falsifier fired on B** — below run 6's 63.2%.

**The cause is a property I did not consider.** O-18 lifted the numerator by 46 resolutions and created **67 new composition edges** from the types those registrations construct. Resolving a registration does not only add a resolution; it opens the constructed type and exposes its dependencies. The denominator grew faster than the numerator.

**Ruled, because this generalises well past my miss:**

> **Coverage is not monotone in resolver quality.** Improving the resolver can lower coverage, because each resolution admits edges that were previously invisible. A fall in coverage between runs is therefore not evidence of regression, and a rise is not evidence of improvement.

Any report comparing coverage across runs states this, or the comparison misleads in both directions. The figure that moves monotonically with resolver quality is the **classified fraction**, not coverage.

I predicted an arithmetic effect on a graph I treated as fixed. It is not fixed, and the instrument's own design says so.

---

## CG-R-93 — Second convergence. Coverage leaves the headline entirely.

A at 91.8% classified and B at 92.9%. Error bounds 8.2% and 7.1%. **Again within about a point, across codebases 19× apart in edge count.**

This is CG-R-86's finding repeating on a second quantity, and together they settle what each figure measures:

| Figure | A | B | What it measures |
|---|---|---|---|
| coverage (run-6 form) | 88.2% | 88.8% | the **instrument** |
| classified fraction | 91.8% | 92.9% | the **instrument** |
| **scored fraction** | **50.7%** | **65.4%** | the **codebase** |

**Ruled: coverage is removed from the headline entirely.** It is retained in the report, beneath the scored fraction and the error bound, and it is never the figure quoted. It converges across unrelated codebases, it moves non-monotonically with resolver quality, and it has now been the headline for four runs while measuring the tool.

The scored fraction is the only figure that has discriminated between the two codebases in any run.

---

## CG-R-94 — Parsed is not known, and the instrument record must not count itself

**O-18 moved 40 of B's reached calls from unknown to parsed without the table learning anything.** Those are different properties and one figure cannot carry both.

**Ruled: reported as two figures** — *parsed* (the reader extracted the call's structure) and *known* (the table knows what the call registers). A call parsed but unknown is progress in the reader and none in the table, and conflating them would show the bottleneck moving when it has not.

**The instrument record counting itself as an uncommitted path** — the redirection creates the file before the status check runs inside it — is a self-reference defect, not a working-tree finding. Compute the status before the file exists, or exclude the record's own path and say so in the record. It is small and it makes the record's most load-bearing field wrong.

**`IHtmlLocalizer<T>` at 66 boundary edges behind a builder chain through a variable** is rule 11's stated limit operating as declared. Correctly attributed, and it belongs in the limits list with its incidence per CG-R-77 — 66 edges, measured, dated.

---

## CG-R-95 — The table grows by measured frequency, or it is declared permanent

**722 edges in B — 28% of its composition edges — carry no verdict but *a framework call the resolver does not parse supplies it*, and the table has not grown since the day it was written.**

That is the bottleneck, it is measured, and it will not move by itself. Two honest dispositions, and drifting between them is the third:

**Grow it by measured frequency.** Rank unknown reached calls by the edge count behind them — `Configure` at 220, `AddLogging` at 170 — and take the top N. The work is bounded, finite, and documentable, and the ranking does the prioritising. This is *report by cluster, not by symbol* applied to the table.

**Or declare it permanent** and report the 28% as a standing limit of the instrument rather than as a gap awaiting work nobody has scheduled.

**Ruled: the first, by frequency, with a stated N.** A table grown by documentation sweep would repeat the defect that created it — written from memory, field-tested on one fixture entry. Grown by measured frequency, every entry is justified by edges it actually moves, and its coverage figure at CG-R-79 shows whether it worked.

---

## The thing worth saying plainly

Seven runs. The instrument has been validated, and validated hard: precision has held at 100% throughout, ground truth is at 65/67, two convergence findings have established which figures measure the tool and which measure the code, and a whole class of proxy discipline was corrected along the way.

**And no finding about either codebase has been produced.** The delta has never run. Gate 1b has never started. §12.1 cannot be disposed. Every one of those waits on an act vocabulary that has been outstanding since the PRD and is mine.

That is not a criticism of the work — instrument validation had to happen and it found real defects at every turn. It is the honest position: the exercise so far has measured the measuring device.

---

Register debt: CG-R-17 … CG-R-95. Mine, along with the act vocabulary.
