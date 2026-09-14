# Rulings CG-R-89 … CG-R-91 — run 7

**Issued by Emil, 2026-09-14.**

---

## CG-R-89 — §12.1's threshold form is superseded. The scored fraction is an error bound, not a verdict.

The proposal is to carry the existing three bands onto the scored fraction. **Refused**, for two reasons.

**A threshold chosen for one quantity does not transfer to another.** 75% was fixed for resolution coverage. Applying it to the scored fraction because it is the number to hand is the arbitrary-ceiling move already refused for the boundary ratio and for the re-derivation overlap.

**And §12.1 has now been read four times against proxies for something that has never been computed.** It asks whether the reachable/isolated split can be trusted. There is no split: the delta needs an act vocabulary, and Gate 1b has never run.

**Ruled: §12.1's fire/clear form is retired and replaced.**

> The **unscored fraction is the error bound on the reachable/isolated split.** Every unscored composition edge is one that could reclassify a type between reachable-undeclared and isolated-undeclared. A split whose error bound exceeds the difference it is meant to reveal does not discriminate.

**No disposition is taken until the split exists.** Run 7 produces the bound; the bound is reported with the split when there is one. A measure reported with an error bar larger than its effect is unusable, and that statement needs no threshold.

This puts the blocker where it belongs. §12.1 cannot resolve while the act vocabulary is outstanding, and that is mine.

---

## CG-R-90 — *Scored fraction* is defined before the predictions are compared

Two readings are live and they differ by a lot:

| Reading | A, run 6 | B, run 6 |
|---|---|---|
| resolved + unresolved, over composition edges | 68/134 = **51%** | 1,673/2,552 = **66%** |
| resolved + unresolved + registration-not-read, over composition edges | 110/134 = **82%** | 2,332/2,552 = **91%** |

The session's prediction — A 48–54, B 63–69 — is against the first. But CG-R-83 puts `registration-not-read` inside the *coverage* denominator from run 7, which makes it a verdict, and a verdict-bearing edge is scored by any ordinary reading of the word.

**This is CG-R-63 again**: a prediction committed against a quantity that has two readings and no stated definition.

**Ruled:**

- The restatement **defines scored fraction explicitly** before run 7, and the session states which reading its committed prediction used.
- **Both figures are reported.** They measure different things and both matter: the first is *what received a resolution verdict the resolver could act on*, the second is *what received any verdict at all*.
- **The predictions are not voided.** They stand against the stated reading, and the second figure is reported without being scored against anything.
- Under CG-R-89 the error bound is computed from the **second** reading, since a `registration-not-read` edge is classified and cannot silently reclassify a type.

---

## CG-R-91 — Prediction committed, against the session's reading

Resolved + unresolved over composition edges, so the two are comparable.

| | Scored fraction | Coverage, not-read inside | Coverage, run-6 form |
|---|---|---|---|
| **A** | 49–54, centre **51** | 50–55, centre **52** | 87–92, centre **89** |
| **B** | 64–69, centre **66** | 63–68, centre **65** | 88–92, centre **90** |

**Where I agree and where I do not.** On the scored fraction I expect essentially no movement and land on the session's centres: v6's repairs change classification and resolution, not what is scoreable. Two predictions agreeing is not a failure of the two-prediction design — independence was the point, not disagreement.

**I disagree on the not-read-inside coverage, upward on both.** For A, the host-provider entries move roughly six to nine edges from `boundary` into `registration-not-read`, enlarging the denominator without adding resolutions, so coverage falls from 54.5% but less far than 49% implies. For B, the 46 `ServiceDescriptor` forms were already scored and unresolved; resolving them adds to the numerator without touching the denominator, which should lift B rather than lower it — the session predicts 62 against run 6's 63.2, and I expect it to rise.

**Stated falsifier.** If B's not-read-inside coverage comes in below 63.2% — below run 6 — I was wrong that the `ServiceDescriptor` repair reaches edges that were already in the denominator, and the repair is touching edges that were outside it.

---

## Also

**The 400-line limit catching the resolver** and forcing report figures into their own module is a constraint doing its job. Worth noting that the gate fired on structure rather than on a defect, which is the cheap kind.

**The run-7 script building the binary it measures with, exiting on any failing step, and writing an instrument record with commit, tree state and hashes before the first measurement** — that is CG-R-88 discharged properly rather than patched. The instrument record is the artefact that makes run 6's build-chain defect impossible to repeat silently.

**The fixture at 24/24 on all three ground-truth figures**, with a genuine boundary edge included, is the right shape. The figure that will matter is the same three on A and B.

---

Gate 1b's act vocabulary remains mine, and CG-R-89 now makes it the blocker for §12.1 as well. Register debt: CG-R-17 … CG-R-91.
