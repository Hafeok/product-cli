# The session's prediction for run 6 — committed 2026-09-14, against `reader-emission-rules-v5.md`

Made after the v5 inventories were generated (facts only: counts) and before either is walked.
Emil's prediction is filed separately when it arrives; run 6 runs after both are committed.

**Primary conventions unchanged**: A = `Main@Web` + the framework-constructed handler bases of
run 2 (Endpoint, EndpointWithoutRequest, PageModel, Controller, ControllerBase, ComponentBase;
`ViewComponent` is proposed and not ruled, so its 2 known edges stay missed); B = `Main@Cms.Web`
+ Controllers + module `StartupBase` (CG-R-65). Test projects excluded (CG-R-75).

| Solution | Resolution coverage | Centre | Scored / composition edges (population) |
|---|---|---|---|
| **A — eShopOnWeb** | 84–94% | **89%** | roughly two thirds |
| **B — Orchard Core** | 82–92% | **87%** | roughly two thirds |

**Reasoning, so it can be wrong for a stated reason.**

*A.* The run-5 unresolved set was 32 external concrete classes, now `registration-not-read`
(`AddIdentity`, `AddMetronome`); the 13 `IRepository<>` edges enter and resolve through the
open-generic pair; `IMediator` enters and resolves through the generated `TryAdd` pair, and the
handlers are reached through the generated `resolve` edges. What stays unresolved:
`CatalogContext`/`AppIdentityDbContext` (registered inside `if`/`else` — *conditional*), and
`CatalogSettings` at `UriComposer` (an options POCO named by no parsed registration —
*no-registration*). Expected distribution: conditional-registration first, no-registration
second, nothing else. Blind spot beside it: 45 `@inject` directives in 71 Razor files.

*B.* P-5 moves 307 edges out of *partial* into the ordinary states, most resolved: `INotifier`,
`IShellHost`, `IClock`, `IExtensionManager` are registered in module startups the walk roots on.
P-7 moves 181: `IOptions<T>` and `IHttpContextAccessor` to *registration-not-read*, the host
environments to *boundary*, `IUpdateModelAccessor` resolved. The 86 test-double edges leave.
What stays unresolved: YesSql `ISession`/`IStore` (~47, registered through lambdas the resolver
reads as *factory*), keyed (7), collection injection whose type argument no reached registration
names, and GraphQL types. Expected distribution: factory first, no-registration second, keyed
third, conditional minor; scanning 0, decorator 0. Blind spot beside it: 342 `@inject`
directives in 1,610 Razor files, roughly a quarter of the population.

**Table coverage (CG-R-79)**, predicted: A — most reached external calls parsed or known, under
five unknown; B — tens of unknown calls, led by `Configure`-adjacent options calls and Orchard's
`AddTagHelpers`/`AddResourceConfiguration` extension methods in the `Microsoft.Extensions.
DependencyInjection` namespace that are Orchard's own (in-solution, bodies walked, not gaps).

**Where this is most likely wrong.** B's factory count: if Orchard registers `ISession` through a
form the resolver reads (a `typeof` pair or a constructing lambda), coverage lands above the
band; if collection injection finds many unregistered type arguments, below it.

---

**Correction, recorded against the session (2026-09-14).** The commit message of `59b6945`
transcribes A's band as "84–96". The band is **84–94, centre 89**, as written above; this file
is the record, the commit message is not rewritten.
