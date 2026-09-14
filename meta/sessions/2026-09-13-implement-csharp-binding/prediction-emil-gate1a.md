# Emil's prediction — Gate 1a, committed before measurement

*Received 2026-09-14, reproduced verbatim. Committed before `product csharp reach` was run on
either solution's inventory. The second, separately attributed prediction is CG-R-62's table in
`rulings-gate1a-di.md`.*

---

. Emil's prediction — committed before measurement
Resolution coverage, the fraction of interface-mediated edges the resolver can follow:
Solution
Predicted
eShopOnWeb
85–100%
Orchard Core
60–90%
Expected unresolved distribution. On eShopOnWeb: near-zero, with whatever remains coming from open generics rather than from registration style. On Orchard Core: dominated by module configuration and conditional enablement, with assembly scanning second; decorator chains and keyed services minor.
Reachability, under the chosen root convention: to be read once resolution coverage is known. No reachability figure is predicted separately, since under CG-R-60 there is one graph and reachability is a function of how completely it was built.
This prediction disagrees with the recorded expectation at CG-R-62 on Orchard Core — 60–90% against 40–65%. The disagreement is the informative part: it is about whether module registration is more mechanically resolvable than convention-based wiring appears from the outside. eShopOnWeb discriminates nothing between the two predictions; Orchard Core is the measurement that separates them.
Both predictions span the §12.1 line. A result near 60% probably fires it, near 90% clearly does not. Neither prediction commits to a side, and that is stated rather than repaired after the fact.

---

## The precise form of §12.1, fixed here before the run (CG-R-62)

CG-R-62 restates §12.1 as firing when resolution coverage is low enough that the reached/isolated
split cannot be trusted, and leaves the precise form to be fixed at the measurement with the
prediction committed first. Fixed, by this session, `[PROPOSED]`:

- **Fires** when resolution coverage under the chosen root convention is **below 75%**.
- **Provisionally clears** at **75% to below 85%**: the ratio ships with its unresolved count
  travelling on every report.
- **Clears** at **85% and above**.

Read against the primary root convention per solution: `member:` the application `Main`
(`Web` for A, `OrchardCore.Cms.Web` for B). Every other convention is reported and labelled but
does not decide. The thresholds sit between Emil's "near 60 fires, near 90 clearly does not" and
CG-R-62's bands (85–95 clears, 40–65 fires); 75 is the midpoint of the disputed region on Orchard
Core. Coverage is counted once per (referencing type, service) pair, as `csharp_reach.rs` states.
