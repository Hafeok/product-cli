# Invocation — rulings CG-R-89 … CG-R-91 and Emil's run-7 prediction, verbatim

Received 2026-09-14 before either v6 inventory was walked; filed as
`rulings-gate1a-run7-predictions.md`. The message:

> §12.1's threshold form is retired. Carrying 75% from coverage onto the scored fraction is the
> arbitrary-ceiling move already refused twice. And §12.1 has now been read four times against
> proxies for something never computed — it asks whether the reachable/isolated split can be
> trusted, and there is no split, because the delta needs an act vocabulary and Gate 1b has never
> run.
>
> Replacement: the unscored fraction is the error bound on the split. Every unscored edge is one
> that could reclassify a type between reachable and isolated. A split whose error bound exceeds
> the difference it's meant to reveal doesn't discriminate — and that needs no threshold. No
> disposition until the split exists, which puts the blocker on me where it belongs.
>
> CG-R-90 is CG-R-63 arriving a sixth time. Scored fraction has two live readings:
> resolved+unresolved gives A 51% and B 66%; adding registration-not-read gives A 82% and B 91%.
> The session's prediction is against the first, but CG-R-83 makes not-read a verdict from run 7,
> and a verdict-bearing edge is scored on any ordinary reading.
>
> Both figures get reported, the definition gets stated, the predictions aren't voided — they
> stand against the stated reading. The error bound comes from the second, since a classified
> edge can't silently reclassify.
>
> My prediction agrees with the session on the scored fraction and disagrees upward on
> not-read-inside coverage for both. For B specifically: the 46 ServiceDescriptor forms were
> already scored and unresolved, so resolving them lifts the numerator without touching the
> denominator. The session predicts 62 against run 6's 63.2 — I expect it to rise, not fall. If B
> comes in below 63.2 I was wrong about which edges that repair reaches.

What it settles before the run: **CG-R-89** retires §12.1's fire/clear form — the unscored
fraction (from the second reading) is the error bound on the reachable/isolated split, and no
disposition is taken until the split exists (Gate 1b, Emil's). **CG-R-90** — *scored fraction* is
defined before the predictions are compared; both readings reported; the session states which
reading its prediction used (the first); predictions not voided. **CG-R-91** — Emil's prediction,
against the first reading, filed in `prediction-emil-run7.md`.
