# Rulings CG-R-57 … CG-R-59 — C# binding Gate 1a

**Issued by Emil, 2026-09-13.** Gate 1a ratified as built. The measurement has not run and the fixture numbers carry no finding, correctly stated.

---

## CG-R-57 — O-8: the attribution rule is outside CG-R-52 and is graded separately

**It does not sit inside CG-R-52.** That ruling ratified a separator with its three proxy fields. An attribution rule arrived at afterwards by making a fixture case come out as expected is a different artefact with a different warrant, and folding it in would launder fixture-fitting through a ratified ruling.

**Ruled: the same test as the category list's per-row rule.** The attribution rule is defended **from CG-R-52's criterion alone**, without citing the fixture. If it follows from *a type references realised facts of one act, or of two or more*, it stands and the fixture is corroboration. If it can only be justified by the case it was tuned against, it is fitted.

Two grades, both usable, recorded on the rule and carried into every report that uses it:

| Grade | Condition |
|---|---|
| **derived** | the rule follows from the criterion; the fixture corroborates |
| **authored** | the rule is the author's judgement, arrived at against a named case |

An **authored** attribution rule is not disqualifying. It means the region boundary is an artefact of this author's reading rather than of the criterion, and a §12.2 firing under an authored rule says less than one under a derived rule.

**Whichever grade it takes, name the fixture case and state what outcome it was made to produce.** A rule tuned against a case nobody can see is the worst of the three positions.

---

## CG-R-58 — §12.1 is evaluated against the traversal setting that models real call paths

The weakest point matters more than it reads, and it biases in a direction worth naming.

A one-hop syntactic reference graph **under-reports** reachability: handlers wired by reflection or string registration do not link. Under-reporting lowers the measured percentage, which makes §12.1 — *fires above ~90%* — **less** likely to fire. So the instrument's known weakness pushes toward keeping a ratio that may not deserve to be kept. A clear obtained that way is a false clearance, not a result.

**Ruled, in two parts.**

**The threshold is evaluated against the traversal setting that best models how the code actually runs**, not against the most favourable one. Reporting 50% with DI traversal off, on a solution where every handler is resolved through a container, measures a graph that does not represent the program. All settings are reported; the ruling is on which one §12.1 is judged against.

**A clear is provisional while the reflection under-report is unquantified.** Either count the types reachable only via reflection or string registration, or state it as unmeasured and mark §12.1 *provisionally cleared*. Provisional means the ratio ships and the limit travels with every report that uses it.

**The sharper question**, which the flat threshold obscures: is there a traversal setting under which reachability both resolves real call paths and stays usefully below the threshold? If not, the ratio is dead regardless of which number is reported, and §12.1 has fired in substance.

---

## CG-R-59 — Two predictions, separately labelled

The §12.1 prediction is Emil's to commit. Mine is recorded alongside it, clearly attributed, so that if both are wrong in the same direction that is itself informative about the design's assumptions.

**Recorded expectation, for a DI-heavy .NET solution, before any measurement:**

| Root convention | DI traversal | Expected reachable |
|---|---|---|
| entry-point (controllers, `Main`, hosted services) | off | 40–60% |
| entry-point | on | 85–95% |
| public API surface | off | 70–85% |
| public API surface | on | >95% |

**The consequence if that holds:** §12.1 clears with DI traversal off and fires with it on — which under CG-R-58 means it fires, because the DI-on graph is the one that models how the code runs.

I state that expectation plainly rather than hedging, because a prediction that cannot be wrong is not a prediction.

---

## Before the run

Mine: the named brownfield solution, attached. The classifier refusal stands and is not worked around.

Both predictions committed before the measurement, per §12 and CG-R-59.

Register debt: CG-R-17 … CG-R-59. Mine.
