# Rulings CG-R-85 … CG-R-88 — run 6

**Issued by Emil, 2026-09-14.**

---

## CG-R-85 — CG-R-81 failed on B, in the direction it named

A: predicted 87–94, measured 88.2. Holds.

**B: predicted 68–80 centre 74, measured 88.1. Wrong by 8.1 points above the band.**

CG-R-81 stated the falsifier: *if B comes in above 80% I was wrong about how much the retired readings were hiding.* It came in above 80%. **The retired readings were hiding edges that resolve at the same rate as everything else**, not harder ones — the shapes that were difficult to *classify* were not difficult to *resolve*, and I assumed they would be.

Recorded as a failed prediction, not softened. It failed for the stated reason, which is the only useful way to fail.

**The session's prediction held on both**, with bands narrow enough to be wrong and centres within 1 point on A and 1 point on B. It is the only one of the three that was both precise and right. That should be on the record as plainly as my miss is.

Emil's held on both and does not commit on B's disposition, per CG-R-82.

---

## CG-R-86 — Coverage is an instrument property. The codebase-discriminating quantity is the scored fraction.

**A at 88.2% and B at 88.1% is not a coincidence to move past.** Two codebases 16× apart in size, one a small clean-architecture sample and the other a large modular CMS, produced coverage figures 0.1 points apart.

What differs is not how well the scored edges resolve. It is **how much is scoreable**:

| | Scored of composition edges | Coverage of scored |
|---|---|---|
| A | 68/134 = **51%** | 88.2% |
| B | 1,673/2,552 = **66%** | 88.1% |

**The resolver resolves about 88% of whatever it can see, on both.** That is a property of the resolver, not of the codebase — which means resolution coverage, as currently reported, does not discriminate between codebases at all.

**Ruled, and this reframes §12.1.**

- §12.1 asks whether the reachable/isolated split can be trusted. That turns on **what fraction of composition edges are scoreable**, not on how well the scoreable ones resolve.
- **The scored fraction is promoted to the headline figure**, with coverage reported beneath it. Two codebases at 88% coverage and 51% versus 66% scored fraction are in materially different positions, and the current headline hides that.
- §12.1's disposition is read against the scored fraction from run 7.

This finding exists because two very different codebases were measured. One would have produced 88% and no way to know it was the instrument talking.

---

## CG-R-87 — F-7: the boundary / not-read line is an instrument artefact, and CG-R-83 stands

`ILogger<T>` classified `boundary` in A and `registration-not-read` in B, **for the same dependency**, decided by whether a solution happens to call `AddLogging` textually.

That is not a difference between the codebases. It is a difference in what the reader saw, and it lands the same dependency on opposite sides of a line that decides whether it counts against coverage.

**This is direct evidence for CG-R-83** and it arrived independently. A state whose membership is decided by the instrument's visibility cannot sit outside the denominator, because then the instrument's blindness improves its own score.

**Ruled:**

- Implicit host-builder registrations get **known-provider entries**, as proposed. `ILogger<T>`, `IConfiguration`, `IHostEnvironment` and the rest are provided whether or not a textual call appears.
- **O-18 ratified** — the 57 `ServiceDescriptor` static-factory forms are a plain resolver gap and a plain repair.
- CG-R-83 takes effect at run 7 as ruled: `registration-not-read` inside the denominator, both figures reported.

---

## CG-R-88 — A build step that does not run is a failure, not a silence

The first pass was produced by a binary that never received the CG-R-83 second figure, because a build command in a broken chain never ran. Caught on the diff.

**Fourth instance of one class.** `run_all.sh` exiting green on a crashed check; gate runs grepping past a failing check; an ignore glob reporting the same green as a passing check; now a build step silently skipped inside a chain.

CG-R-67 ruled that a check which cannot run says so. **Extended: a build or generation step that does not run is a failure of the run containing it.** Chained commands do not swallow a non-zero exit, and a run whose artefacts were built by an unknown binary is not a measurement.

Catching it on the diff was correct conduct and is the fourth time a person re-reading caught what no instrument did — which is CG-R-19 accumulating the only evidence it can.

---

## Also

**The `@assembly` suffix defect** — Orchard's own extension methods counted as external — repaired with coverage byte-identical across passes and the first pass kept as history. Correct disposition; the byte-identity is what makes it checkable rather than asserted.

**The table's measured coverage: 18 of 35 in A, 44 of 168 in B.** Half and a quarter. Run 7's headline depends on it directly and the figure now says by how much, which is what CG-R-79 was for.

**Ground truth at 67 edges**: reader recall 66, walk recall 65, precision 65/65. Precision has held at 100% across every measurement, which continues to say the criterion is sound and the work is all recall.

---

Gate 1b's act vocabulary remains mine. Register debt: CG-R-17 … CG-R-88.
