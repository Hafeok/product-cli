# Run 8 — the table grown by frequency, both predictions scored, the CG-R-96 figure unscored — 2026-09-14

**Status: measured, scored, held.** Instrument record `gate1a-measurement/run8-instrument.txt`:
commit `4023741`, working tree 0 uncommitted paths captured before the record existed, binary
sha256 `17435095…`, inventories v6 unchanged from run 7 (`3197d7e0…`, `98ce4d0a…`). The script
built the binary it measured with and exited on any failing step. Its closing line still says
"run 7 complete" — a `sed` that renamed the paths did not touch that string; cosmetic, fixed in
the run-9 script, recorded.

---

## 1. The measurement (primary conventions, as run 7)

| | Composition | Scored | **Scored fraction** | Classified | **Error bound** | Resolved | Unresolved | Not-read | Boundary | Partial | Excluded | Coverage (instrument) | Not-read inside (instrument) | **CG-R-96 figure, unscored** |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| **A** | 134 | 68 | **50.7%** | 127 (94.8%) | 7, **5.2%** | 60 | 8 | 59 | 4 | 2 | 1 | 88.2% | 47.2% | 0 resolved + 59 boundary → **88.2%** |
| **B** | 2,626 | 1,714 | **65.3%** | 2,458 (93.6%) | 168, **6.4%** | 1,522 | 192 | 744 | 82 | 74 | 12 | 88.8% | 61.9% | 0 resolved + 744 boundary → **88.8%** |

Blind spot beside each: A 45 `@inject` in 71 Razor files, B 342 in 1,610.

**What moved from run 7.** A: 4 boundary edges became registration-not-read
(`ILocalStorageService` 2 via `AddBlazoredLocalStorage`, `Identity.UI.Services.IEmailSender` 1
via `AddDefaultUI`, `UrlEncoder` 1 via the MVC row); nothing else — the scored fraction is
68/134 exactly. B: the two typed `AddHttpClient<T>` registrations made their client types
container-constructed: +7 composition edges, +2 resolved; 17 boundary edges became not-read
(`AddWebEncoders` 14, `AddAWSService` 2, `AddGraphQL` 2, host builder +1); `HttpClient` itself
appears as a new boundary row (2) — the factory supplies it to the typed clients.

**Registration reader (CG-R-94), four figures.**

| | Reached external calls | Parsed | Known | Opaque | Unknown |
|---|---|---|---|---|---|
| A | 84 | 49 | 35 (run 7: 18) | 0 | **0** (run 7: 17) |
| B | 1,148 | 1,022 | 114 (run 7: 44) | 12 | **0** (run 7: 84) |

**Per new entry, the edges it moved (CG-R-95).** A: `AddBlazoredLocalStorage` 2, `AddDefaultUI`
1, the MVC row's encoders 1. B: `AddWebEncoders` 14, `AddAWSService` 2, `AddGraphQL` 2, host
builder `IServer` 1. **Every other new entry moved no edge** — the OpenTelemetry builder calls,
`AddServiceDiscovery`, `AddOAuth`, `AddCookie`, `ConfigureApplicationCookie`, the delegate
`AddCheck`, `AddDatabaseDeveloperPageExceptionFilter`, `AddServerSideBlazor`,
`AddDefaultTokenProviders`, `AddTokenProvider`, `OptionsBuilder.Configure`, the resilience
handlers, the OpenIddict calls, `AddProblemDetails`, `AddMetrics`, `AddSwaggerGen`,
`AddStackExchangeRedis`, `AddOpenApi`, `AddMiniProfiler`, `AddJsonProtocol`,
`AddHttpMessageHandler`, `AddEndpointsApiExplorer`, `AddAzureSignalR`, `AddAzureClientsCore`,
the data-protection builder calls, `AddResponseCompression`, `AddRateLimiter`, `AddHsts`, the
seven collection/LINQ operations and `RemoveAll`. They are verdicts, not omissions (an empty
entry says the call registers nothing a constructor asks for), and they are what took *unknown*
to zero on both solutions.

**Unresolved by reason.** A: conditional 4, module-configuration 3, no-registration 1. B: factory
118, no-registration 40, conditional 18, keyed 16. Unchanged from run 7.

**Ground truth, A's `Web`.** 66/67, 65/67, 65/65 — the same two misses (Razor view, middleware).
Handlers 3 of 3.

### The labelled conventions (scored fraction; coverage beneath as instrument property)

| Run | Scored fraction | Coverage | Not-read inside |
|---|---|---|---|
| A main only | 35/61 = 57.4% | 85.7% | 52.6% |
| A every entry point | 41/73 = 56.2% | 82.9% | 49.3% |
| A public | 79/226 = 35.0% | 79.7% | 71.6% |
| B main only | 192/293 = 65.5% | 74.0% | 52.0% |
| B every entry point | 192/294 = 65.3% | 74.0% | 51.8% |
| B main + controllers, *registrations not read* | 703/1,050 = 67.0% | 49.5% | 37.0% |
| B public | 3,421/5,271 = 64.9% | 82.2% | 60.2% |

---

## 2. The two predictions, scored

| | Session | Emil (CG-R-97) | **Measured** |
|---|---|---|---|
| A scored fraction | **50.7 exact** — exact | 50.7–52.5, centre 51.0 — inside | **50.7%** |
| A classified | 94–96, centre 94.8 — **exact** | 94–97, centre 95.5 — inside | **94.8%** |
| A not-read inside | 46–48, centre 47.2 — **exact** | 47–51, centre 49.0 — inside, at the edge | **47.2%** |
| B scored fraction | 65.0–65.8, centre 65.4 — inside | 65.0–66.5, centre 65.6 — inside | **65.3%** |
| B classified | 93.0–94.5, centre 93.7 — inside | 93–95.5, centre 94.2 — inside | **93.6%** |
| B not-read inside | 61–63, centre 62.0 — inside | 61–64, centre 62.5 — inside | **61.9%** |

Six of six inside for both. **CG-R-97's stated falsifier fired in the session's favour**: A came
back at 50.7 exact, so no new entry named an in-solution type the container constructs — the
`AddDbContext<CatalogContext>` shape did not occur among the new entries. On B the session's
edge-by-edge reasoning (typed clients as the only way the fraction moves) was the mechanism:
+7 edges, 65.4 → 65.3.

**The unscored figure.** Under CG-R-96, **0** of A's 59 and **0** of B's 744 not-read edges take
*resolved* — every type the table names is a framework type no production type implements — so
all of them take *boundary* and leave the denominator, and the CG-R-96 coverage is the run-6
form exactly: A 88.2%, B 88.8%. Emil's unscored bet of "B in the 70s" did not land; the session
said before the run (§9 of the restatement) that the construction puts the figure at the run-6
form, and it did. What the table earned is visible where the walk can show it: *unknown* to zero
on both, the classified fraction up 3.0 and 0.7 points, the error bound down 3.0 and 0.7 points.

---

## 3. §12.1 — no disposition (CG-R-89)

Error bound on the reachable/isolated split: A 7 of 134 (5.2%), B 168 of 2,626 (6.4%). Reported
with the split when there is one; Gate 1b has not run.

---

## 4. Findings

- **CG-R-96's resolved branch never fires on these solutions.** Every table entry names framework
  types; none has a production implementor. Under CG-R-96 the table's whole effect on coverage is
  to move edges *out* of the denominator, which is why the CG-R-96 figure equals the run-6 form.
  The table's worth is measured by the classified fraction and the error bound, not by coverage.
- **`HttpClient` as a boundary row (B, 2 edges)**: the typed clients' constructors take
  `HttpClient`, which the factory supplies. Candidate for the `AddHttpClient<T>` knowledge row;
  proposed, not applied.
- **The script's closing line** said "run 7 complete": a rename that missed a string. Cosmetic;
  no step was skipped (the instrument record and every output file are run 8's). Fixed for run 9.
- **A fitness gate caught the CG-R-96 addition after the run.** The three CG-R-96 fields took
  `resolution_of` to 44 statement lines against the 40-line limit; `cargo t` failed on the
  code-quality binary after run 8 had already been measured. The gate set had not been run before
  the run (the pre-run gate was the build only, per `run8.sh`). Repaired by extracting the
  partition into `not_read_partition`; a pure extraction, no figure changes, and the run-8
  outputs stand as produced by the binary at `4023741`. Recorded because the order was wrong:
  gates precede a measurement, not follow it.

## 5. Weakest point

The table is now complete against everything runs 7 and 8 reached — and that says nothing
about the next solution. Its coverage figure is over *these* calls; a third codebase will reach
calls it does not know, and the four figures will show it. The other standing weakness is
unchanged: 744 edges in B and 59 in A whose verdict depends on the table being right about what a
framework call registers, written from documentation, checked against no ground truth.

## 6. Held

Gate 1b's act vocabulary is Emil's and blocks §12.1. CG-R-96 is in force from run 9; nothing
here is ratified by the session.
