# Run 6 — instrument v5, scored against three committed predictions, §12.1 read twice — 2026-09-14

**Status: measured, scored, held.** Instrument at commit `43d16b6` plus the O-19 repair below
(a report-figure fix that moves no coverage number — both passes are filed). Inventories: v5,
generated after the v5 rules were restated, walked only after all three predictions were
committed. Raw outputs: `gate1a-measurement/run6-*.txt`, the primary JSONs beside them, the
first pass under `run6-before-O-19/`.

---

## 1. The measurement (primary conventions)

A: `Main@Web` + Endpoint, EndpointWithoutRequest, PageModel, Controller, ControllerBase,
ComponentBase, **ViewComponent** (CG-R-84). B: `Main@Cms.Web` + Controllers + module
`StartupBase`. Test projects excluded (A 5 projects, 73 types; B 5 projects, 606 types).

| | Composition edges | Scored | Resolved | Unresolved | Partial | Boundary | Not-read | Excluded | Collection | **Coverage, rule in force** | **Coverage, not-read inside (run-7 rule, CG-R-83)** |
|---|---|---|---|---|---|---|---|---|---|---|---|
| **A** | 134 | 68 (50.7%) | 60 | 8 | 2 | 21 | 42 | 1 | 0 | **60/68 = 88.2%** | 60/110 = **54.5%** |
| **B** | 2,552 | 1,673 (65.6%) | 1,474 | 199 | 71 | 138 | 659 | 11 | 139 | **1,474/1,673 = 88.1%** | 1,474/2,332 = **63.2%** |

Blind spot beside each (CG-R-78): A 45 `@inject` directives in 71 Razor files, B 342 in 1,610 —
composition edges the instrument cannot see; A's is proportionally the larger (45 against 134
edges, B's 342 against 2,552).

Reached: A 231 of 320 production types (72.2%), B 3,696 of 5,864 (63.0%).

**Unresolved by reason.** A: conditional-registration 4 (`CatalogContext` ×3,
`AppIdentityDbContext`, registered inside `if`/`else`), module-configuration 3 (BlazorAdmin
services registered in BlazorAdmin's own host, not on A's primary roots), no-registration 1
(`CatalogSettings` at `UriComposer`). B: factory 119 (`YesSql.ISession` 62,
`IShellConfiguration` 41 — lambda registrations the resolver cannot read through),
no-registration 49, conditional 18, keyed 13; scanning 0, decorator 0.

**Registration-not-read, by call.** A 42: `AddIdentity` 31 (`UserManager<>` 16,
`SignInManager<>` 8, `RoleManager<>` 7), `AddMemoryCache` 4, `Configure` 3, `AddMetronome` 2,
`AddHttpContextAccessor` 1, `ConfigureHttpClientDefaults` 1. B 659: `Configure` 220,
`AddLogging` 170, `AddAuthorization` 105, `AddHttpContextAccessor` 82, `AddIdentity` 43,
`AddDataProtection` 22, `AddMvc` 9, `AddRouting` 4, `AddSignalR` 2, `AddAntiforgery` 1,
`AddAuthentication` 1.

**Boundary.** A 21 over 7 types: `ILogger<>` 11, `IMapper` 3, `ILocalStorageService` 2,
`ILoggerFactory` 2, `IPipelineBehavior<,>` 1, `Identity.UI.Services.IEmailSender` 1,
`UrlEncoder` 1. B 138 over 15 types: `IHtmlLocalizer<>` 66, `IHostEnvironment` 29,
`HtmlEncoder` 11, `IWebHostEnvironment` 8, `IHostApplicationLifetime` 3, `JavaScriptEncoder` 2,
GraphQL types 3, others 16.

**Collection injection** (P-6, its own classification). A 0. B 139: resolved 124,
no-registration 10, factory 3, keyed 1, boundary 1.

**Table coverage** (CG-R-79, after O-19). A: of 84 reached external registration calls the
resolver parses 49, the table knows 18, **17 unknown** (14 distinct — Blazor, OpenTelemetry,
`AddDefaultUI`, `AddDefaultTokenProviders`, `AddOAuth`, `AddCookie`, …). B: of 1,148 reached the
resolver parses 980, the table knows 44, **124 unknown** (45 distinct), ranked by frequency in
the report; the first pass printed 380 unknown of 1,404 before O-19.

**Handlers (tracked, A).** `Mediator.IRequestHandler<,>` 2 of 2 reached,
`INotificationHandler<>` 1 of 1 reached — through `IMediator` (a service since P-1) and the
generated wrappers' `resolve` edges. Run 5's 0 of 3 was O-13.

### The labelled conventions

| Run | Scored / composition | Coverage | Not-read inside | Unresolved |
|---|---|---|---|---|
| A main only | 35 / 61 | 85.7% | 30/47 = 63.8% | 5 |
| A every entry point | 41 / 73 | 82.9% | 34/55 = 61.8% | 7 |
| A public | 79 / 226 | 79.7% | 63/83 = 75.9% | 16 |
| B main only | 192 / 293 | 74.0% | 142/258 = 55.0% | 50 |
| B every entry point | 192 / 294 | 74.0% | 142/258 = 55.0% | 50 |
| B main + controllers, *registrations not read* | 703 / 1,050 | 49.5% | 348/920 = 37.8% | 355 |
| B public | 3,419 / 5,264 | 81.5% | 2,787/4,550 = 61.3% | 632 |

---

## 2. Ground truth on A's `Web`, mechanically (CG-R-71, `--ground-truth`)

67 edges at the reader's granularity: 56 from the hand enumeration of committed source, 11 in
the source-generated Mediator code (verified from the emitted generated files after the first
pass listed them as walk extras — the addendum in `ground-truth-A-web.md`).

| Reader recall | Walk recall | Walk precision |
|---|---|---|
| **66/67 (98.5%) over C# source; Razor views not covered** (CG-R-78) | **65/67 (97.0%)** | **65/65 (100%)** |

The two walk misses: the Razor view (R-1) and `UserContextEnrichmentMiddleware → ILogger<>`
(middleware named by `UseMiddleware<T>`, no fact to root on — the stated gap). Precision 65 of
65: nothing the walk called a composition edge failed the criterion on reading the source,
generated code included.

---

## 3. The three predictions, scored (rule in force: not-read outside)

| Solution | Session | CG-R-81 (the rulings author) | Chat prediction | **Measured** |
|---|---|---|---|---|
| **A** | 84–94, centre 89 — **inside** (0.8 under centre) | 87–94, centre 91 — **inside** (2.8 under) | 80–100, centre 90 — inside | **88.2%** |
| **B** | 82–92, centre 87 — **inside** (1.1 over) | 68–80, centre 74 — **outside**, 8.1 above the band's top | 70–95, centre 80 — inside, non-committing (CG-R-82) | **88.1%** |

CG-R-81's own falsifier fired: "If B comes in above 80% I was wrong about how much the retired
readings were hiding." The retired readings were hiding edges that resolve: of the 307
factory-provider edges P-5 released and the 181 P-7 released, the run shows B's scored
population at 1,673 with 1,474 resolved — the newly scored edges resolved at roughly the same
rate as the previously scored ones, not materially lower.

**Expected distributions.** Session, A: conditional first, no-registration second — measured
conditional 4, module-configuration 3 (not predicted: BlazorAdmin's second host),
no-registration 1. Session, B: factory first, no-registration second, keyed third, conditional
minor — measured factory 119, no-registration 49, conditional 18, keyed 13: the third and fourth
places are swapped. CG-R-81, A: "one no-registration and three conditional" — measured 1 and 4,
plus the 3 module-configuration.

**Under the run-7 rule** (not-read inside) every band is missed by a wide margin: A 54.5%,
B 63.2%. No prediction was made against that rule; none is scored against it.

---

## 4. §12.1, read twice as CG-R-83 requires

Form: fires below 75% under the primary convention, provisional 75–<85%, clears at 85%.

| | Rule in force (not-read outside) | Run-7 rule (not-read inside) |
|---|---|---|
| **A** | 88.2% — **clears** | 54.5% — **fires** |
| **B** | 88.1% — **clears** | 63.2% — **fires** |

The disposition turns entirely on where `registration-not-read` sits. Under the rule the
predictions were made against, both clear. Under the rule CG-R-83 puts in force from run 7 —
the instrument's ignorance of a registration that exists counts against it — both fire, because
26% of B's composition edges and 31% of A's are registrations the reader cannot read.

---

## 5. Findings from the run

- **O-19 (repaired, report figure only).** The registration-knowledge table's "unknown reached
  calls" counted Orchard's own extension methods as external: a type declared by several
  projects carries an `@assembly` suffix in its id, a method id names it without one. 256 of the
  380 "unknown" in the first pass were in-solution methods whose bodies the walk enters. Coverage
  figures are byte-identical across the two passes (diffed); the table line is the only change.
- **O-18 (not repaired — a resolver gap).** Among B's 124 truly external unknown calls, 57 carry
  lifetime names the resolver claims to parse but could not read as a pair:
  `TryAddEnumerable` ×25, `Replace` ×15, `TryAddScoped` ×6, `RemoveAll` ×5, `AddTransient` ×3,
  `Add` ×2, `AddSingleton` ×1 — the `ServiceDescriptor.Transient<I, C>()` static-factory forms,
  not `new ServiceDescriptor(typeof, typeof)`. Their services land in *boundary* or
  *no-registration* today. Repair would move the denominator: proposed for run 7.
- **F-7 (a rule the run made visible).** `ILogger<T>` is *boundary* in A (11 edges: no
  `AddLogging` call in A's source — the host builder registers logging implicitly) and
  *registration-not-read* in B (170 edges: Orchard's own code calls `AddLogging`). The same
  dependency, supplied the same way, is classified by whether a call is textually present.
  Under the run-7 rule that difference decides 170 edges of B's denominator. Proposal for
  ruling: the host builder's implicit registrations (`WebApplication.CreateBuilder` / `Host.
  CreateDefaultBuilder`: logging, configuration, options, hosting environment) are a known
  provider, reported as *registration-not-read (host builder)* wherever the entry point is a
  reached root — so `ILogger<T>` classifies the same in both.
- **Middleware roots.** One ground-truth miss remains for lack of a fact (`UseMiddleware<T>`);
  R-2 widened to `IApplicationBuilder` calls would supply it. Held.

---

## 6. What neither figure establishes

- Whether B's 88% would survive the run-7 rule with a table that knew the 124 unknown calls:
  the not-read set is 659 edges with the table as it is, and the calls it does not know can
  only add to it.
- Anything about the 342 (B) and 45 (A) Razor edges.
- Whether YesSql's `ISession` (62 factory edges) resolves to one implementation — the lambda
  registration is unread, not unresolvable.

## 7. Weakest point

Unchanged and now quantified: the registration-knowledge table. It knows 44 of the 168 distinct-
by-count external calls B reaches that the resolver does not parse, and 18 of 35 in A; the
run-7 headline depends on it directly. Second: the first pass of this run was produced by a
binary that did not carry the CG-R-83 second figure because a build command never ran — caught
on the diff between passes, recorded in `run6-before-O-19/README.md`.

## 8. Held

Gate 1b remains blocked on Emil's act vocabulary. O-18, F-7 and the middleware root are
proposals for run 7; nothing here is ratified by the session.
