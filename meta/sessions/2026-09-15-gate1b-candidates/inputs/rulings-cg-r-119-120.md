# CG-R-119 and CG-R-120 — Gate B

**Issued by Emil, 2026-09-15.** Committed before the walk.

---

## CG-R-119 — Prediction

| | Prediction | Centre | Session |
|---|---|---|---|
| **reachable-undeclared** | 30–50% | **38%** | 18–32, centre 24 |
| **error bound** | 8–20 points | **13** | 2–6 points |
| largest region among undeclared entry points | *no facts under P-EP-4* | — | agreed, ~32 of 46 |

**Reasoning, so it can be wrong for a stated reason.**

**Reachable-undeclared, higher than the session's.** The twenty accepted acts are concentrated on basket, catalogue and identity — three areas that reach the shared core: the entities, the repositories, the specifications, the context. Those are undeclared types and they are reachable from almost every accepted root. What should be isolated is the ordering side — the order aggregate, its services and its pages, none of which is in the accepted subset — plus the manage pages and whatever `BlazorShared` carries that nothing accepted touches.

So I expect a large reachable core and a coherent isolated region, rather than the thinner reachability the session's centre implies.

**Stated falsifier:** if reachable-undeclared comes back under 25%, the accepted roots do not reach the shared core the way I expect, and the concentration of the accepted subset bought less than it looks like it should.

---

## CG-R-120 — The error bound includes `registration-not-read`, or it understates in the flattering direction

**This is a ruling, not a prediction, and it is why my bound is three times the session's.**

The bound exists because an unfollowed edge can reclassify a type between reachable and isolated. **`registration-not-read` is an unfollowed edge.** The registration exists; the reader cannot read it; the target is not reached; the type reads as isolated.

A bound of 2 to 6 points is consistent with counting only `unresolved`. **A carried 42 `registration-not-read` edges at run 6, against 134 composition edges.** If those are outside the bound, the bound is measuring a small part of the instrument's blindness and presenting it as the whole.

And the direction is the one CG-R-62 named: an unfollowed edge moves a type from the **urgent** category to the **benign** one. The codebase reads as better specified than it is, and the bound that is supposed to warn about exactly that is silent.

**Ruled:** the error bound is computed over **every edge the walk could not follow** — `unresolved` and `registration-not-read` together, reported separately beneath the single bound so the composition is visible.

**Stated falsifier for my own prediction:** if the bound comes back at 2–6 points *with* `registration-not-read` included, then A's not-read edges mostly sit on paths already reachable by another route, and the exposure is genuinely smaller than the edge count suggests. That would be a real and welcome finding about how much redundancy the graph carries.

---

## Also

**CG-R-101's table remains unvalidated**, and Gate B's `registration-not-read` verdicts rest on it. The report saying so beside each figure is correct and it compounds with CG-R-120: the bound is built from a category whose membership is decided by a table nobody has checked. Two unvalidated things in series, both labelled.

**F-EP-3 is the first codebase finding of the programme.** Nine runs of instrument work, and the first thing established about the code rather than the tool is that a Microsoft reference sample declares `UpdateRoleEndpoint` inside `UpdateUserEndpoint.cs`. Small, real, and found by confirming a route row by row rather than accepting a reading in bulk — which is CG-R-114 earning its place on its first use.

The deferral stands and the row is undeclared for Gate B.

---

Register debt: CG-R-17 … CG-R-120. Mine.
