# Rulings CG-R-121 … CG-R-124 — Gate B

**Issued by Emil, 2026-09-15.**

---

## CG-R-121 — Both predictions failed. Reach does not propagate sideways.

**Reachable share: predicted 30–50, actual 21.9. Outside, falsifier fired.** The session's 18–32 held.

The error is stated: I expected the accepted roots to reach the periphery as well as the core. They reach the core and stop. **Web alone holds 83 isolated types**, because page models and view models hang off their own entry points and are shared with nothing.

> **Reachability from a ratified subset propagates through shared infrastructure and not across sibling features.** The entities, specifications, interfaces and data context are reachable because everything uses them. A page model is reachable only from the page nobody ratified.

That is a property of how the measure works and I should have seen it before predicting.

**Bound: predicted 8–20, actual 60.4. Outside, and wrong in the direction opposite to the one I guarded against.** My stated falsifier was that a small bound would mean not-read edges sat on redundant paths. The opposite holds: nothing on the accepted paths is partial, boundary or excluded, so the old form is empty at 0.0 and the new form is everything unfollowed. The not-read edges are load-bearing, not redundant.

The session's 2–6 against the old form also failed, at 0.0. **Both predictions failed for one cause**, which is the same shape as CG-R-63: two parties predicting a quantity whose composition neither had established.

---

## CG-R-122 — §12.1 fires in substance. The residual numeric form is retired.

The report says it does not fire, by 7.7 points. **Something still carries a threshold, and CG-R-89 retired the threshold form.**

Read against CG-R-89's actual replacement:

> a split whose error bound exceeds the difference it is meant to reveal does not discriminate

**A 60.4-point bound on a 21.9% figure is a bound nearly three times the effect.** The report's own sentence is the disposition: *the accepted roots could reach anything from 22% to 82% depending on 26 registrations the reader cannot read.* That is what not discriminating means, stated in full, by the session, in the same document that says it does not fire.

**Ruled: §12.1 fires.** And any remaining numeric form is struck — the bound is read against the effect it qualifies, never against a constant. A figure that "does not fire by 7.7 points" while spanning 60 points of uncertainty is a threshold measuring the wrong thing.

**What fires means here.** Not that the work was wasted. It means the reachable/isolated split on A **cannot be trusted as currently instrumented**, and the named cause is CG-R-101's unvalidated table plus P-EP-4's blindness. Both were known and labelled before the run; the run measured how much they matter, which is 60.4 points.

---

## CG-R-123 — Split the bound by external target, as composition, never as exclusion

**Report it. Do not subtract it.**

An unfollowed edge whose target is an external framework type still gates reachability to in-solution types behind it. `UserManager<ApplicationUser>` is external; `ApplicationUser`, the identity context and any custom store are not, and they sit behind it. In this codebase specifically — F-EP-4 says 26 of 29 unfollowed edges are the identity stack — the external targets are precisely the ones with in-solution types behind them.

**Ruled:** the bound's composition gains a third line — how many unfollowed edges have external targets — reported beneath the single bound alongside `unresolved` and `registration-not-read`. It is informative about where the resolver would have to learn next. **It is not subtracted**, and a report that subtracted it would shrink the bound by assuming the answer.

---

## CG-R-124 — The split measures the ratified subset as much as it measures A

**This is the reading Gate C must lead with, and F-EP-5 is the proof.**

`Order` is written by no accepted act because no ordering entry point was in the signal-bearing subset — which was my selection, under CG-R-115. So the ordering side reads as isolated **because of a ratification choice**, not because of anything about the codebase.

**The figure is not *78% of A is unspecified*. It is *78% of A is not reached by the twenty acts I chose to ratify*.** A different subset gives a different split, and nothing in the number distinguishes the two readings.

**Ruled: every reachable/isolated figure states the subset it was computed from**, and the headline carries the subset's size and its grade. `stand-in, 22 of 68 entry points` travels with the 21.9%.

**And F-EP-4 scopes it further.** Twelve of twenty acts carry no position because their state lives behind `UserManager`, `RoleManager` and `SignInManager`, and 26 of 29 unfollowed edges are the same stack. So the measurement is, in substance:

> the non-identity half of A, measured; the identity half, not measured at all.

That belongs in the report as a sentence, not as two findings a reader has to combine.

---

## Also

**The weakest point is correctly identified and correctly attributed.** *78% isolated is supported for the ordering side, the manage pages and the client only because nothing unfollowed points at them — and that argument is the session's, not the instrument's.* Naming an argument as the session's rather than the tool's is the distinction most reports lose, and it is the reason this result is usable despite firing.

**Two things stand unvalidated and in series**, both labelled throughout: the registration table under CG-R-101, and P-EP-4. The bound is built from a category whose membership that table decides. Gate C should not present a figure that depends on both without saying so in the same breath.

---

Register debt: CG-R-17 … CG-R-124. Mine.
