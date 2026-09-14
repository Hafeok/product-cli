# Run 5 — the recomputation under CG-R-68/69, scored against CG-R-70 — 2026-09-14

**Status: measured, scored, held.** Both predictions are wrong, in opposite directions. Reading
the edge list behind the figures found four instrument defects (O-13 … O-16); their extents are
counted below and **nothing has been recomputed under a repair** — each repair moves the
denominator after numbers exist, which CG-R-63/69 put in Emil's hands. The session holds.

Runs 3 and 4 are instrument history (CG-R-69). No figure here is compared with them.

---

## 1. What was built for CG-R-68, stated before the walk

- **`boundary`** is a fourth edge state in `csharp_walk.rs`: the target is declared outside the
  solution, is an abstraction (an interface or an abstract class, as the reader's `external_types`
  carries them), no in-solution type implements or inherits it, and no registration names it.
  Read *before* the role, so the boundary set is the whole used library surface; the role is
  reported beside each row. Outside the resolved/unresolved denominator; reported per external
  type with the assembly and the edge count; the JSON carries every composition edge.
- **Container-constructed** is the root set plus every implementation a *reached* registration
  resolves to — the registration list, never shape.
- **Property injection is emitted and read.** The reader emits the property, its attributes and a
  `signature` edge to its type; a property carrying `[Inject]` or `[FromServices]` (matched by
  resolved symbol id) on a container-constructed type is a composition edge.
  **Method injection is not emitted**: the reader carries no parameter attributes, so a
  `[FromServices]` parameter is invisible — a stated gap. Bound by source grep, not by the
  instrument: A 0 occurrences, B 7.
- **CG-R-61's shortcut is not applied.** Neither solution declares (no `[Slice]`, no binding
  attributes); generic dispatch falls back to resolution.
- **The fixture** exercises the new rows: `ILogger<T>` at a constructor (boundary), a
  `ComponentBase` component with `[Inject] ICartReader` (resolved through the registration) and
  `[Inject] ILogger<T>` (boundary). 40 types, 0 diagnostics.

**The reader's emission rules, as they stood for this run** (CG-R-69 — stated so the denominator
is defined by what the instrument emits, not only by what the consumer counts):

1. An edge whose target is declared outside the solution is emitted when the target is an
   abstraction, or the edge is a `parameter` or a `resolve`; `inherit`, `implement` and
   `attribute` edges are emitted whatever the target. Everything else lands outside and is dropped.
2. An external type is described (`external_types`) with kind, arity, abstractness and its
   **declared** member counts, when it is the target of an emitted edge under rule 1 — *not* when
   it is only inherited or implemented (`ComponentBase` has no row; the edge to it exists).
3. Base interfaces are not emitted for external types; for in-solution types they are.
4. Attributes with arguments are emitted for types and members; not for parameters.
5. Every `IServiceCollection` call is emitted as written, with generic, `typeof` and constructed
   arguments; `Add`/`TryAdd(new ServiceDescriptor(typeof(I), typeof(C), …))` arrives as a
   `typeof` pair and the resolver reads it (the source-generated Mediator registrations in A
   arrive this way).

Rules 2 and 3 are the cause of O-13 below.

---

## 2. The measurement

Instrument: this commit (`csharp_walk.rs`, `csharp_reach.rs`, reader v3 unchanged since run 4).
Inventories: the run-4 v3 inventories, unchanged (A 393 types, B 6,470). Raw outputs in
`gate1a-measurement/run5-*.txt`; every composition edge of the primary runs in
`run5-A-primary-edges.json` and `run5-B-main-controllers-startups-extract.json`.

### A — eShopOnWeb, primary (Main@Web + framework-constructed handlers)

| Composition edges | In denominator | Resolved | Unresolved | Partial | Boundary | Excluded by role | **Coverage** |
|---|---|---|---|---|---|---|---|
| 91 | 62 | 26 | 36 | 0 | 14 | 15 | **41.9%** |

Unresolved by reason: no-registration 33, conditional-registration 3. Assembly scanning: **0**.
Roles at composition edges: service 59, marker 19, factory-provider 8, generic-dispatch 4,
abstract-data 1. Registrations read 64, calls ignored 58. Reached 187 of 393 types.

### B — Orchard Core, primary (Main@Cms.Web + controllers + module Startups, CG-R-65)

| Composition edges | In denominator | Resolved | Unresolved | Partial | Boundary | Excluded by role | **Coverage** |
|---|---|---|---|---|---|---|---|
| 1,450 | 967 | 504 | 130 | 333 | 27 | 456 | **79.5%** |

Unresolved by reason: module-configuration 86, no-registration 26, keyed-service 7, factory 7,
conditional-registration 4. Assembly scanning 0, decorator 0. Roles: service 594,
factory-provider 346, marker 274, data-contract 181, generic-dispatch 44, abstract-data 8,
value 3. Registrations read 1,348, calls ignored 1,554. Reached 2,572 of 6,470 types.

### The labelled conventions, for the record

| Run | Scored edges | Coverage | Unresolved | Partial | Boundary |
|---|---|---|---|---|---|
| A main only | 7 | 14.3% | 6 | 0 | 0 |
| A every entry point | 13 | 23.1% | 10 | 0 | 2 |
| A public | 108 | 42.6% | 62 | 0 | 41 |
| B main only | 40 | 90.0% | 4 | 51 | 6 |
| B every entry point | 40 | 90.0% | 4 | 51 | 7 |
| B main + controllers, *registrations not read* | 386 | 33.2% | 258 | 194 | 14 |
| B public | 2,299 | 82.3% | 406 | 952 | 191 |

### The boundary sets — the used library surface

A (6 external types, 14 edges): `ILogger<T>` 4 (Microsoft.Extensions.Logging.Abstractions),
`AutoMapper.IMapper` 3, `IMemoryCache` 3 (Microsoft.Extensions.Caching.Abstractions),
`ILoggerFactory` 2, `IEmailSender` 1 (Microsoft.AspNetCore.Identity), `UrlEncoder` 1
(System.Text.Encodings.Web).

B (15 external types, 27 edges): `IDataProtectionProvider` 8, `ITempDataProvider` 2,
`LinkGenerator` 2, `IHubContext<T>` 2, `IHostApplicationLifetime` 2, `JavaScriptEncoder` 2,
`IAmazonS3` 1 (AWSSDK.S3), `IAntiforgery` 1, `IActionDescriptorCollectionProvider` 1,
`ITagHelperFactory` 1, `IHtmlHelper` 1, `IServiceScopeFactory` 1, `HttpMessageHandlerBuilder` 1,
`MethodInfo` 1, `PropertyInfo` 1. Full rows with assemblies in the run files.

Both sets are **lower bounds** — O-13 below keeps 4 (A) and 223 (B) edges to external
inheriting-only interfaces out of them.

### Handlers (tracked, A)

`Mediator.IRequestHandler<,>`: 2 types, 0 reached, 2 unreached. `Mediator.INotificationHandler<>`:
1 type, 0 reached. **This figure is an artefact**: the two edges to `Mediator.IMediator` are
excluded as *marker* (O-13), so the source-generated `Mediator.Mediator` is never entered and its
`resolve` edges to the handlers are never walked. The number says nothing about the handlers.

---

## 3. CG-R-70, scored

| Solution | Predicted | Centre | Measured | Verdict |
|---|---|---|---|---|
| A — eShopOnWeb | 88–96% | 92% | **41.9%** | wrong; 50.1 points below the centre, outside the band |
| B — Orchard Core | 58–72% | 65% | **79.5%** | wrong; 14.5 points above the centre, outside the band |

Scored as committed, against the criterion as this commit implements it. The defects in §4 bear
on both cells; they are the reason the score is not the last word, not a reason to rescore.

**Expected unresolved distributions.** A: "assembly scanning almost exclusively" — measured
scanning 0, no-registration 33 (32 of them edges to external *concrete classes*, §4 O-14),
conditional 3. **The premise was wrong, and the session shares the error**: eShopOnWeb (the
NimblePros fork) does not use MediatR. It uses the source-generated `Mediator` library
(`martinothamar/Mediator`), whose registrations are generated into the Web project's compilation
and arrive as `TryAdd(ServiceDescriptor)` `typeof` pairs — there is no scanning registration in
A at all. The tracked labels have said `Mediator.IRequestHandler` since run 2; the session never
flagged what that implied for the "MediatR scanning" premise in CG-R-62/70. Recorded against
the session.
B: "module configuration first, scanning second, keyed and decorator minor" — measured
module-configuration 86 first, no-registration 26, keyed 7, factory 7, conditional 4, scanning
0, decorator 0. The order is as predicted but **the first term is not what the prediction
meant**: 82 of the 86 are `IAuthorizationService`, whose only registrations and only
in-solution implementor live in `OrchardCore.Tests` (O-15). That is test doubles, not module
startup ordering.

---

## 4. Instrument findings on the edge list — extents counted, nothing recomputed

Each is a denominator move if repaired. Under CG-R-63/69 the session does not move it; the
counts below are data from the run-5 edge lists, and the *direction* each repair would push a
figure is stated without a number (CG-R-69: no hand estimates).

### O-13 — the marker proxy reads an inheriting-only interface as a marker

The proxy is "an interface declaring no methods, properties or events"; the predicate is "no
member to satisfy". An interface that declares nothing and **inherits** its members —
`IRepository<T> : IRepositoryBase<T>`, `ILogger<T> : ILogger`, `IStringLocalizer<T>`,
`IShellConfiguration : IConfiguration`, `Mediator.IMediator : ISender, IPublisher` — has members
to satisfy and is read as a marker. The declared divergence ("an empty interface used as a DI key
or a type-test target") never occurs in either solution; the undeclared one is the whole set:

| | Marker edges | To in-solution interfaces *with* base interfaces | To in-solution interfaces *without* | To external interfaces |
|---|---|---|---|---|
| A | 19 | 13 (all `IRepository<T>`, registered by an open-generic `typeof` pair) | 0 | 6 (`ILogger<T>` 4 → already boundary; `Mediator.IMediator` 2, which has an in-solution implementor) |
| B | 274 | 51 (`IShellConfiguration` 30, `IDistributedLock` 7, `IActivityDisplayManager` 3, 11 others) | 0 | 223 (`ILogger<T>` 91, `IStringLocalizer<T>` 72, `IHtmlLocalizer<T>` 58, 2 others) |

**Every marker-role edge measured in both solutions is an inheriting interface.** Cause: rule 2
(declared member counts) and rule 3 (no base interfaces for external types) in §1, and the
consumer counting only `members_of` for in-solution interfaces. Direction: the in-solution rows
(13 + 2 in A, 51 in B) enter the denominator, most with a registration; the external rows become
boundary. Repair needs a **reader emission change** (base interfaces on external types, and
noting them) — CG-R-69 puts that before any prediction it is scored against.

### O-14 — external concrete classes at composition edges

`UserManager<T>`, `SignInManager<T>`, `RoleManager<T>` (registered by `AddIdentity<,>()`, an
ignored call, with the user and role types as its type arguments), `DbCallCountingInterceptor`,
`GraphQL.Types.Schema`, `ModelExpressionProvider`: external, **not abstract**, no in-solution
subclass, no registration naming them. The implementation reads CG-R-68's "external abstraction"
as interface-or-abstract-class, so these are `unresolved: no-registration`.

| | Unresolved edges | Of which to external concrete classes |
|---|---|---|
| A | 36 | **32** (`UserManager<T>` 16, `SignInManager<T>` 7, `RoleManager<T>` 7, `DbCallCountingInterceptor` 2) |
| B | 130 | 13 (`UserManager<T>` 9, `SignInManager<T>` 2, `Schema` 1, `ModelExpressionProvider` 1) |

From the definition: the container constructs the framework's own class; there is no
implementation to choose between; the registration that supplies it is the framework's. That is
CG-R-68's "a framework contract satisfied by the framework itself", but the ruling's word is
*abstraction* and its examples are interfaces. The session implemented the narrower reading
before the walk and reports the wider one as a question, not a repair. Direction if ruled
boundary: 32 A edges and 13 B edges leave *unresolved* for *boundary*.

### O-15 — test projects count as "in the solution"

The boundary test asks whether *any* in-solution type implements the abstraction and whether
*any* registration names it. In B, 86 of 130 unresolved edges are `module-configuration`
(registered, site not reached); 82 of them land on `IAuthorizationService`, whose only
in-solution implementor is `DenyAllAuthorizationService` in `OrchardCore.Tests` and whose every
registration is at a test site. The other four (`IGraphQLSerializer`, `IGraphQLTextSerializer`,
`IAuthenticationService`, `HtmlEncoder`) are the same shape. In production these are framework
contracts (`AddAuthorization()`, ignored) — boundary. B carries 554 of its 2,902 registrations
in test projects; A 5 of 122 and no affected edge in the primary run.

The criterion as ratified says "in-solution"; a `.sln` is a build artefact that includes tests.
Whether a test project is part of the composition is a convention, like roots. Direction if test
projects are ruled outside: 86 B edges leave *unresolved* for *boundary*, and B's
"module-configuration first" disappears with them.

### O-16 — `implements:<T>` selects direct heirs only

The root convention reads `implementors[id]`, one level. A type inheriting the framework base
through an in-solution intermediate is not a root:

| | Direct heirs of the primary bases | Heirs through an intermediate, missed |
|---|---|---|
| A | 41 | 3 — `BlazorAdmin.Pages.{CatalogItemPage,RolePage,UserPage}.List` via `BlazorComponent : ComponentBase` |
| B | 464 | 6 — the `OrchardCore.Users` controllers via `TwoFactorAuthenticationBaseController` |

Consequence in A: of the 8 `[Inject]` properties, 7 sit on those three pages and never become
composition edges; the one on `ToastComponent` does (resolved). Consumer-only repair (walk the
inherit chain through in-solution bases); a root-convention change, so Emil's.

### Not a defect, checked

`Add`/`TryAdd(new ServiceDescriptor(...))` is read (rule 5). The O-11 fix holds: every external
abstraction at a composition edge is now in the denominator or in `boundary`, none absent.

---

## 5. §12.1, read on the run-5 figures

Form (fixed in `prediction-emil-gate1a.md`): fires below 75% under the primary convention,
provisional 75–<85%, clears at 85%.

- **A: fires** (41.9%).
- **B: provisional** (79.5%).

Read as the form requires, and **not settled**: O-13 moves both denominators, O-14 moves A's
unresolved count almost entirely, O-15 moves B's. The session does not read the disposition past
the numbers as they stand.

---

## 6. What neither figure establishes

- Whether A's repositories resolve — 13 edges the criterion should score are excluded (O-13).
- Anything about A's handlers — the tracked 0/3 is O-13 through `IMediator`.
- Whether B's module startup ordering costs anything — the 86 "module-configuration" edges are
  test doubles (O-15), and no edge in the primary run is unresolved because a *module's*
  registration site went unreached.
- The size of either boundary set — lower bounds until O-13 (and O-14/O-15 if ruled) land.
- Whether the *proposed* repairs would move either figure across the §12.1 line. That is
  computed after ruling or not at all.

---

## 7. Proposals, for ruling

- **P-1 (O-13).** Reader: emit base interfaces for external types and note each as an external
  type. Consumer: an interface's member count for the marker and data-contract proxies is the sum
  over its interface chain. Emission-rule change under CG-R-69: Emil rules whether run 6 is scored
  against CG-R-70 or against a fresh prediction made after the rules are restated.
- **P-2 (O-14).** Rule whether an external, non-abstract class with no in-solution subclass and no
  registration naming it is `boundary`. Recommended: yes, reported with its assembly like the rest.
- **P-3 (O-15).** Rule whether test projects are inside the composition. Recommended: an
  implementor or registration counts toward "in-solution" only if its project is reached by the
  primary roots; the rest are reported separately as *implemented only in unreached projects*.
- **P-4 (O-16).** `implements:<T>` walks the inherit chain through in-solution bases.

None applied. The order the session would work them, if ratified: P-4 (roots), P-1 (reader,
then a stated rule set), P-2 and P-3 (walk), one run, one report.

---

## 8. Weakest point

The marker proxy. It was ratified with a declared divergence that never occurs, while the
divergence that occurs at every marker edge in both solutions (293 of 293) was undeclared — the
fixture's `IDomainEvent` is the only kind of marker the proxy was tested on. A proxy whose test
case is the one shape the field never shows is not a proxy for the field, and the session built
and proposed it.

Second: the session's reading of "abstraction" in CG-R-68 as interface-or-abstract-class was a
choice made before the walk without flagging that `UserManager<T>` would fall on the other side
of it; the choice was right to make before the numbers and wrong not to name.

---

## 9. Held

Gate 1b remains blocked (Emil's act vocabulary). Nothing here is ratified by the session.
