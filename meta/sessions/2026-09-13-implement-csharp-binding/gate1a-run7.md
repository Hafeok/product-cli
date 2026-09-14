# Run 7 — instrument v6, both readings of the scored fraction, the CG-R-89 error bound, two predictions scored — 2026-09-14

**Status: measured, scored, held.** Instrument record: `gate1a-measurement/run7-instrument.txt`
(commit `3bda245`, binary sha256 `4e9c7898…`, inventories v6 `3197d7e0…` and `98ce4d0a…`); the
script built the binary it measured with and exited on any failing step (CG-R-88). Its "1
uncommitted path" is the record file itself — the redirection creates it before `git status`
runs inside it; the tree was otherwise clean. Fixed in the next script, noted here.

---

## 1. The measurement (primary conventions, as run 6)

| | Composition | Scored (1st reading) | **Scored fraction** | Classified (2nd reading) | **Error bound** (CG-R-89) | Resolved | Unresolved | Not-read | Boundary | Partial | Excluded | Coverage, run-6 form | **Coverage, not-read inside** |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| **A** | 134 | 68 | **50.7%** | 123 (91.8%) | 11 unscored, **8.2%** | 60 | 8 | 55 | 8 | 2 | 1 | 60/68 = 88.2% | **60/123 = 48.8%** |
| **B** | 2,619 | 1,712 | **65.4%** | 2,434 (92.9%) | 185 unscored, **7.1%** | 1,520 | 192 | 722 | 99 | 74 | 12 | 1,520/1,712 = 88.8% | **1,520/2,434 = 62.4%** |

Blind spot beside each (CG-R-78): A 45 `@inject` in 71 Razor files; B 342 in 1,610. Reached: A 231
of 320 production types, B 3,704 of 5,864.

**What moved from run 6.** A: the host-builder provider moved `ILogger<>` (11) and
`ILoggerFactory` (2) from *boundary* to *registration-not-read (host builder)*; nothing else.
B: O-18 read 46 more registrations — composition edges rose 2,552 → 2,619 (67 new edges from
types those registrations construct), resolved 1,474 → 1,520, unresolved 199 → 192; the host
provider moved 43 edges (`IHostEnvironment`, `IWebHostEnvironment`, `IHostApplicationLifetime`)
to *registration-not-read (host builder)*; boundary 138 → 99.

**Unresolved by reason.** A: conditional 4, module-configuration 3, no-registration 1 (unchanged).
B: factory 118, no-registration 40, conditional 18, keyed 16.

**Registration-not-read by call.** A 55: `AddIdentity` 31, host builder 13, `AddMemoryCache` 4,
`Configure` 3, `AddMetronome` 2, `AddHttpContextAccessor` 1, `ConfigureHttpClientDefaults` 1.
B 722: `Configure` 226, `AddLogging` 179, `AddAuthorization` 107, `AddHttpContextAccessor` 83,
`AddIdentity` 43, host builder 43, `AddDataProtection` 23, `AddMvc` 10, `AddRouting` 4,
`AddSignalR` 2, `AddAntiforgery` 1, `AddAuthentication` 1.

**Boundary.** A 8 over 5 types (`IMapper` 3, `ILocalStorageService` 2, `IPipelineBehavior<,>`,
`Identity.UI.Services.IEmailSender`, `UrlEncoder`). B 99: `IHtmlLocalizer<>` 66 (no
`AddViewLocalization` reached; `AddMvc`'s table row does not list it), `HtmlEncoder` 11,
`IAmazonS3` 2, `JavaScriptEncoder` 2, GraphQL 3, `IServer` 1, others 14.

**Table coverage** (CG-R-79). A: of 84 reached external calls — parsed 49, known 18, unknown 17.
B: of 1,148 — parsed 1,020 (run 6: 980; O-18 moved 40 into parsed), known 44, unknown 84.

**Collection injection.** A 0; B 141.

**Ground truth, A's `Web`, 67 edges.** Reader recall 66/67 over C# source, Razor views not
covered; walk recall 65/67; walk precision 65/65. The two misses are the Razor view and the
middleware, as before. Handlers: 3 of 3 reached.

### The labelled conventions

| Run | Scored fraction | Classified | Coverage | Not-read inside |
|---|---|---|---|---|
| A main only | 35/61 = 57.4% | 90.2% | 85.7% | 54.5% |
| A every entry point | 41/73 = 56.2% | 90.4% | 82.9% | 51.5% |
| A public | 79/226 = 35.0% | 36.7% | 79.7% | 75.9% |
| B main only | 192/293 = 65.5% | 92.5% | 74.0% | 52.4% |
| B every entry point | 192/294 = 65.3% | 92.2% | 74.0% | 52.4% |
| B main + controllers, *registrations not read* | 703/1,050 = 67.0% | 89.0% | 49.5% | 37.3% |
| B public | 3,421/5,271 = 64.9% | 87.4% | 82.1% | 61.0% |

---

## 2. The two predictions, scored (both against the first reading, CG-R-90)

| | Session | Emil (CG-R-91) | **Measured** |
|---|---|---|---|
| A scored fraction | 48–54, centre 51 — **inside** (0.3 under) | 49–54, centre 51 — **inside** | **50.7%** |
| A coverage, run-6 form | 86–91, centre 88 — inside | 87–92, centre 89 — inside | 88.2% |
| A coverage, not-read inside | 45–52, centre 49 — **inside** (0.2 under) | 50–55, centre 52 — **outside**, 1.2 below the band | **48.8%** |
| B scored fraction | 63–69, centre 66 — **inside** (0.6 under) | 64–69, centre 66 — **inside** | **65.4%** |
| B coverage, run-6 form | 86–91, centre 88.5 — inside | 88–92, centre 90 — inside | 88.8% |
| B coverage, not-read inside | 58–66, centre 62 — **inside** (0.4 over) | 63–68, centre 65 — **outside**, 0.6 below the band | **62.4%** |

**CG-R-91's stated falsifier fired, narrowly.** B's not-read-inside coverage came in at 62.4%,
below run 6's 63.2%. The mechanism is visible in the counts: O-18's registrations did lift the
numerator (+46 resolved, −7 unresolved), but they also made 67 new composition edges (the
constructors of the types those registrations construct) and the host provider added 43
not-read edges to the denominator — the repair reached edges on both sides of it, and the
denominator grew faster than the numerator. A's not-read-inside fell to 48.8% for the reason
Emil named (13 host-provider edges enlarging the denominator without a resolution), further
than his band allowed.

**Expected unresolved distributions.** Session, A: conditional 4, module-configuration 3,
no-registration 1 — exact. Session, B: factory first, no-registration second, conditional
third, keyed fourth — measured 118 / 40 / 18 / 16, as predicted.

---

## 3. §12.1 — no disposition (CG-R-89)

The fire/clear form is retired. What run 7 produces is the **error bound on the
reachable/isolated split**: the unscored fraction under the second reading —

| | Unscored (partial + boundary + excluded) | Error bound |
|---|---|---|
| A | 11 of 134 | **8.2%** |
| B | 185 of 2,619 | **7.1%** |

— to be reported with the split when there is one. The split needs the delta over an act
vocabulary; Gate 1b has not run. No disposition is taken.

---

## 4. Findings

- **The instrument record counted itself** as the one uncommitted path (the redirection creates
  the file before `git status` runs inside the braces). The run-8 script captures the status
  before opening the record. No measurement is affected.
- **`IHtmlLocalizer<T>` at 66 boundary edges in B** is the largest remaining boundary row: it is
  registered by `AddViewLocalization` on `IMvcBuilder`, chained off `AddMvc()` in Orchard's own
  `OrchardCoreBuilderExtensions.AddMvc` — an in-solution extension the walk enters, whose inner
  `AddViewLocalization` call is a reached external call the table knows (row present) but whose
  receiver chain was not recorded because the chain runs through a *variable*, not a single
  statement (rule 11's stated limit). Reported, not repaired.
- **Emil's table-coverage figure moved by O-18 alone**: B parsed 980 → 1,020, unknown 124 → 84.
  The 44 known are unchanged; the table did not learn, the resolver did.

## 5. Weakest point

The same one, measured again: 84 reached external calls in B and 17 in A that neither the
resolver parses nor the table knows, and 722 edges (28% of B's composition edges) whose only
verdict is *a framework call the resolver does not parse supplies it*. Under the rule now in
force those edges count against coverage, which is what CG-R-83 intended; the figure that would
move them is the table's, and the table has not grown since it was written.

## 6. Held

Gate 1b's act vocabulary is Emil's and is now the blocker for §12.1 as well (CG-R-89). Nothing
here is ratified by the session.
