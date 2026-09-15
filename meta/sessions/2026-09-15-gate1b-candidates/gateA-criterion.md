# Gate A — the entry-point criterion, stated before any candidate is computed

**Status: `[PROPOSED]`, 2026-09-15.** Committed before the derivation runs (CG-rule-08). What
makes an inventory symbol an *external integration point* (CG-R-105), by resolved symbol id;
every place a framework's own discovery rule is a name convention is declared as a proxy with
the framework's rule cited and its incidence measured (CG-R-77); every known instrument limit is
named with its incidence. Nothing here names an act.

The inventory is A's run-8 artefact (reader v6, sha256 `3197d7e0…`). The reader is not changed.

---

## 1. Entry-point kinds

Base chains are followed through in-solution `inherit` edges and then the external
`base_type` chain the inventory records. Test projects are excluded throughout (CG-R-75).

| Kind | Symbol test | Transport name | Path |
|---|---|---|---|
| **E-1 controller action** | production type whose chain reaches `T:Microsoft.AspNetCore.Mvc.ControllerBase`; each public instance method that is not a constructor and does not carry `T:Microsoft.AspNetCore.Mvc.NonActionAttribute` | `<Type>.<Method>` | route templates from `RouteAttribute` on the type and its in-solution bases, then the member's `RouteAttribute` / `Http*Attribute` positional argument; tokens `[controller]` `[action]` `[area]` substituted by the framework's rule (template printed beside). No template anywhere → *conventional* — the `MapControllerRoute` template is a call argument the reader does not emit (L-EP-1) |
| **E-2 Razor page handler** | production type whose chain reaches `T:Microsoft.AspNetCore.Mvc.RazorPages.PageModel`; each public instance method matching the framework's handler rule (P-EP-1); a page model with no matching method yields one GET-render candidate | `<PageModel>.<Handler>` | from the code-behind's file path by the framework's routing rule (P-EP-2) |
| **E-3 ViewComponent** | production type whose chain reaches `T:Microsoft.AspNetCore.Mvc.ViewComponent`, with a public `Invoke` / `InvokeAsync` (P-EP-3) | `<Type>` | none — framework-activated from a Razor view the reader does not read (CG-R-78) |
| **E-4 FastEndpoints endpoint** | production type whose external chain reaches `T:FastEndpoints.BaseEndpoint` | `<Type>` | **not read** — the route is an argument to `Endpoint`2.Get/Post/Put/Delete/Patch/Head/Verbs/Routes` and the reader does not emit call arguments (L-EP-2) |
| **E-5 hosted service** | production type implementing `T:Microsoft.Extensions.Hosting.IHostedService` or inheriting `T:Microsoft.Extensions.Hosting.BackgroundService` | `<Type>` | none (a trigger, not an address) |

Message consumers and scheduled triggers need a library-specific id table; none is declared for
A and none is needed — A references no messaging or scheduling library
(`referenced_assemblies` of its production projects).

**HTTP method / trigger kind.** E-1: from `HttpGetAttribute`, `HttpPostAttribute`,
`HttpPutAttribute`, `HttpDeleteAttribute`, `HttpPatchAttribute`, `HttpHeadAttribute`,
`HttpOptionsAttribute` on the member (`AcceptVerbsAttribute` → *verbs, list not read*); none →
*any (conventional)*. E-2: the verb in the handler name (P-EP-1). E-3: *view-component*. E-4:
from the `call` edges to `M:FastEndpoints.Endpoint`2.Get(System.String[])` etc., by exact
member id. E-5: *hosted*.

## 2. Observed positions on each candidate

| Field | Read from |
|---|---|
| authorisation attributes, roles | `AuthorizeAttribute` / `AllowAnonymousAttribute` (`Microsoft.AspNetCore.Authorization`) on the member, the type, and the in-solution base chain, with their arguments (`Roles`, `Policy`, `AuthenticationSchemes`) as recorded |
| FastEndpoints authorisation | `call` edges from the type's members to `Endpoint`2.AllowAnonymous / Roles / Claims / Permissions / Policies / Policy / AuthSchemes`, by exact member id; the arguments are not read (L-EP-2), but `access` edges from the same member to constant fields (`F:…Constants.Roles.ADMINISTRATORS`, `F:…JwtBearerDefaults.AuthenticationScheme`) are printed as *argument symbols referenced from the configuring member* — the binding of symbol to call is not read |
| anti-forgery | `ValidateAntiForgeryTokenAttribute` on the member |
| client-identity check on the path | an `access` or `call` edge from any member on the path to `P:Microsoft.AspNetCore.Mvc.ControllerBase.User`, `P:Microsoft.AspNetCore.Mvc.RazorPages.PageModel.User`, `P:Microsoft.AspNetCore.Mvc.ViewComponent.User`, `P:Microsoft.AspNetCore.Http.HttpContext.User`, `P:System.Security.Principal.IIdentity.Name`, `P:System.Security.Principal.IIdentity.IsAuthenticated`; printed with the member it occurs in |

Conventions that live in call arguments — `AuthorizePage("/Basket/Checkout")`,
`AuthorizeFolder`, a fallback policy — are **not read** (L-EP-1). Stated beside every
candidate whose observed authorisation is empty: *empty means no attribute or call was found,
not that the endpoint is anonymous*.

## 3. The path from a candidate

Roots: the handler member(s) plus the constructors of the declaring type (E-1, E-2); the type
(E-3, E-4, E-5, a no-handler page). The walk is `csharp_walk` unchanged, with two stated
differences from the reach measure, both consumer-side:

- **Registration sites** are the closure of the hosting project's production entry point
  (`M:Program.{Main}$(System.String[])@Web`, `…@PublicApi`), computed once. A candidate's own
  walk resolves through those sites; it does not need to reach `Main` itself.
- **O-17 is off** on the per-candidate walk. O-17 makes every implementation a reached
  registration names container-constructed; on a walk whose sites include the whole host, that
  would put every registered implementation on every path. The path is what the candidate's
  own edges reach.

Reported per candidate: the production types reached (count; list in JSON), and the
composition-edge states along the path (resolved, unresolved, partial, boundary,
registration-not-read, excluded) so the per-candidate error bound is visible (CG-R-89).

The existing measurements do not move: the run-8 A reach output is regenerated with the new
binary and diffed. A difference is a defect of this gate.

## 4. Facts read and written along the path — proxy P-EP-4

A has no `[RealisesFact]` declarations and no event model, so *facts* are read through a
proxy, printed as one:

- **read:** the type argument of a composition-edge target on the path whose id, or whose
  external base chain, is `T:Ardalis.Specification.IReadRepositoryBase`1` or
  `T:Ardalis.Specification.IRepositoryBase`1`; plus the type argument of a `DbSet`1` property
  (`return_type` `T:Microsoft.EntityFrameworkCore.DbSet`1`) accessed on the path.
- **written:** the type argument of a write-capable repository parameter
  (`IRepositoryBase`1`) held by a path type that calls a write member of it — `AddAsync`,
  `AddRangeAsync`, `UpdateAsync`, `UpdateRangeAsync`, `DeleteAsync`, `DeleteRangeAsync` — by
  exact member id. A type holding more than one write-capable repository has every one listed
  as *possibly written*: the call site's type argument is not in the inventory.
- **Original predicate:** the facts (state changes) an act reads and writes.
- **Known divergence:** an entity type is where state lives, not a state change; a read of
  `Order` and a write of `Order` are not one fact. `DbSet` accesses do not distinguish read
  from write.
- **Incidence:** unmeasured — first use, no ground truth over facts exists for A.

## 5. Overlap — the merge candidates (CG-R-106)

Two candidates overlap when their paths share a production type other than their own
declaring types. Reported: groups of candidates with identical path sets; pairs with Jaccard
overlap ≥ 0.5; and the shared types ranked by how many candidates reach them. All
transport-derived; whether an overlap is a merge is not decided here.

## 6. Proxies (CG-R-77)

| Id | Proxy | Framework's own rule | Known divergence | Incidence on A |
|---|---|---|---|---|
| **P-EP-1** | a public method on a `PageModel` named `On{Verb}[{Handler}][Async]` is a handler | ASP.NET Core Razor Pages handler discovery is this name rule | a public helper method named `OnXxx` would read as a handler; a handler declared through a non-standard name is invisible | measured at run: public `PageModel` methods matching / not matching |
| **P-EP-2** | the page route is the code-behind's path under `Pages/` (or `Areas/<A>/Pages/`) without `.cshtml.cs` | ASP.NET Core Razor Pages file-path routing | a `@page "template"` directive appends or replaces the route; it lives in the `.cshtml` the reader does not read (CG-R-78) | **1 of 12** pages carries a template (`Basket/Index`, `@page "{handler?}"`), read by hand from source for this figure only |
| **P-EP-3** | a public `Invoke` / `InvokeAsync` on a `ViewComponent` is its entry | ASP.NET Core ViewComponent invocation rule | none known | measured at run |

## 7. Instrument limits that bear on this gate (reported, not repaired — CG-R-103)

| Id | Limit | Incidence on A |
|---|---|---|
| **L-EP-1** | calls to `Microsoft.AspNetCore.Builder.*` extension members are absent from the reference graph: `MapControllerRoute`, `MapRazorPages`, `MapHealthChecks`, `MapFallbackToFile`, `UseFastEndpoints` do not appear as `call` edges | 0 references to `M:Microsoft.AspNetCore.Builder.*` in 6,992, against ≥7 such calls in `src/Web/Program.cs` alone; 2 health-check endpoints and 1 fallback-file endpoint are entry points the instrument cannot see (hand-enumerated) |
| **L-EP-2** | call arguments are not emitted (emission rule 5): every FastEndpoints route, every FastEndpoints role/scheme string, every Razor Pages authorisation convention | 23 of 23 FastEndpoints routes unread; 3 `Roles`/`AuthSchemes` argument sets are constants observable only as field references |
| **L-EP-3** | Razor views and `.cshtml` are not read (CG-R-78): custom page templates, view-side component invocation, `@inject` | Web: 48 razor files, 1 `@inject`; 1 custom `@page` template |

## 8. Out of the entry set, stated

- **BlazorAdmin** — a WebAssembly client. Its pages are entry points of the client, not of
  the server; the server's integration point for it is the fallback file (L-EP-1). `[OPEN]`
  whether A's candidate set should carry the client's pages as a second system.
- The `CookieAuthenticationEvents` subclass in Web — a framework callback on an existing
  request, not an external initiation.
- `Program.Main` of each host — host start, not an integration point.
- `BaseApiController` — a controller base with no subclass in A; yields no action.

## 9. The recall check (CG-R-71's form)

`ground-truth-A-entry-points.yaml` is the hand enumeration from `src/Web` and
`src/PublicApi` at `03d8cff`, committed with this criterion before the run. Reader recall =
hand-enumerated entry points the derivation also finds / hand-enumerated; precision = derived
candidates that are in the hand enumeration / derived. Expected misses before the run: the 2
health-check endpoints and the fallback-file endpoint (L-EP-1).
