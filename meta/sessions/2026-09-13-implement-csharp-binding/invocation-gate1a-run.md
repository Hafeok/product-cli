# Invocation — the Gate 1a run, verbatim

Received 2026-09-13 with two attachments, filed beside it: `rulings-gate1a.md` (CG-R-57 … 59,
sha256 `48a82e28fd6d6fb9bfeed947a554e9c2e75be66e129aeadb48babbbdd14351c7`) and
`rulings-gate1a-di.md` (CG-R-60 … 62, sha256
`6b84a5a526c3854aeb1a10d3af6fa58319a2d9816456c4a6e9ea822a2a2ad1ef`). The message names only
the first attachment; the second arrived with it and supersedes part of the first (CG-R-60
supersedes CG-R-58; CG-R-62 replaces CG-R-59's table).

**§2 arrived as the unfilled template.** It is reproduced exactly as received. The message's own
first line says it is not complete without it.

---

# Reply to the held C# binding session — Gate 1a run

**Attach:** `rulings-cg-r-57-59.md`

**Before sending: fill in §2 with your own prediction.** The message is not complete without it, and a prediction written after the numbers are visible is not a prediction.

---

## The message

Resume. Three rulings attached, the solutions named, and both predictions below.

### 1. File and apply the rulings

`rulings-cg-r-57-59.md` — file verbatim, then apply before anything runs.

**CG-R-57 (O-8).** The attribution rule sits outside CG-R-52 and is graded separately. Defend it from CG-R-52's criterion alone, without citing the fixture. **Derived** if it follows from *references realised facts of one act, or of two or more*; **authored** if it only stands against the case it was tuned on. Either way, name the fixture case and state what outcome it was made to produce. The grade travels with every report that uses the rule.

**CG-R-58.** §12.1 is judged against the traversal setting that best models how the code actually runs, not the most favourable one. All settings are reported; the ruling is on which one the threshold is read against.

And the bias is named: a one-hop syntactic graph **under-reports** reachability, under-reporting lowers the percentage, and §12.1 fires *above* a threshold — so the instrument's known weakness makes a clear more likely than it deserves. **A clear is provisional while the reflection under-report is unquantified.** Either count the types reachable only via reflection or string registration, or mark §12.1 *provisionally cleared* and carry that limit into every report.

**CG-R-59.** Two predictions, separately attributed. Both are in this message and both are committed before the measurement.

### 2. Emil's prediction — commit first

> **[FILL THIS IN BEFORE SENDING]**
>
> Expected reachable percentage, per root convention, with and without DI traversal, for each named solution. State it as numbers, not ranges so wide they cannot be wrong.

### 3. The recorded expectation from CG-R-59

For a DI-heavy .NET solution: entry-point roots 40–60% without DI traversal, 85–95% with; public API surface 70–85% without, above 95% with. The consequence if it holds: §12.1 clears with DI traversal off and fires with it on — which under CG-R-58 means it fires, because the DI-on graph is the one that models the program.

### 4. The solutions

**Two, and the small one is not only calibration.**

**A — `NimblePros/eShopOnWeb`.** MIT, ASP.NET Core 8, single solution, five or six projects, clean architecture. The Microsoft repository at `dotnet-architecture/eShopOnWeb` was archived in January 2025; use the community-supported fork. Verify the path before cloning.

Its value is not that it is small. **It dispatches through MediatR**, so handlers are resolved by reflection over `IRequestHandler<,>` and are never referenced syntactically from a controller. A one-hop syntactic graph should report every handler unreachable. That is the weakest point you named, on the cheapest project available. **Report the handler reachability separately from the aggregate** — it is the specific number that bears on CG-R-58's under-report.

**B — `OrchardCMS/OrchardCore`.** BSD-3, large, module-registered: nearly everything resolves through convention-based DI registration inside modules. It is the hardest realistic case for a syntactic graph, and **defining an entry point there is itself hard — report how you defined it and what you rejected.**

If Orchard Core will not load or proves unreasonable to reason about, fall back to `nopCommerce/nopCommerce` — large, plugin-based, heavy DI, and a real commerce domain that would support an act vocabulary at Gate 1b. Report the fallback and why.

**Do not use `dotnet/eShop`.** It is the active Microsoft reference and it is Aspire-based, needing a workload install. Build complexity would confound an instrument you are still trying to trust.

**If the classifier refuses a clone, report it and hold.** Do not work around it, as before.

### 5. What to report

For each solution:

- reachability under every root convention, with and without DI traversal, each labelled
- which setting CG-R-58 judges the threshold against, and why that one models the program
- **prediction against measurement, both predictions, with the gap stated per cell**
- for solution A specifically: handler reachability as its own figure
- for solution B: the entry-point definition used and the alternatives rejected
- the reflection/string-registration under-report, counted or declared unmeasured
- §12.1: fired, cleared, or provisionally cleared, with the evidence

**Report both solutions; do not average them.** §12.1's disposition is ruled on the evidence rather than pre-committed to one project. A fire on the large modular case with a clear on the small clean one is a nuanced finding — the ratio works for one shape of codebase and not another — and it must be reported as that rather than resolved into a verdict.

**And say plainly what neither solution establishes.** Both are open-source projects maintained to a standard; neither is the brownfield codebase the delta was designed for. A clean result here tells you the instrument runs, not that it works on the thing it was built for.

### 6. Then

Hold before Gate 1b. The act vocabulary for whichever solution proceeds is still outstanding and is mine.
