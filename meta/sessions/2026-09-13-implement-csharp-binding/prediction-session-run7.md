# The session's prediction for run 7 — committed 2026-09-14, against `reader-emission-rules-v5.md` §6

Made after the v6 inventories were generated (counts only) and before either is walked. Emil's
prediction is filed when it arrives; run 7 runs after both are committed.

**Primary conventions**: as run 6 (A with `ViewComponent`, CG-R-84). Test projects excluded.

| Solution | Scored fraction (headline) | Coverage, not-read inside (rule in force) | Coverage, not-read outside (run-6 form) |
|---|---|---|---|
| **A** | 48–54%, centre **51%** | 45–52%, centre **49%** | 86–91%, centre 88% |
| **B** | 63–69%, centre **66%** | 58–66%, centre **62%** | 86–91%, centre 88.5% |

**Reasoning.** Run 6 stands at A 68/134 (50.7%) and B 1,673/2,552 (65.6%). The host-builder
provider moves A's 11 `ILogger<T>` edges and B's 29 `IHostEnvironment` edges from *boundary* to
*registration-not-read* — no change to the scored fraction, a small fall in the not-read-inside
coverage (A from 54.5 to about 49.6 on run-6 counts). O-18 reads 57 more of B's registrations;
their services enter *scored* from *boundary*/*no-registration* and mostly resolve, a rise of a
point or two in B's fraction; A has none. The scored fraction otherwise holds: it is set by how
much of the composition graph is inside the solution's own registrations, which the repairs do
not touch.

**§12.1 under the proposed form (bands on the scored fraction):** both **fire** — A at about
half, B at about two thirds. If the form ruled differs, the disposition is read from the ruled
form; the fractions are the prediction.

**Expected unresolved distribution.** A: conditional 4, module-configuration 3, no-registration
1 — unchanged. B: factory first (`ISession`, `IShellConfiguration`), no-registration second,
conditional third, keyed fourth — the run-6 order.

**Where this is most likely wrong.** B's scored fraction, if O-18's 57 registrations feed more
collection-injection resolutions than expected (each `IEnumerable<IContentHandler>` edge counts
once whatever the number of registrations, so the effect is bounded by edges, not registrations).
