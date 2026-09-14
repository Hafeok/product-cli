# Ground truth — composition edges in A's `Web` project, by reading the source (CG-R-71)

**Enumerated 2026-09-14 from `src/Web/**/*.cs` and `*.cshtml` of eShopOnWeb (the run-4/5
clone), before consulting the inventory or the run-5 edge list for this project.** Filed and
committed before the comparison in `gate1a-ground-truth.md` is computed.

**Not blind, stated:** the session had already read run 5's *summaries* for A (the first 25
unresolved edges, the boundary rows, the role counts) before this enumeration. It did not open
the inventory or the run-5 JSON while enumerating. The reader's per-project edge list was not
looked at until this file was committed.

## The criterion applied

An edge is a composition edge when the dependency is satisfied by a chosen implementation at
composition time: a constructor parameter of a **container-constructed** type, a
**service-locator** argument, or an **injected property**. Container-constructed is read from the
registration list (Web's own `IServiceCollection` calls) *and* from the framework activation
conventions that construct a type from the container without a registration naming it —
Razor `PageModel`s, MVC `Controller`s, `ViewComponent`s, middleware added by `UseMiddleware<T>`,
health checks added by `AddCheck<T>` (a call on `IHealthChecksBuilder`, which the reader does not
read), Mediator handlers registered by the source-generated `TryAdd(ServiceDescriptor)`, and Razor
views (`@inject`). Each type below says which.

## Edges — 71 files read, 25 types carry edges

| # | From (container-constructed by) | Target | Kind |
|---|---|---|---|
| 1 | `RevokeAuthenticationEvents` (`AddScoped<RevokeAuthenticationEvents>()`) | `IMemoryCache` | ctor |
| 2 | ″ | `ILogger<RevokeAuthenticationEvents>` | ctor |
| 3 | `UserContextEnrichmentMiddleware` (`UseMiddleware<T>`) | `ILogger<UserContextEnrichmentMiddleware>` | ctor (primary ctor) |
| 4 | `ApiHealthCheck` (`AddCheck<ApiHealthCheck>` on `IHealthChecksBuilder`) | `IOptions<BaseUrlConfiguration>` | ctor |
| 5 | ″ | `IHttpClientFactory` | ctor |
| 6 | `HomePageHealthCheck` (`AddCheck<HomePageHealthCheck>`) | `IHttpContextAccessor` | ctor |
| 7 | `ManageController` (Controller) | `UserManager<ApplicationUser>` | ctor |
| 8 | ″ | `SignInManager<ApplicationUser>` | ctor |
| 9 | ″ | `IEmailSender` | ctor |
| 10 | ″ | `IAppLogger<ManageController>` | ctor |
| 11 | ″ | `UrlEncoder` | ctor |
| 12 | `OrderController` (Controller) | `IMediator` | ctor |
| 13 | `UserController` (Controller) | `ITokenClaimsService` | ctor |
| 14 | ″ | `SignInManager<ApplicationUser>` | ctor |
| 15 | ″ | `ILogger<UserController>` | ctor |
| 16 | ″ | `IMemoryCache` | ctor |
| 17 | `ConfirmEmailModel` (PageModel) | `UserManager<ApplicationUser>` | ctor |
| 18 | `LoginModel` (PageModel) | `SignInManager<ApplicationUser>` | ctor |
| 19 | ″ | `ILogger<LoginModel>` | ctor |
| 20 | ″ | `IBasketService` | ctor |
| 21 | `LogoutModel` (PageModel) | `SignInManager<ApplicationUser>` | ctor |
| 22 | ″ | `ILogger<LogoutModel>` | ctor |
| 23 | ″ | `IMemoryCache` | ctor |
| 24 | `RegisterModel` (PageModel) | `UserManager<ApplicationUser>` | ctor |
| 25 | ″ | `SignInManager<ApplicationUser>` | ctor |
| 26 | ″ | `ILogger<RegisterModel>` | ctor |
| 27 | ″ | `IEmailSender` | ctor |
| 28 | `EditCatalogItemModel` (PageModel) | `ICatalogItemViewModelService` | ctor |
| 29 | `CheckoutModel` (PageModel) | `IBasketService` | ctor |
| 30 | ″ | `IBasketViewModelService` | ctor |
| 31 | ″ | `SignInManager<ApplicationUser>` | ctor |
| 32 | ″ | `IOrderService` | ctor |
| 33 | ″ | `IAppLogger<CheckoutModel>` | ctor |
| 34 | `Basket.IndexModel` (PageModel) | `IBasketService` | ctor |
| 35 | ″ | `IBasketViewModelService` | ctor |
| 36 | ″ | `IRepository<CatalogItem>` | ctor |
| 37 | `Pages.IndexModel` (PageModel) | `ICatalogViewModelService` | ctor |
| 38 | `Basket` (ViewComponent) | `IBasketViewModelService` | ctor |
| 39 | ″ | `SignInManager<ApplicationUser>` | ctor |
| 40 | `GetMyOrdersHandler` (generated `TryAdd(ServiceDescriptor)`) | `IReadRepository<Order>` | ctor |
| 41 | `GetOrderDetailsHandler` (generated `TryAdd(ServiceDescriptor)`) | `IReadRepository<Order>` | ctor |
| 42 | `BasketViewModelService` (`AddScoped<IBasketViewModelService, …>`) | `IRepository<Basket>` | ctor |
| 43 | ″ | `IRepository<CatalogItem>` | ctor |
| 44 | ″ | `IUriComposer` | ctor |
| 45 | ″ | `IBasketQueryService` | ctor |
| 46 | `CachedCatalogViewModelService` (`AddScoped<ICatalogViewModelService, …>`) | `IMemoryCache` | ctor |
| 47 | ″ | `CatalogViewModelService` | ctor |
| 48 | `CatalogItemViewModelService` (`AddScoped<ICatalogItemViewModelService, …>`) | `IRepository<CatalogItem>` | ctor |
| 49 | `CatalogViewModelService` (`AddScoped<CatalogViewModelService>()`) | `ILoggerFactory` | ctor |
| 50 | ″ | `IRepository<CatalogItem>` | ctor |
| 51 | ″ | `IRepository<CatalogBrand>` | ctor |
| 52 | ″ | `IRepository<CatalogType>` | ctor |
| 53 | ″ | `IUriComposer` | ctor |
| 54 | `ServiceCollectionExtensions.AddDatabaseContexts` (two `AddDbContext` factory lambdas) | `DbCallCountingInterceptor` | service locator, 2 sites |
| 55 | `WebApplicationExtensions.SeedDatabaseAsync` | `CatalogContext` | service locator |
| 56 | ″ | `UserManager<ApplicationUser>` | service locator |
| 57 | ″ | `RoleManager<IdentityRole>` | service locator |
| 58 | ″ | `AppIdentityDbContext` | service locator |
| 59 | `Views/Manage/_ManageNav.cshtml` (Razor view, `@inject`) | `SignInManager<ApplicationUser>` | injected property |

**Totals:** 53 constructor-parameter edges, 5 service-locator edges (6 call sites), 1 injected
property — **59 distinct (type, target) edges**, 60 sites.

## Considered and not counted, with the reason

- `UserContextEnrichmentMiddleware(RequestDelegate next, …)` — `next` is passed explicitly by
  the pipeline when it activates the middleware, not chosen from the container.
- `GetMyOrders(string)`, `GetOrderDetails(string, int)` — messages, `new`-constructed.
- `Program`: `AddScoped<HttpClient>(s => new HttpClient …)` takes the provider and resolves
  nothing; `app.Logger` is a property read, not a resolution.
- `SlugifyParameterTransformer` — `new`-constructed in `AddMvc`, and named in `ConstraintMap`
  by `typeof`; no constructor parameters either way.
- `IdentityHostingStartup` — activated by reflection, parameterless; its `ConfigureServices`
  lambda is empty.
- `BaseUrlConfiguration`, `CatalogSettings` — options POCOs, bound not constructed.
- Every `ViewModels/**` type, `Constants`, `CacheHelpers`, the extension-method statics,
  `GitHubClaimsHelper`, `ManageNavPages`, `BaseApiController`, `Admin/IndexModel`,
  `SuccessModel`, `ErrorModel`, `PrivacyModel` — no constructor parameters, no resolution.
- No `[FromServices]` and no `[Inject]` anywhere in `Web` (`BlazorAdmin` is a different project).

## What the enumeration does not cover

Types outside `Web` that Web's registrations construct (`EfRepository<>`, `BasketService`,
`OrderService`, `LoggerAdapter<>`, `UriComposer`, `IdentityTokenClaimService`, …) — their
constructor edges belong to their own projects. The generated Razor view classes and the
generated Mediator code exist only in `obj/` and were not read; edge 59 is read from the
`@inject` directive.

---

## Addendum, after run 6's first pass (2026-09-14)

Run 6 listed eleven walk edges from `Web` outside this enumeration, all in the source-generated
Mediator code the enumeration had declared out of its reach. They were verified by reading the
files a build with `EmitCompilerGeneratedFiles` writes (`Mediator.g.cs`: `ContainerMetadata(IServiceProvider)`
at line 656; `sp.GetRequiredService<RequestHandlerWrapper<…>>` at 667–668; the wrappers'
`GetRequiredService<IRequestHandler<…>>` / `GetServices<IPipelineBehavior<…>>` at 141–142;
`GetServices<INotificationHandler<…>>` at 592; `GetServices<IContainerProbe>` at 660;
`sp => sp.GetRequiredService<ForeachAwaitPublisher>()` at 74) and added to
`ground-truth-A-web.yaml` as a second section: **67 edges** at the reader's granularity. The
markdown table above is unchanged — it is the hand enumeration of the committed source, and the
generated section is labelled as such in the YAML.
