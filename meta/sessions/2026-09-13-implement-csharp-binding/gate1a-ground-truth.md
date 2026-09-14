# CG-R-71 — reader recall and precision against hand-verified ground truth (A's `Web` project)

**2026-09-14.** Ground truth: `ground-truth-A-web.md` (59 edges, committed at `1a508ef` before
this comparison was computed). Instrument: the run-5 commit (`8bdb019`), inventory
`eshop-inventory.json` (v3, unchanged since run 4), the run-5 primary walk
(`gate1a-measurement/run5-A-primary-edges.json`).

**Read with the A inventory's own diagnostics in view** (`run5-A-inventory.txt`): 20
diagnostics — `BlazorAdmin` has 12 compile errors (`CS0246 'Edit' could not be found`, a
Razor-generated component missing), and the workspace reports MSBuild messages on `Web.csproj`
(package-audit warnings). `Web` itself compiled; the reader's `Web` facts are read here as facts.

---

## 1. Granularity

The reader emits a reference to a target's **original definition**: `IRepository<CatalogItem>`
and `IRepository<CatalogBrand>` from one constructor are one `(member, T:…IRepository`1)` edge.
That is an emission rule (rule 6 in §5) and it collapses three ground-truth rows:
`CatalogViewModelService` 49–53 → 3, `BasketViewModelService` 42–45 → 3.
**Ground truth at the reader's granularity: 56 edges.** Both figures below are over 56.

## 2. Reader level — are the facts there?

For each ground-truth edge: does the inventory carry a `parameter` reference from the
constructor, a `resolve` reference from the member, or a `signature` reference from an injected
property, to the target's id?

| | |
|---|---|
| Ground-truth edges | 56 |
| Present as reader facts | **55** |
| **Reader recall** | **55/56 = 98.2%** |
| Missed | 1 — #59, `Views/Manage/_ManageNav.cshtml` `@inject SignInManager<ApplicationUser>` |

**The miss (R-1).** No Razor-generated view class exists in the inventory — for `Web` or for any
project of A or B (0 types from `.cshtml`-generated sources in either; the 14 `Web` types whose
file ends in `.cshtml.cs` are the hand-written `PageModel`s). A normal `dotnet build` with
`EmitCompilerGeneratedFiles` produces `Views/Manage/_ManageNav_cshtml.g.cs` with the injected
property, so the Razor source generator runs in a build; the `MSBuildWorkspace` compilation the
reader walks does not carry its output, while it does carry the Mediator generator's output.
The cause is not established. Bound by source grep: 34 `@inject` directives across A's
`.cshtml`/`.razor` files, **342 across 222 views in B**. Every one is a composition edge the
reader cannot see.

Precision has no meaning at the fact level: the reader emits *every* constructor parameter
(`RequestDelegate`, `string`, `int`) and every resolution; deciding which are composition edges is
the walk's job. It is measured at the instrument level.

## 3. Instrument level — does the walk produce the edge?

Run 5's primary walk over `Web`'s types yields 46 composition edges.

| | |
|---|---|
| Ground-truth edges | 56 |
| Produced by the walk | **45** |
| **Walk recall** | **45/56 = 80.4%** |
| Walk edges not in the ground truth | 1 — `MediatorDependencyInjectionExtensions → ForeachAwaitPublisher` |
| **Walk precision** | **46/46 = 100%** |

The one edge outside the enumeration is in source-generated code that is not on disk
(`Mediator.g.cs`, line 74: `services.TryAdd(new ServiceDescriptor(typeof(INotificationPublisher),
sp => sp.GetRequiredService<ForeachAwaitPublisher>(), …))`), verified by emitting the generated
files. It is a real service-locator edge. The enumeration's stated limit (generated code not
read) is the reason it is absent from the ground truth, not a precision fault.

### The eleven misses, by cause

| Edges | From | Cause |
|---|---|---|
| 2 | `RevokeAuthenticationEvents` | **O-17.** Registered (`AddScoped<RevokeAuthenticationEvents>()`, site reached) but no edge resolves to it — the framework activates it through `options.EventsType = typeof(…)`. The walk marks a type container-constructed only when a composition edge resolves to it. CG-R-68 says container-constructed is *the registration list*; every implementation named by a reached registration is container-constructed, resolved to or not. An instrument defect against the ruling as written. |
| 2 | `GetMyOrdersHandler`, `GetOrderDetailsHandler` | **O-17** again (generated `TryAdd(ServiceDescriptor(typeof(Handler)…))`, site reached), compounded by O-13 (the `IMediator` edge that would reach them is excluded as marker). |
| 3 | `ApiHealthCheck` ×2, `HomePageHealthCheck` | **R-2.** `AddCheck<T>` is a call on `IHealthChecksBuilder`, chained off `AddHealthChecks()`; the reader records calls on `IServiceCollection` only. The registration exists in source and is invisible. |
| 1 | `UserContextEnrichmentMiddleware` | **Convention gap.** `UseMiddleware<T>` is a call on `IApplicationBuilder`; no root convention and no registration fact names the middleware. |
| 2 | `Basket` (ViewComponent) | **Convention gap.** `ViewComponent` is not in A's primary root set; `implements:T:Microsoft.AspNetCore.Mvc.ViewComponent` would select it. |
| 1 | `_ManageNav.cshtml` | **R-1**, the reader miss above. |

### Classification of the 45 hits, for the record (not recall)

- 6 excluded as *marker* by O-13: `IRepository<>` ×5 (registered by the open-generic `typeof`
  pair; would resolve), `IMediator` ×1 (has an in-solution implementor, generated).
- 11 *unresolved: no-registration* to `UserManager<>`, `SignInManager<>`, `RoleManager<>`,
  `DbCallCountingInterceptor` — every one registered by a call the reader ignores
  (`AddIdentity<,>()`, `AddMetronome()`): CG-R-75's `registration-not-read`, not boundary.
- 3 *boundary* to `IMemoryCache` and (missed anyway) `IHttpContextAccessor` at `HomePageHealthCheck`
  — `AddMemoryCache()` and `AddHttpContextAccessor()` are both called in `Program.cs` and both
  ignored: also `registration-not-read`, not boundary. **Under CG-R-75, run 5's `boundary` set
  for A contains at least one laundered row.**
- `ILogger<>`, `ILoggerFactory`, `UrlEncoder`, `Identity.UI.Services.IEmailSender`: no call in
  any reached source registers them — the host does. Boundary, correctly.
- Two `IEmailSender`s exist: `ApplicationCore.Interfaces.IEmailSender` (in-solution, resolved)
  and `Microsoft.AspNetCore.Identity.UI.Services.IEmailSender` (external, boundary). Same short
  name, different types, both classified consistently.

---

## 4. What the two figures say

- **The reader's facts are nearly complete for what it can see** (98.2%), and blind to one class
  it cannot see at all — Razor views — whose size in B (342 directives) is larger than B's whole
  `boundary` set.
- **The walk loses a fifth of the edges** (80.4%) before any resolution is attempted, for three
  reasons the ground truth separates cleanly: container-constructed read too narrowly (O-17, 4
  edges), registrations on chained builders not read (R-2, 3 edges), framework activation
  conventions absent from the root set (3 edges), plus R-1.
- **Precision is not the problem** (46/46). Nothing the walk calls a composition edge fails the
  criterion on reading the source.
- CG-R-71's sentence holds as stated: run 5's 41.9% for A was a resolver figure over 62 edges the
  walk gave it out of a population the ground truth puts at 56 *for one project*, with the
  classification of a third of those edges wrong for reasons already ruled (O-13, CG-R-75).

## 5. Emission rules learned here, to add to the restatement

6. A reference carries the target's **original definition**; type arguments are dropped
   (`IRepository<A>` and `IRepository<B>` are one edge).
7. Source-generated code is included when the generator's output is in the workspace
   compilation: the Mediator generator's is, the Razor source generator's is not (R-1, cause
   open).
8. Registration facts are calls whose receiver is `IServiceCollection`; a call on a builder
   returned by such a call (`IHealthChecksBuilder.AddCheck<T>`, `IdentityBuilder.…`,
   `IMvcBuilder.…`) is not recorded (R-2).

## 6. Proposals arising (beyond P-1 … P-4)

- **O-17** — container-constructed = every implementation type named by a reached registration,
  plus the roots. Applies CG-R-68's operational form as written; the session treats it as
  conformance to the ruling, not a new denominator decision, and says so.
- **R-2** — the reader records calls chained off an `IServiceCollection` call (the receiver is
  the result of a recorded call, in the same statement), with the same fields.
- **R-1** — establish why the Razor generator's output is absent from the workspace compilation;
  until then, Razor `@inject` is a stated gap with the grep bound beside every A/B figure.
- **Roots** — A's primary convention gains `implements:T:Microsoft.AspNetCore.Mvc.ViewComponent`;
  middleware named by `UseMiddleware<T>` needs a fact the reader does not emit (a call on
  `IApplicationBuilder`) — proposed as part of R-2's widening or held as a stated gap.

## 7. Weakest point of this measurement

One project, 56 edges, enumerated by the party that built the instrument, after having seen the
instrument's summaries for the same solution. The enumeration was made from source and committed
before the comparison, and the generated code it could not read was checked afterwards by
emitting it — but an independent enumerator would be a stronger ground truth than this one.
