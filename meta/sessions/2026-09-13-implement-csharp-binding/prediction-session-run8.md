# The session's prediction for run 8 — committed 2026-09-14, against `reader-emission-rules-v5.md` §8

Made before either v6 inventory is walked by the run-8 instrument. Emil's prediction is filed
when it arrives; run 8 runs after both are committed.

| | Scored fraction | Classified fraction | Coverage, not-read inside |
|---|---|---|---|
| **A** | **50.7%, exact** (68/134 — no walk decision changes) | 94–96%, centre **94.8%** (127/134) | 46–48%, centre **47.2%** (60/127) |
| **B** | 65.0–65.8%, centre **65.4%** | 93.0–94.5%, centre **93.7%** | 61–63%, centre **62.0%** |

**Reasoning, edge by edge where the construction allows it.** A: the new entries relabel
`ILocalStorageService` 2 (`AddBlazoredLocalStorage`, reached), `Identity.UI.Services.IEmailSender`
1 (`AddDefaultUI`, reached) and `UrlEncoder` 1 (`AddRazorPages` → the MVC row) from boundary to
not-read: +4 classified, 0 resolved. `IMapper` 3 stays boundary: `AddAutoMapper` is called in
PublicApi's host, not on A's primary roots. B: `HtmlEncoder` 11 and `JavaScriptEncoder` 2
(`AddWebEncoders` reached ×1, and the MVC row), GraphQL 3 (`AddGraphQL`), `IAmazonS3` 2
(`AddAWSService<T>`), `IServer` 1 (host) — about 19 relabelled; the two typed `AddHttpClient<T>`
registrations may add a handful of composition edges from the client types, which is the only
way B's scored fraction moves. Table figures: A unknown 17 → 0; B unknown 84 → the opaque
lifetime calls (about 17) reported apart, unknown near 0.

**Where this is most likely wrong.** B's typed clients: if each opens a constructor with several
dependencies, the scored fraction shifts a few tenths and the classified fraction with it.
