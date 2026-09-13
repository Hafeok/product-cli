# Gate 1a run — rulings applied, solutions loaded, **measurement held on §2**

**Register discipline.** In force: CG-R-57, CG-R-59 (its table withdrawn), CG-R-60 … CG-R-62
(`rulings-gate1a.md`, `rulings-gate1a-di.md`); CG-R-58 is superseded and not applied.
`[PROPOSED]`: everything built below. `[OPEN]`: §2.

---

## 1. Why the measurement has not run

The run message's §2 — *Emil's prediction — commit first* — arrived as the template, verbatim:

> **[FILL THIS IN BEFORE SENDING]**

The message's own first line says it is not complete without it, and CG-R-59 requires two
predictions, separately attributed, both committed before the measurement. The second prediction
(CG-R-62's table, the ruling author's) is committed. Emil's is not. Under CG-rule-08 there is no
re-roll, so nothing below has been run on the named solutions: `product csharp reach` and `product
csharp delta` have not been invoked on their inventories, and this session has not looked at any
resolution-coverage or reachability figure for either. What *has* run is the reader, whose output
is counts of projects, types, members, references, registrations and load diagnostics — facts
about whether the solutions loaded, not the measurement.

This hold is, incidentally, the pre-registration discipline biting in a session other than the one
that originated `CG-rule-08` — the kind of instance its `discharged_by` names. Recorded, not
claimed; grading is Emil's.

## 2. Rulings applied

**CG-R-57 — the attribution rule is graded `authored`.** Defended from CG-R-52's criterion alone:
the criterion fixes the separator (a type references realised facts of one act, or of two or more)
and is silent on *which* of those acts a spanning type is attributed to. The attribution that
follows from the criterion — every act touching any fact the type references — is derivable, and it
reads every consumer of a shared fact as spanned. The rule as built uses read/write position
(`construct` = the type produces the fact), which is not in the criterion, so it is the author's.
Tuned against: fixture `CheckoutService` (constructs `OrderPlaced` and `OrderConfirmed`, reads
`Cart` and `ActorIdentity`) versus `read-model:OrderSummary`, which reads `OrderPlaced`. Made to
produce: `OrderSummary` reports *unrealised* rather than *unstructured*. The grade, the rule, the
case and the outcome are a `const` in `pf/csharp_delta.rs` and print on every delta report.

**CG-R-60 — one graph.** The `--through-implementations` flag is removed from `reach` and `delta`.
No DI-off figure exists in the code or its output. The run message's §5 ("with and without DI
traversal, each labelled") is superseded by this ruling and is not reported.

**CG-R-61 — resolution.** The reader now records every call on an `IServiceCollection` as written
(`registrations`, inventory schema **version 2**; version 1 retired unreleased). `pf/csharp_di.rs`
reads them: explicit generic pairs, `typeof` pairs (closed or open generic), instance arguments,
factory lambdas whose body constructs one type, self-registrations; keyed and `Decorate` calls are
recorded as such; `Scan`/`AddMediatR`/`RegisterServicesFromAssembly*` and kin as scanning sites.
Declaration-supplied edges (an `[Slice]` on both ends) are **not built** (2026-09-13) — no
declarations exist on either solution, and the reader records generic arguments as their
definitions, so the closed type argument a declaration edge needs is a schema-3 field.

**CG-R-62 — unresolved as a category.** An interface-mediated edge (a call or reference landing on
an interface, an abstract type, or a registered service) the resolver cannot follow is recorded
with its reason: `assembly-scanning`, `keyed-service`, `decorator-chain`,
`conditional-registration`, `module-configuration` (the registration exists but its site was never
reached), `factory` (an opaque lambda), `no-registration`. `open-generic` is not distinguished: an
open-generic `typeof` pair resolves, and one the resolver cannot read falls under `factory` or
`no-registration`. `factory` is a seventh reason, added and flagged. A type is *unresolved* when it
is not reached and implements an interface an unresolved edge landed on; the three-way split is
printed for the total, per namespace, per tracked set, and in the delta's undeclared ratios.
Resolution coverage prints first, and no reachability figure prints without its unresolved count.

Two reading rules the walk needs, both stated in the module: a registration's type arguments are
the registration, not a use of the type (`AddScoped<IFoo, Foo>()` does not reach `Foo`; the edge
exists when the registration resolves); a construction inside a registration's argument runs when
the container resolves the service, not when the site runs.

## 2a. Defects found while applying the rulings, reported

- **The first .NET 10 run of the reader over the fixture produced 0 registrations and 684 compile
  errors** (`System` unresolvable): the fixture had been restored with SDK 8 and the reader,
  now on SDK 10, saw no reference assemblies. Caught by the per-project compile-error diagnostic
  added in the same change — the reader's output said it was broken instead of silently emitting
  an inventory with no registrations. Fixed by re-restoring the fixture under SDK 10. Recorded
  because a solution inventoried with the wrong SDK would show every registration as
  `no-registration` and read as a resolver finding; the diagnostic is what stops that.
- **Registration mentions were first read as uses.** `AddScoped<IFoo, Foo>()` at `Main` produced a
  `type-reference` edge to `Foo`, so every explicitly registered implementation was "reached" by
  its own registration line — including a conditionally registered one. Caught by
  `unresolved_is_its_own_category` (the conditional `ConsoleAudit` came out reached). Fixed with
  the two reading rules in §2.
- **Interface-mediated edges were first counted per member**, so one type with four references to
  `IAudit` reported four unresolved edges. Now counted once per (referencing type, interface);
  resolution coverage is over that set.
- `Workspace.WorkspaceFailed` is obsolete on Roslyn 5.0; `RegisterWorkspaceFailedHandler` is used.

## 3. The solutions, as found — and where they differ from the message

**A — `NimblePros/eShopOnWeb`**, `main` at `03d8cffb305976e55a0b2079f5582eb6b3924302`, cloned
directly (public; no classifier refusal). Three facts differ from the message's description:

| Message says | Found |
|---|---|
| ASP.NET Core 8, `.sln` | `global.json` pins SDK **10.0.0**; every project targets **net10.0**; the solution is `eShopOnWeb.slnx` |
| not Aspire-based | `src/eShopWeb.AppHost` uses `Aspire.AppHost.Sdk/13.4.6` and `eShopWeb.AspireServiceDefaults` exists |
| dispatches through MediatR | **two** `IRequestHandler<,>` implementations in `src/Web/Features/`, and no `AddMediatR` call found by text search under `src/` |

The fork moved on since the message was written. `main` was kept (it is the named path); a
`v-sruspasari/dotnet8_migration` branch exists and was not taken. The .NET 10 SDK was installed for
the session (`dotnet-install` to `/root/.dotnet`); `dotnet restore eShopOnWeb.slnx` succeeded,
Aspire included, with no workload install. The reader was moved to .NET 10 / Roslyn 5.0 to read
`.slnx` and C# 14. The handler figure the message asks for will be reported with `--track
implements:T:MediatR.IRequestHandler`2` — over two types, which is a much thinner test of the
under-report than the message expected.

**A, as loaded (facts, not the measurement).** 13 projects, 390 types, 1,389 members, 4,277
references, 91 `IServiceCollection` calls. Six entry points — `Program.{Main}$` of `Web`, one
`AutoGeneratedProgram.Main`, and four xunit-generated `Main`s in the test projects — so the
`entry-point` convention on this solution includes test roots; a `member:<M:…>` root convention
was added to name one `Main`, and both are reported. Load diagnostics: `BlazorAdmin` carries 12
compile errors (a Razor `Edit` component the design-time build does not see), `docker-compose.dcproj`
is not a language project, and the "MSBuild failed" entries are package-pruning warnings the
workspace reports as failures while still loading the project.

**And the MediatR premise is wrong on this fork, in an instructive way.** The handlers implement
`Mediator.IRequestHandler<,>` — martinothamar's **source-generated** Mediator, not MediatR. The
generator emits `AddMediator` into `obj/…/Mediator.g.cs`, and that generated member registers
every handler explicitly: `Add(new ServiceDescriptor(typeof(IRequestHandler<,>), typeof(GetMyOrdersHandler), …))`.
The reader records the generated code because it is part of the compilation; the resolver was
extended to read the `ServiceDescriptor` forms (`Add`/`TryAdd` with `typeof` pairs). So on this
solution the "handlers are resolved by reflection and never referenced syntactically" case does
not arise: the registrations are explicit, generated, and reachable from `Program`. The
under-report the message wanted measured cannot be measured here. Three handler types exist
(`GetMyOrdersHandler`, `GetOrderDetailsHandler`, `OrderCreatedHandler`); they are tracked with
`implements:T:Mediator.IRequestHandler`2` and `implements:T:Mediator.INotificationHandler`1`.

**B — `OrchardCMS/OrchardCore`**, `main` at `4306c0717fe573f6fca1b4955909ddab6a192807`, cloned
directly. `global.json` pins SDK 10.0.302 with `rollForward: latestMajor`; `OrchardCore.slnx`; 101
module projects under `src/OrchardCore.Modules/`; `dotnet restore` succeeded. No fallback to
nopCommerce is needed at the load stage.

**B, as loaded (facts, not the measurement).** 239 projects, 6,343 types, 29,444 members,
117,209 references, 2,724 `IServiceCollection` calls; the artefact is 56 MB and took 2m55s to
produce. Seven entry points — `OrchardCore.Cms.Web`'s `Program.{Main}$`, the Pages sample, the
benchmarks, four xunit-generated test `Main`s. **361 types derive from `OrchardCore.Modules.StartupBase`**
and 1,155 of the registration calls sit in classes named `Startup` — the module-registered
shape the message describes, as facts. 101 controllers derive from `Microsoft.AspNetCore.Mvc.Controller`.
34 keyed registrations, 49 conditional, 332 with a lambda argument, and **no assembly-scanning
call at all**: Orchard registers explicitly, per module, and what makes it hard is that the
module `Startup`s are discovered by the module loader, not called. Orchard also registers through
its own generic wrappers — `AddDisplayDriver` (80), `AddNavigationProvider` (73),
`AddPermissionProvider` (72), `AddDataMigration` (68), `AddActivity` (67), … — whose bodies
register with the wrapper's type parameters, which the reader records as `!:T`. The resolver does
not read those; they land in its *ignored calls* count, now reported by method name so that the
distribution says what the resolver would have to learn next (CG-R-62's map). Load diagnostics:
five projects with compile errors (three are Orchard's own `Errors.*` negative fixtures; the test
project and `Navigation.Core` carry a few), two duplicate-source warnings, one MSBuild failure on
`OrchardCore.Tests.Integration`. The reader crashed once on Orchard — an attribute argument the
compilation could not bind gave a `TypedConstant` with no value — and now records such an
argument as `null` rather than aborting; the crash is in this record because a reader that dies
on the hard case is the kind of instrument weakness CG-R-58's successor warned about.

**Entry-point definition for Orchard Core `[PROPOSED]`.** Roots reported separately:
`entry-point` (the `Main` of `OrchardCore.Cms.Web`); `inherits Controller`
(`implements:T:Microsoft.AspNetCore.Mvc.Controller`, the framework-registered handlers MVC
resolves by reflection); and, as a *registration* root rather than an execution root,
`implements:T:OrchardCore.Modules.StartupBase` — the module `Startup` classes the module loader
discovers by scanning. Rejected: treating every `Startup` as reached from `Main` (it is not; the
loader finds them by reflection, which is exactly the `module-configuration` reason CG-R-62 names,
and folding it in would resolve by assumption what the ruling says must be reported); and
`public`, which on a framework whose every module is a public API measures the API, not the
program — it is still reported, labelled, because it is cheap and its number is itself informative.

## 4. What runs when §2 lands

For each solution, once, with the roots above:

```
product csharp inventory <inv>
product csharp reach <inv> --roots entry-point --track implements:T:Mediator.IRequestHandler`2 --track implements:T:Mediator.INotificationHandler`1   # A
product csharp reach <inv> --roots "member:M:Program.{Main}$(System.String[])" --track …           # A, the Web Main alone
product csharp reach <inv> --roots entry-point,implements:T:Microsoft.AspNetCore.Mvc.Controller,implements:T:OrchardCore.Modules.StartupBase   # B
product csharp reach <inv> --roots public                                                          # both, labelled
```

Reported per solution: resolution coverage with the unresolved distribution by reason; the
three-way disposition; the tracked handler set for A; the entry-point definition for B; both
predictions against measurement, gap per cell; §12.1 as restated at CG-R-62 (fires when the
unresolved count is large relative to the undeclared population it would reclassify — the precise
form fixed at the measurement, the prediction first). And what neither solution establishes.

**Hold.** §2 is Emil's.
