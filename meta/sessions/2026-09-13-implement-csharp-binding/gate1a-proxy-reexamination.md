# Proxy re-examination before run 6 (CG-R-74) — does the declared divergence describe the field?

**2026-09-14.** Evidence: run 5's edge lists (`gate1a-measurement/run5-A-primary-edges.json`,
`run5-B-main-controllers-startups-extract.json`, the full B JSON in the scratchpad), the only
field evidence available; the fixture is what the proxies were written against. Question per
proxy, as ruled: not *is its divergence declared*, but *does the declared divergence describe
the field, or only the fixture*.

| Proxy | Declared divergence | The field (run 5, B primary unless stated) | Verdict |
|---|---|---|---|
| **marker** — no member anywhere | "an empty interface used as a DI key or type-test target" | 293 of 293 marker edges (A 19, B 274) were inheriting-only interfaces; the declared case: 0 | described only the fixture; **repaired (P-1)**, proxy text now says so |
| **data-contract** — properties only (over the chain since v4) | "a service whose API is property-shaped (an options accessor) is excluded although composition chooses it" | 181 of 181: `IOptions<T>` 97, `IUpdateModelAccessor` 35 (in-solution, registered), `IHttpContextAccessor` 24, `IHostEnvironment` 12, `IWebHostEnvironment` 5, `IApplicationContext` 4, `IHubContext<T>` 2, others 2. Every one an accessor the container supplies; a "shape, not a dependency": 0 | the declared divergence **is the whole field**; the role excludes exactly what the criterion admits |
| **factory-provider** — `Func<>`, `Lazy<>`, `IServiceProvider`, `IServiceScopeFactory`, *or any interface with a member returning an abstraction* | "a service that merely returns another service's result reads as a factory" | 346 edges: provider ids **39** (11%); `IEnumerable<T>` **76** (22%) — collection injection, resolvable to every registration of `T`; **231** (67%) services with one abstraction-returning member: `INotifier` 55, `YesSql.ISession` 42, `IShellHost` 16, `IClock` 15, `IExtensionManager` 14, `IOptionsMonitor<T>` 12 (`OnChange` returns `IDisposable`), `IMemoryCache` 10, …. A: 8 of 8 in the third group | the declared divergence names the field's **majority as an edge case**; the *partial* population (333, a third of B's edges) is mostly services |
| **generic-dispatch** — arity > 0, not a factory | "a generic service abstraction that is not dispatch reads as dispatch; resolved by registration until a declaration supplies the edge" | 44: `IDisplayManager<T>` 22, `IDocumentManager<T>` 19, `IVolatileDocumentManager<T>` 3, all resolved; A 4: `IAppLogger<T>`, resolved. Dispatch contracts: 0 (A's Mediator handlers are reached through generated `resolve` edges, not through a generic contract) | the declared divergence is the whole field, **with no denominator effect** — resolution is the same path either way; the label is informational until CG-R-61 declarations exist |
| **abstract-data** — abstract class no registration names | "registered only through a wrapper the resolver does not read" | 8: `LinkGenerator` 2, `JavaScriptEncoder` 2, `HttpMessageHandlerBuilder`, `MethodInfo`, `PropertyInfo` — framework-provided, now `boundary`/`registration-not-read` before the role is read; `EndpointDataSource` 1 excluded. A: `UrlEncoder` 1 | the declared case not observed; the role now bites only on in-solution abstract classes no registration names — the fixture case. Keep, noted |
| **value** — struct, enum, delegate, string, object | "a primitive the container does supply (an options-bound connection string)" | 3: `Int32`, `Object`, `String`, one each; A 0 | consistent with the field; keep |

## Proposals — none applied, the predictions wait on them

- **P-5 (factory-provider).** The role is read from the declared provider ids only (`Func<>`,
  `Lazy<>`, `IServiceProvider`, `IServiceScopeFactory`, `IServiceProviderFactory<>`); nothing is
  read from return types. A service returning an abstraction is a service: resolved, unresolved,
  boundary or registration-not-read by the ordinary path. Extent in B's primary run: 307 of 346
  factory-provider edges leave *partial* for the ordinary states (76 of them are P-6's); A: 8, all
  already boundary/registration-not-read by the v4 order. Defended from the definition: the
  criterion asks whether composition chooses an implementation *of this dependency*; what the
  dependency's members return is a different edge.
- **P-6 (collection injection).** Reader: per-parameter (and per-property) type arguments,
  emitted beside the open definition (rule 6). Walk: a composition edge whose target is
  `IEnumerable<T>` / `IReadOnlyList<T>` / `IReadOnlyCollection<T>` / `T[]` is the container's
  collection injection — an edge to `T`, resolved to *every* registration of `T` (resolved if at
  least one reached registration, else the reason). Extent: B primary 76 edges, A 0. An
  emission-rule change (rule 6 amended).
- **P-7 (data-contract).** Retire the role. A property-only interface at a composition edge is
  a service like any other; "a shape, not a dependency" is served by *value*, *abstract-data*,
  and by parameters of non-container-constructed types not being composition edges at all.
  Extent: B 181 edges enter the ordinary states (97 `IOptions<T>` → registration-not-read via
  `Configure`/`AddOptions`, 24 `IHttpContextAccessor` → registration-not-read via
  `AddHttpContextAccessor`, 17 host-environment → boundary, 40 in-solution → resolved by their
  registrations, if reached); A 0 in the primary run.
- **generic-dispatch**: keep, no change; **abstract-data**, **value**: keep.

## Why the session did not apply P-5 … P-7 itself

Each moves the denominator (P-5 by a third of B's composition edges). CG-R-63/69 put denominator
moves before predictions and under ruling; CG-R-74 asked for the re-examination, not for the
repairs. The predictions are therefore not committed in this commit: they would be made against
proxies the ruling on P-5 … P-7 may change.
