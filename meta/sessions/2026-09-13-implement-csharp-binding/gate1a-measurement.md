# Gate 1a — the measurement (run 3, 2026-09-14)

**Register discipline.** In force: CG-R-57, CG-R-60 … CG-R-62; the two predictions
(`prediction-emil-gate1a.md`, `rulings-gate1a-di.md`); the §12.1 form fixed in
`prediction-emil-gate1a.md` before any run. `[PROPOSED]`: every reading below. `[OPEN]`: §7.

**Provenance of the numbers.** Run 3, over inventories produced by the reader as committed with
this report (the id disambiguation, the entry-point type, and the extension-method fix all
landed between runs), by `product csharp reach` with the resolver as committed with it. Runs 1 and 2 are retained under `gate1a-measurement/run1-superseded/` and
`run2-superseded/`, each with a README naming the reader defect its numbers exposed. **Both
predictions were committed before run 1; the reader was fixed twice after numbers were seen.**
Each fix corrected something the inventory shows independently of any prediction (an entry point
with no call edges; a scanning site naming a project no implementor lives in). No threshold,
root convention or prediction was changed after run 1. The raw outputs are the `run3-*.txt`
files beside this report.

---

## 1. Resolution coverage — the figure CG-R-61 wants first

Counted once per (referencing type, service) pair; "interface-mediated" is an edge landing on an
interface, an abstract type, or a registered service.

| Solution | Root convention | Interface-mediated edges | Resolved | **Coverage** | Unresolved |
|---|---|---|---|---|---|
| **A** eShopOnWeb | `Main@Web` alone (labelled) | 5 | 0 | 0.0% | 5 |
| **A** | **PRIMARY**: `Main@Web` + FastEndpoints, PageModel, Controller/Base, ComponentBase implementors | 72 | 42 | **58.3%** | 30 |
| **A** | every `entry-point` the compiler found (9, tests included; labelled) | 8 | 0 | 0.0% | 8 |
| **A** | `public` (labelled) | 141 | 84 | 59.6% | 57 |
| **B** Orchard Core | `Main@Cms.Web` alone (labelled) | 31 | 8 | 25.8% | 23 |
| **B** | **PRIMARY**: `Main@Cms.Web` + MVC `Controller` implementors | 871 | 351 | **40.3%** | 520 |
| **B** | primary + `StartupBase` implementors as registration roots (labelled) | 2,339 | 1,239 | 53.0% | 1,100 |
| **B** | every `entry-point` (7; labelled) | 32 | 8 | 25.0% | 24 |
| **B** | `public` (labelled) | 6,118 | 2,935 | 48.0% | 3,183 |

Registration facts the resolver read: A 64 of 122 `IServiceCollection` calls, B 1,348 of 2,902.
The ignored calls, by method, are in each `run3-*` file — on B they are Orchard's own generic
wrappers (`AddDisplayDriver` 80, `AddNavigationProvider` 73, `AddPermissionProvider` 72, …),
which is the map of what the resolver would have to learn next.

## 2. The unresolved distribution, by reason

| Reason | A primary | A public | B primary | B + startups | B public |
|---|---|---|---|---|---|
| module-configuration | 0 | 4 | **230** | 32 | 3 |
| no-registration | 9 | 23 | **258** | **992** | 3,023 |
| assembly-scanning | **14** | 16 | 0 | 0 | 0 |
| conditional-registration | 7 | 13 | 10 | 12 | 39 |
| factory | 0 | 1 | 17 | 52 | 96 |
| keyed-service | 0 | 0 | 5 | 12 | 22 |
| decorator-chain | 0 | 0 | 0 | 0 | 0 |

**What the A list actually contains** (read from `run3-A-reach-PRIMARY-…txt`, after the run):
of the 30 unresolved edges, 14 `assembly-scanning` land on `PublicApi.BaseMessage` and 4
`no-registration` on `ApplicationCore.BaseEntity` — both **abstract base classes used as data
types**, not services anyone registers. The resolver treats every abstract type as
interface-mediated, so these 18 edges are the instrument's classification, not DI. The 7
conditional edges are real: `CatalogContext` and `AppIdentityDbContext` are registered inside
`if (IsDevelopment())` (in-memory vs SQL). **The instrument's definition of "interface-mediated"
is too wide, and the number that follows from it is reported as measured**; the narrowing is
proposed at §7, not applied here, because applying it after seeing the numbers would move A
from 58.3% to about 78% and the reader should see that move as a proposal, not as the result.

**What the B list contains**: under the primary convention `module-configuration` (230) is the
module `Startup`s the loader discovers by reflection and `Main` never calls — the shape the
message predicted — and `no-registration` (258) is largely **interface-typed data** (`IShape`,
`ContentDefinition`, `OrchardCoreBuilder`) plus services registered through the ignored generic
wrappers. Adding the `StartupBase` implementors as registration roots reaches the module
registrations (module-configuration falls to 32) and moves the residue to `no-registration`
(992): what is left is the wrappers and the data-shaped interfaces. Orchard has **no**
assembly-scanning call at all. Keyed and decorator are minor, as both predictions said.

## 3. Disposition — reached / unresolved / unreached

| Solution | Convention | Types | Reached | Unresolved | Unreached |
|---|---|---|---|---|---|
| A | `Main@Web` alone | 393 | 40 (10.2%) | 0 | 353 |
| A | **PRIMARY** | 393 | **188 (47.8%)** | 4 | 201 |
| A | `entry-point` | 393 | 59 (15.0%) | 0 | 334 |
| A | `public` | 393 | 351 (89.3%) | 1 | 41 |
| B | `Main@Cms.Web` alone | 6,470 | 100 (1.5%) | 27 | 6,343 |
| B | **PRIMARY** | 6,470 | **1,166 (18.0%)** | 324 | 4,980 |
| B | primary + startups | 6,470 | 3,287 (50.8%) | 282 | 2,901 |
| B | `entry-point` | 6,470 | 118 (1.8%) | 27 | 6,325 |
| B | `public` | 6,470 | 6,128 (94.7%) | 69 | 273 |

Every figure above stands beside its unresolved count, per CG-R-62. The `public` rows are the
old §12.1 shape ("more than ~90% reachable"): both solutions are at or above it under `public`,
which measures the API surface, not the program, and is labelled as such.

## 4. The handler figure for A — the number the message asked for

Three handler types implement the source-generated `Mediator.IRequestHandler<,>` /
`INotificationHandler<>`:

| Convention | Handlers reached | unresolved | unreached |
|---|---|---|---|
| `Main@Web` alone | 0 of 3 | 0 | 3 |
| **PRIMARY** | **0 of 3** | 0 | 3 |
| `public` | 3 of 3 | 0 | 0 |

**Under the primary convention every handler is unreached, and no unresolved edge lands on
their interface.** That is CG-R-62's exact worry realised: the instrument's blindness reads as
*isolated*, the benign category. The registrations are explicit and generated
(`AddMediator` is reached from `AddWebServices`, which `Main` calls); `Mediator.Mediator` is in
the solution and is reached; the generated dispatch from it to the handler wrappers
(`Mediator.Internals`, 0 of 20 reached) is not followed, and this session has **not established
why** — it is recorded as a walk limit with the cause open (§7). The message's premise was that a
syntactic graph would miss the handlers because reflection resolves them; here the registrations
are visible and the walk still misses them, which is a different and narrower failure.

## 5. Prediction against measurement, per cell

Read against the primary convention, as fixed before the run.

| Cell | Emil | CG-R-62 | Measured | Gap to Emil | Gap to CG-R-62 |
|---|---|---|---|---|---|
| A coverage | 85–100% | 85–95% | **58.3%** | −26.7 pts below the floor | −26.7 pts below the floor |
| B coverage | 60–90% | 40–65% | **40.3%** | −19.7 pts below the floor | inside, at the floor |
| B coverage, with startups as registration roots (labelled) | 60–90% | 40–65% | 53.0% | −7.0 pts below the floor | inside |
| A unresolved distribution | near zero; open generics | — | 30 edges: abstract-base classification 18, conditional 7, no-registration 5 | wrong in kind: the residue is the instrument's definition and environment conditionals, not open generics | — |
| B unresolved distribution | module configuration + conditional dominate, scanning second, keyed/decorator minor | — | module-configuration 230, no-registration 258, factory 17, conditional 10, keyed 5, scanning 0 | half right: module configuration dominates, conditional is minor, scanning absent, keyed/decorator minor | — |

**Both predictions were wrong in the same direction on A**, and by the same amount: neither
allowed for the resolver counting abstract data types as services, nor for a solution whose
startup is conditional on environment. On B the two predictions disagreed (60–90 vs 40–65) and
the measurement fell in CG-R-62's band under the primary convention and just below Emil's floor
with the startups as registration roots. The disagreement was about whether module registration
is mechanically resolvable from outside: with the module `Startup`s treated as roots it mostly is
(module-configuration 230 → 32); without that convention it is not. Which convention is the
program is a ruling, not a measurement, and is put at §7.

## 6. §12.1, as fixed before the run

Form: fires below 75% coverage under the primary convention; provisional 75–85%; clears ≥ 85%.

| Solution | Primary coverage | Disposition |
|---|---|---|
| A eShopOnWeb | 58.3% | **fires** |
| B Orchard Core | 40.3% | **fires** |

**Fired on both**, and the reader should hold two things at once. First, the number that fired
on A is dominated by an instrument classification (abstract base types as services) that a
one-line narrowing would remove, taking A to roughly 78% — provisional, not clear — and that
narrowing was not applied because it was identified after the number was seen. Second, even
with it applied, the handler finding in §4 stands: the walk cannot follow the generated
dispatch, and the handlers read as isolated. On B the firing is robust to the convention
question: 40.3% and 53.0% both fire.

**The sharper question** CG-R-58's successor asked — is there a setting that resolves real call
paths and stays usefully below the threshold — has this answer: on B, treating the module
`Startup`s as registration roots resolves most of what the loader would (coverage 53%, half the
solution reached, the residue named by wrapper), and 53% is not usefully above anything. On A
the answer depends on the §7 narrowing and on the dispatch limit. **The ratio is not dead; it is
half-blind, and it says so on every line.**

## 7. Open, for ruling — none applied

| # | Question |
|---|---|
| O-9 | **Narrow "interface-mediated" to interfaces plus abstract types that some registration names as a service.** Abstract base classes used as data (`BaseEntity`, `BaseMessage`) would stop counting. Identified after run 3; moves A to ≈78%. Ruling wanted before it is applied, and if applied, run 4 is reported against run 3, not instead of it. |
| O-10 | **Whether `StartupBase` implementors are part of Orchard's primary convention.** The module loader finds them by reflection; treating them as registration roots is modelling the loader. Under it B reads 53.0%, under the fixed convention 40.3%. Both fire; the ruling changes the residue's name, not the disposition. |
| O-11 | **The generated-dispatch walk limit** (§4). Cause not established. Whether to establish it before Gate 1b, or carry it as the recorded blindness on the handler set. |
| O-12 | The resolver's next idioms, by count: Orchard's generic registration wrappers (~700 calls); interface-typed data as a non-service category. |

## 8. What neither solution establishes

Both are open-source projects maintained to a standard. Neither is the brownfield codebase the
delta was designed for; neither carries an act vocabulary, a `[Slice]`, or a `[RealisesFact]`;
Gate 1b's regions and ratios have not been computed on either. A coverage figure here says the
instrument runs and where it is blind. It says nothing about whether, on a client's solution
with its own container idioms and its own conditionals, the reached/isolated split would be
trustworthy — the wrapper count on B is the warning: every codebase has its own `AddDisplayDriver`.

## 9. Weakest point

That the two reader fixes between run 1 and run 3 were made after numbers were seen. Each is
defended from the inventory alone, each is recorded with the figures it replaced, and neither
touched a threshold; but a reader who wants to distrust run 3 has this to point at, and the O-9
narrowing — not applied for exactly this reason — shows the line is real.

**Hold before Gate 1b.** The act vocabulary is Emil's.
