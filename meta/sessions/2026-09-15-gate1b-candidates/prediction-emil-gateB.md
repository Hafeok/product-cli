# Prediction — Emil's, Gate B (CG-R-119), committed before the walk

Filed verbatim in `inputs/rulings-cg-r-119-120.md` (sha256 `8f9b199df66a228a06b6a632fe354f1e3634860ad3fb6787e8d9a97cba1f6434`), received 2026-09-15
before `product csharp regions` first ran on A. Reproduced here as the prediction record:

| | Band | Centre |
|---|---|---|
| reachable-undeclared share | **30–50%** | **38%** |
| error bound (CG-R-120 form: unresolved + registration-not-read over composition edges) | **8–20 points** | **13** |
| largest region among undeclared entry points | *no facts under P-EP-4* | — |

**Stated falsifier:** reachable-undeclared under 25% means the accepted roots do not reach the
shared core as expected.

**Falsifier for the bound:** 2–6 points *with* registration-not-read included means A's not-read
edges mostly sit on paths reachable by another route.

**The two predictions are against two forms of the bound.** The session's
(`prediction-session-gateB.md`, 2–6 points) was committed against CG-R-89's unscored fraction;
Emil's against CG-R-120's unfollowed fraction, ruled after the session's was committed. Both
forms are printed; each prediction is scored against its own form, and that is stated.
