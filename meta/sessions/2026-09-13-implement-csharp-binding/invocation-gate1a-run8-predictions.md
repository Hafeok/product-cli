# Invocation — rulings CG-R-96 … CG-R-98 and Emil's run-8 prediction, verbatim

Received 2026-09-14 before the run-8 instrument walked either inventory; filed as
`rulings-gate1a-run8-predictions.md`. The message:

> A ruling first, because it decides what the table is worth — and it bears on the predictions.
> CG-R-96 is the ruling that decides whether the table work was worth doing. As the rules stand,
> CG-R-87 keeps table-known edges in registration-not-read, which is inside the denominator — so
> every entry added under CG-R-95 explains an edge without changing its verdict, and the table
> cannot move coverage at all. That makes the ranking exercise bookkeeping by construction.
> registration-not-read means the reader cannot read the registration. A table entry is reading
> it. From run 9 those edges take the verdict the entry implies — resolved, or boundary where the
> type is external and unimplemented in-solution. Not applied to run 8, since predictions are
> committed; run 8 reports both figures with the second unscored.
> My prediction agrees with the session almost everywhere, and that agreement is the point. Under
> the current rule the table explains without moving verdicts, so little should change. Two
> independent predictions both expecting nothing to happen is a statement about the rule rather
> than about the table.
> One real disagreement: A's scored fraction is not exact. The session says it can't move because
> the table only relabels boundary edges. That holds unless a newly-known entry names an
> in-solution type the container constructs — AddDbContext<CatalogContext> is the shape — in which
> case the constructed type opens its own dependencies and the graph expands exactly as O-18
> expanded it. That's the mechanism I missed in run 7, not one I'm inventing now. If A comes back
> at 50.7 exact, the session was right.
> The number I'd actually bet on is the unscored one. Under CG-R-96, B's coverage should rise into
> the 70s against 62.4% — Configure at 220 edges and AddLogging at 170 resolve to framework types
> and leave the denominator. That figure is what says whether the table earned its keep, and run
> 8 cannot answer it under the current rule.

What it settles before the run: **CG-R-96** — from run 9 a table-known edge takes the verdict
the entry implies (resolved to the named type, or boundary where external and unimplemented);
`registration-not-read` is reserved for calls neither parsed nor known; run 8 reports both
figures, the second unscored. **CG-R-97** — Emil's prediction (filed in `prediction-emil-run8.md`),
with its falsifier: A's scored fraction at 50.7 exact means no new entry named an in-solution
constructed type. **CG-R-98** — a termination command is scoped to process ids from a prior
query, never to a pattern that could match the session issuing it.
