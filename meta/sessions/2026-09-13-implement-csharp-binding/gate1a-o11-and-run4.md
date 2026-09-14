# O-11's cause, and run 4 (CG-R-64 applied) — 2026-09-14

**Register discipline.** In force: CG-R-63 … CG-R-67 (`rulings-gate1a-measurement.md`).
`[PROPOSED]`: the cause below, the extent measure, run 4's readings. `[OPEN]`: §5.

---

## 1. O-11 — the cause, established

**The reader never emits an edge whose target is declared outside the solution, except
`inherit`/`implement`/`attribute`.** That was a stated scope rule ("`System.*` would otherwise
dominate"). Its consequence was not seen until now: **a dependency on an abstraction declared in
a package — `Mediator.IRequestHandler<,>`, `IMediator`, `IAuthorizationHandler`,
`IConfigureOptions<>`, `IHostedService` — produces no edge at all**, so the walk can neither
follow it through the registrations (which are in the solution and were read) nor report it
unresolved. The registration `Add(typeof(IRequestHandler<GetMyOrders,…>), typeof(GetMyOrdersHandler))`
is read; the only in-solution reference to `GetMyOrdersHandler` is that registration line (which
the walk rightly does not treat as a use); every consumer of `IMediator`/`ISender` references an
external interface and so has no edge. The handlers are therefore reached by nothing and flagged
by nothing — the third state CG-R-66 names, produced by design rather than by chance.

Verified from the inventory, not inferred: the edges into `GetMyOrdersHandler` are the generated
registration's `type-reference` and a unit test; `Mediator.Mediator`'s 41 members carry no
`call` edge into the wrappers; the wrappers' members reference only `Mediator` itself.

**Extent — it is a pattern.** Counting lifetime registrations whose service type is not among the
solution's types:

| Solution | Registrations with an in-solution service | With an **external** service | Distinct external services | In-solution types implementing an external interface |
|---|---|---|---|---|
| A eShopOnWeb | 48 | **15** | 10 (`DbContextOptions<>` 4, `Mediator.IRequestHandler<,>` 2, `HttpClient` 2, `IMediator`, `ISender`, `IPublisher`, `INotificationHandler<>`, …) | 39 of 393 |
| B Orchard Core | 1,057 | **256** | 78 (`IConfigureOptions<>` 88, `IStringLocalizerFactory` 16, `IAuthorizationHandler` 13, `IPostConfigureOptions<>` 11, `YesSql.ISession` 10, `ILoggerFactory` 9, …) | 447 of 6,470 |

Every one of those registrations is invisible to the walk today. Under CG-R-66 **no isolated
count in run 3 or run 4 is trusted**, and this table is the bound on how wrong they can be:
on B, up to 447 types can be reached only through an edge the reader does not emit.

**The fix is the reader's, and it is a schema change.** Edges to external *abstractions*
(interfaces and abstract classes) must be emitted, and those abstractions must appear in the
artefact with the facts the denominator criterion needs (kind, member shape). That is inventory
schema version 3, built with the criterion (§4) rather than before it, since the criterion fixes
what facts an edge must carry. Until it lands, the handler types and everything in the table's
last column are reported under the fourth category CG-R-66 names, *not followed — cause known:
external abstraction*, and not as isolated.

## 2. Run 4 — CG-R-64 applied, side by side with run 3

The narrowing as applied: an interface-mediated edge lands on an **interface** or on a
**registered service**; an abstract class no registration names is data. Run 3's definition:
any interface, abstract type, or registered service. Both runs use the same inventories and the
same roots; only that definition differs. Raw outputs: `gate1a-measurement/run4-*.txt`.

| Solution | Convention | Run 3 definition | **Run 4 definition (CG-R-64)** |
|---|---|---|---|
| A | PRIMARY: `Main@Web` + framework handler bases | 58.3% (42 of 72; 30 unresolved) | **85.7%** (42 of 49; 7 unresolved, all `conditional-registration`) |
| A | `public` (labelled) | 59.6% (84 of 141) | 81.6% (84 of 103) |
| B | **PRIMARY per CG-R-65**: `Main@Cms.Web` + controllers + module `Startup`s | 53.0% (1,239 of 2,339) | **62.3%** (1,250 of 2,005; 755 unresolved: no-registration 644, factory 52, module-configuration 33, keyed 14, conditional 12) |
| B | `Main@Cms.Web` + controllers — *registrations not read* (labelled) | 40.3% | 42.4% |
| B | `public` (labelled) | 48.0% | 59.6% |

Two corrections to the previous report, reported: the narrowing removed **23** of A's edges, not
the 18 counted by hand from the first 25 lines of the list, so A lands at 85.7%, not "about
78%"; and B's primary is now the CG-R-65 convention, so the headline moved from 53.0% to 62.3%
for two reasons at once (the ruling and the narrowing), which the table separates.

**A is unscored** (CG-R-64): both predictions addressed the run-3 denominator, and no prediction
of the run-4 quantity exists. **B's run-4 figure is likewise unscored** — the denominator changed
under it too, and the criterion (§4) will change it again. The run-3 comparison stands as filed
in `gate1a-measurement.md` §5 and is not restated.

**Disposition, not read.** Under the form fixed before run 1, A at 85.7% would clear and B at
62.3% would fire; CG-R-66 and CG-R-63 both say not to read it yet, and every isolated count
under it is untrusted per §1. Not read.

## 3. The handlers, after run 4

Unchanged: 0 of 3 reached under the primary convention, 3 of 3 under `public`. §1 is why.

## 4. The denominator criterion — what the implementation will be, before it is built

Emil's criterion: *an edge enters the denominator iff the dependency must be satisfied by a chosen
implementation at composition time.* Per edge. The build that realises it, `[PROPOSED]`, in
`gate1a-criterion.md` when written; its shape:

- **The reader (schema v3) records what kind of use an edge is**, as a fact: a constructor
  parameter, a method parameter, a field or property type, a return type, a generic argument, a
  type test (`is`/`as`/pattern/cast), a service-locator call (`GetRequiredService<T>` and kin),
  a call, a construction, an access. And it records external abstractions with their member
  shape (methods, properties, events; generic arity).
- **The consumer decides, per edge, from the criterion**: a constructor parameter of a type the
  container constructs, or a service-locator call, is a *composition* edge and enters the
  denominator; a type test, a method parameter, a return type, a generic constraint does not.
  Roles are then read off the target's shape with **declared proxies**: no members → marker
  (divergence: an empty interface used as a DI key); properties only → data contract
  (divergence: a service whose API happens to be property-shaped, e.g. an options accessor);
  abstract class not registered → data (CG-R-64); members returning abstractions, or
  `Func<>`/`Lazy<>`/`IServiceScopeFactory` targets → factory/provider, **partial** (divergence: a
  service that merely returns another service's result); generic contract resolved by definition
  → generic dispatch (resolved by declaration once `[Slice]` exists, CG-R-61).
- **Partial is its own state** beside resolved and unresolved, and prints as such.

## 5. Open

| # | Question |
|---|---|
| O-11 | Cause established (§1); the fix is schema v3 with the criterion. **Still blocking** until v3 runs and the external-abstraction edges are either followed or reported unresolved. |
| O-12 | *Restated, as CG-R-67 asks:* the resolver's next idioms, by count on B — Orchard's own generic registration wrappers (`AddDisplayDriver` 80, `AddNavigationProvider` 73, `AddPermissionProvider` 72, `AddDataMigration` 68, `AddActivity` 67, `AddRecipeExecutionStep` 54, `AddSiteDisplayDriver` 54, `AddDeployment` 52, `AddContentPart` 51, `AddLiquidFilter` 40 — about 700 calls) whose bodies register with the wrapper's own type parameters, which the reader records as `!:T`; and interface-typed data (`IShape`, `ContentDefinition`) that the criterion's data-contract proxy should now exclude. Under the criterion the wrappers are the factory row's *partial* — composition chooses something that chooses later. Whether the resolver should learn to substitute a wrapper call's type arguments into its body's registrations (mechanical, one level) is the ruling wanted. |
