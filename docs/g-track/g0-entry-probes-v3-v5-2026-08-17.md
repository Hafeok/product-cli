# G0 entry probes — V3(a) and V5

**Session type:** verification. Reports, does not rule.
**Date:** 2026-08-17. **All findings pinned to the refs below.**

> **Status update — Emil ruled on this report the same day.** V5 accepted as wiring-only. Kotlin
> ruled **conditionally in**, pending one narrow follow-up: the kotlin-lsp workspace-import probe.
> The proposed PRD edits below were ruled in **and have since been applied** to
> `docs/g-track/prd-ground-as-ontology.md` — read them here as the rationale, not as pending
> work. Two edits were added on the ruling: the **capability-flag discipline** generalised
> extractor-wide (§5.2), and the **foreign-key row marked not-derivable** from the declaration
> subset rather than left as an empty result (§5.2). The three-project-subset caveat was carried
> into §5.2 as an explicit provisionality note, and the discharged Oxigraph sentence (§3, §11,
> §12 item 8) was updated in the same pass.

## Findings at a glance

| Check | Status | One-line result |
|---|---|---|
| **V3(a).1** Gradle ≥ 8.8 floor | **Discharged** | Gradle **8.14.3**, AGP 8.13.0, Kotlin 2.2.10 — clears comfortably |
| **V3(a).2** JDK compatibility | **Discharged** | Project targets JDK 17; kotlin-lsp ships its own **JBR 25.0.2** — no conflict, nothing to install |
| **V3(a).3** Project shape | **Discharged** | 25 modules + composite build, no KMP, 3 flavours × 2 build types = 6 variants |
| **V3(a).4** kotlin-lsp import run | **Partial — the one open item** | Server ran; **workspace import never started** (2 handshakes). But an independent Gradle control enumerated **all six variants cleanly** |
| **V3(a).5** Kotlin synthesis route | **Discharged, premise corrected** | typeHierarchy is **advertised but returns `null`**; `hover` returns the supertype clause, so synthesis is *cheaper* than assumed — but Kotlin's subClassOf edges are ~75% UI state |
| **V5.1** `definition` + `typeHierarchy` | **Discharged — closed** | Roslyn 5.11.0 answers **both**, both hierarchy directions, on production code. Wiring only |
| **V5.2** Six-operation coverage | **Discharged** | All six return usable results, with examples |
| **V5.3** §5.2 rows vs real C# | **Discharged** | Five of six rows fire; the **foreign-key half finds nothing** |

**Net:** V5 is closed and closed favourably. V3(a) is three-and-a-half conditions of four; the
single open question is whether kotlin-lsp can be made to import a Gradle/AGP workspace, and
that question — not the Gradle floor, not typeHierarchy — is what now decides n = 2 versus
single-source.

## Refs read

| Repo | Branch | HEAD | Head commit date | Notes |
|---|---|---|---|---|
| `acme-app/android` | `develop` (default) | `67340f69b4b4fa3230a9371831c979c1c83175b5` | 2025-10-02 | Session branch is at the same commit; `git ls-remote --symref origin HEAD` confirms `develop` is the default branch and is at this SHA. The repo has genuinely not moved since Oct 2025 — not a stale clone. |
| `acme-app/backend` | `main` (default) | `83c9339a47f8f01dfc28fe138691559d40536e7f` | 2026-08-13 | Session branch at the same commit; `main` confirmed default and at this SHA. |
| `Hafeok/product-cli` | — | `d2bf37d0bdfabc3af32647cf55a1e9a9a0a40065` | 2026-08-17 | Merge of PR #44; PRD + C# adapter as merged (PR #43 + #44). |

> **Corpus names are pseudonyms.** The two client repositories are private, and this repository
> is public, so their organisation, repository, namespace, package and domain-type names are
> replaced throughout with neutral placeholders under an `acme-app` / `Acme.App` / `com.acme`
> scheme. The substitution is **consistent and structure-preserving**: every count, ratio,
> version, path shape, declaration form and LSP result is the real measurement, and the commit
> SHAs are the real pins — only the identifying strings are stand-ins. Where a literal name was
> itself the evidence (the exact-name intersection list), the finding is characterised rather
> than enumerated, and the real names remain recoverable from the corpus at the pinned refs by
> anyone with access.

Neither corpus repository was written to. Verified after all work: zero tracked-file
modifications, zero non-ignored untracked files, HEADs unchanged in both. Build by-products
(`obj/`, Gradle caches) were confined to already-gitignored paths or to the scratchpad.

**Method note.** Both corpora were already present as local clones, so the cross-tier
`add_repo` refusal that blocked G-1 did not recur. Server-side facts about kotlin-lsp were
re-verified first-hand from the published distribution (checksum-verified) rather than
inherited from G-1's pin; `github.com` and `api.github.com` are denied by this session's
egress policy (403), but `raw.githubusercontent.com` and `download-cdn.jetbrains.com` are not.

---

## Probe 1 — V3(a): can Kotlin be G0's second leg?

### Results

| # | Check | Finding | Verdict |
|---|---|---|---|
| 1 | Gradle version | **8.14.3** (`gradle/wrapper/gradle-wrapper.properties`, `distributionUrl=…gradle-8.14.3-bin.zip`) | **Clears the ≥ 8.8 floor** |
| 1 | AGP version | **8.13.0** (`gradle/libs.versions.toml`, `gradle-plugin = "8.13.0"`); Android tools 31.9.0 | Current |
| 1 | Kotlin / KSP | Kotlin **2.2.10**, KSP **2.2.10-2.0.2** | K2-era, matches kotlin-lsp's K2 base |
| 1 | SDK levels | compileSdk **36**, targetSdk **36**, minSdk **24** | — |
| 2 | Project JDK target | **17** everywhere — `JavaVersion.VERSION_17` + `JvmTarget.JVM_17` (`build-logic/src/main/java/com/acme/app/KotlinAndroid.kt:33,34,62`; `build-logic/build.gradle.kts:10,11,15`); Azure CI pins `jdkVersionOption: 1.17` | — |
| 2 | kotlin-lsp JDK 25 requirement | **Satisfied by the distribution itself** — the standalone archive bundles JetBrains Runtime **25.0.2** (`jbr/bin/java` → `JBR-25.0.2+10-432.48-nomod`). No JDK 25 needed on the host, and no conflict with the project's JDK 17 target: they are different JVMs (server runtime vs Gradle daemon/toolchain). | **No blocker** |
| 3 | Modules | **25 Gradle modules** (`settings.gradle.kts`) — `:app`, `:api`, `:designsystem`, `:util`, 10 × `:data:*`, 9 × `:core:*` — plus a **`build-logic` composite build** (`includeBuild`) supplying five convention plugins | Large but conventional |
| 3 | Variants | **Multi-variant**: 3 product flavours on one `environment` dimension (`production`, `staging`, `develop` — `AppFlavor.kt`) × 2 build types (`debug`, `release` — `AppBuildType.kt`) = **6 variants for `:app`** | Multi-variant, as expected |
| 3 | KMP | **None.** No `multiplatform` plugin, no `commonMain`; single-target Android/JVM throughout | Simplifies import |
| 3 | Generated sources | KSP, Room (`androidx-room 2.8.0`), protobuf (`:core:datastore-proto` registers per-variant source sets), Navigation safe-args | Import must tolerate generated roots |
| 3 | Other import risks | Config cache + parallel on; `RepositoriesMode.FAIL_ON_PROJECT_REPOS`; one **third-party vendor Maven repository** (unreachable from this session — connection failure, not a policy denial) supplying a single vendor SDK artifact; no `local.properties`, so an Android SDK must come from `ANDROID_HOME` | One private-repo dependency |
| 4 | Live `bin/intellij-server` run | **ATTEMPTED — see below** | — |
| 5 | Synthesis route | **Premise changed — see below** | — |

### Item 4 — the live import run

Unlike G-1, this session could obtain the toolchain: kotlin-lsp **v262.9593.0** standalone
Linux-x64 archive downloaded from `download-cdn.jetbrains.com`, **SHA-256 verified against the
published digest** (`2d99d8e198fbe4aa8f4481e37799724ce94803b4ea12a60b416040e3fcd7cc5e`), Android
SDK platform 36 + build-tools 36.0.0 installed, `ANDROID_HOME` set (no `local.properties`
written into the repo). `bin/intellij-server --stdio` was run against the Android repo at
`67340f69`.

**Result: partial. The Gradle floor is cleared and the variant model demonstrably resolves —
but the kotlin-lsp workspace import did not start, so "variants import *through kotlin-lsp*"
is not demonstrated.**

| Sub-check | Result |
|---|---|
| Server runs headless on Linux | **Yes.** `bin/intellij-server --stdio`, JBR 25.0.2 bundled; no host JDK 25 needed |
| `initialize` against the repo root | **Yes.** Server extensions loaded, incl. `WorkspaceImportLanguageServerExtension` |
| Advertised capabilities | documentSymbol, workspaceSymbol, references, definition, hover, implementation, typeDefinition, **and `typeHierarchyProvider: true`** |
| Per-file analysis | **Works.** `documentSymbol` returned the full nested tree for `BadgeIcon.kt` (class 5, `data object` 19, two `data class` 23, properties, constructors); `hover`, `definition`, `typeDefinition` all answered |
| **Gradle/AGP workspace import** | **Did not run.** `workspace/symbol("Session")` → `array[0]`; `workspace/symbol("BadgeIcon")` → `array[0]`; `references` returned only the in-file occurrence (`array[1]`); the server log contains **zero** occurrences of the string "gradle" |
| **Independent Gradle control** | **All six variants resolve.** `./gradlew :app:tasks --all` on JDK 21 with Android SDK 36 configured the whole project — `assembleDevelopDebug`, `assembleDevelopRelease`, `assembleProductionDebug`, `assembleProductionRelease`, `assembleStagingDebug`, `assembleStagingRelease` (plus AndroidTest/UnitTest variants) — **no failures**. The unreachable vendor repository does not block configuration, because variant collection does not resolve dependency graphs |

**Two handshakes tried, neither started an import.**

1. Plain LSP handshake (`initialize` + `initialized`, workspace folder = repo root): no import;
   zero "gradle" in the server log.
2. Handshake with `initializationOptions: {importWorkspace: true, gradle: {enabled: true}}`,
   `executeCommand` capability declared, followed by an explicit
   `workspace/executeCommand{command: "exportWorkspace"}`. The command **exists but rejected the
   call** — `-32602 "Expected 1 argument, got: 0"` — and its name indicates an *export* of the
   workspace model, not an import trigger. After a further four-minute settle,
   `workspace/symbol` still returned **0** for `Session`, `BadgeIcon` and
   `InstallationId`, and the only occurrence of "gradle" in the server log was **my own
   `initializationOptions` echoed back**.

**What this separates.** The ≥ 8.8 floor is not the obstacle: Gradle is 8.14.3, and the
project's variant model configures cleanly and enumerates exactly the six expected variants
under Gradle itself. The obstacle is that **kotlin-lsp never began an import** in this
environment. This session did **not** establish *why*, and says so rather than guessing.
Unexplored candidates for a follow-up: the argument `exportWorkspace` expects; whatever
`initializationOptions` the "Kotlin by JetBrains" VS Code extension v0.0.6 actually sends (it
ships in the same release and would be the authoritative reference client); and whether the
headless server needs a project to be opened by an explicit non-standard notification.

Reported as a partial discharge of §5.4 condition (a), which has two halves:

- **"Gradle clears ≥ 8.8"** — **discharged.** 8.14.3, and variant resolution independently
  confirmed under Gradle itself.
- **"one `bin/intellij-server` import run confirms variants resolve"** — **not discharged.**
  Attempted, did not start. Not simulated.

### Item 5 — the synthesis route

**A capability trap, found live and worth naming.** kotlin-lsp v262.9593.0 **advertises
`typeHierarchyProvider: true`** in its `InitializeResult` — but `textDocument/prepareTypeHierarchy`
returned **`null`** on every anchor tried, including a nested sealed subclass
(`NoIcon : BadgeIcon()`) and a top-level data class (`SessionPrice`).
`textDocument/implementation` likewise returned `null`.

This **confirms G-1's source reading at runtime** ("API scaffolding exists, no provider is
registered") against the pinned release, and adds a detail G-1 could not see from source: the
failure mode is a **silent `null`, not a `MethodNotFound` error**. An adapter that gates on the
advertised capability — the natural implementation — will conclude typeHierarchy is available
for Kotlin and then quietly extract **zero** `rdfs:subClassOf` edges. That is the worst
available failure mode for an extractor whose output is a proposal graph a human reviews:
absence of edges reads as "this codebase has no hierarchy", not as "the instrument did not
answer". **The Kotlin adapter must treat typeHierarchy as unavailable by probe result, never by
capability flag.**

I also re-verified the release line independently: `Kotlin/kotlin-lsp` `RELEASES.md` fetched
2026-08-17 still has **v262.9593.0** at the top — unchanged since G-1 pinned it on 2026-07-27.
No typeHierarchy implementation has shipped in the interval.

**The good news: the synthesis route is cheaper for Kotlin than for C#.** `hover` returns the
**server-rendered declaration header, including the supertype clause**:

- `NoIcon` → ` ```kotlin\ndata object NoIcon : BadgeIcon()\n``` `
- `SessionPrice` → ` ```kotlin\ndata class SessionPrice(itemId: String?, pricePerUnit: Double?, currency: String?, unit: String?)\n``` `

So the Kotlin `subClassOf` route need **not** slice raw file text at all — the supertype clause
arrives already normalised, per-symbol, from the server, and it works **without a workspace
import** (both results above came from the un-imported, per-file mode). The second hover also
delivers the **full property list with types and nullability**, which feeds §5.2's typed-property
row directly. This is a materially better route than the C# adapter's file-text slicing, and it
is available today.

**First, what the C# adapter actually carries.** Read at `d2bf37d`:
`ddd-lsp/src/adapter/csharp.rs` + `csharp_facts.rs`. The pattern is LSP `documentSymbol` for
names/kinds/ranges, plus **declaration-text slicing** (`declaration_slice`,
`declared_visibility`, `cap_visibility`, `strip_visibility`, `attribute_lines`) to recover
normalised kind, container-capped effective visibility, body-free signature and attached
attributes.

**The adapter carries no `rdfs:subClassOf` synthesis today.** A workspace-wide search for
`subClassOf` / `base_type` / `supertype` / `typeHierarchy` returns nothing relevant
(`rust_facts::inherits_container` concerns *visibility* inheritance, not type inheritance). The
adapter is a **contract-surface classifier**, not an ontology extractor. So what transfers to
Kotlin is the **technique**, not code. §5.2's phrase "the pattern the C# adapter already
carries" is accurate about the technique and should not be read as "an implementation exists to
port" — a later session sizing this work should budget for a new component in both languages.

**Does the technique transfer to Kotlin as this codebase writes it?** Mechanically, largely
yes: Kotlin puts the supertype list on the declaration line just as C# does, and
`declaration_slice`'s cut at `{` / `;` / top-level `=` with paren-depth tracking is broadly
right for Kotlin class headers. Four cases degrade or break it:

| Case | How this repo writes it | Transfers? |
|---|---|---|
| **Supertype-call syntax** | `data class TrailingIcon(@DrawableRes val res: Int) : BadgeIcon()` — the supertype is written as a *constructor call* | Yes, with work: the slicer must strip the supertype's own argument list, whose parens re-open depth after the `:`. New case, not a blocker |
| **Delegation** | `data class LocationId(val value: String) : Parcelable, Comparable<String> by value, CharSequence by value` (`data/location/…/LocationId.kt`); also `class ImmutableSet<out T>(…) : Set<T> by protectedSet` (`app/…/CollectionWrappers.kt:22`) | **Lossy, and it lands on domain types.** A slicer emits `LocationId subClassOf Comparable` and `subClassOf CharSequence` — true in Kotlin, but these are **implementation conveniences, not domain facts**. `by` forwards members rather than inheriting them. This is proposal-graph *noise* on exactly the types that matter most, and a reviewer must reject it triple by triple |
| **Nested sealed hierarchies** | `sealed class BadgeIcon { data object NoIcon : BadgeIcon(); data class TrailingIcon(…) : BadgeIcon() }` — the dominant idiom (50 sealed classes, 6 sealed interfaces) | **Easier than C#**, not harder: containment *and* the supertype clause both carry the edge. But a slicer keyed only on the clause will double-count, and `data object` is a *value* that nonetheless participates in the hierarchy |
| **`typealias`** | `typealias InstallationId = String` (`app/…/io/model/Models.kt:70`) | **Irrecoverably lossy.** No LSP operation and no text slice recovers "InstallationId is a distinct domain identity" — after resolution it *is* `String` |

`@JvmInline value class`: **zero occurrences**, so Kotlin's identity types are ordinary data
classes rather than a zero-cost value-object construct.

**Identity modelling — a structural agreement and a two-sided inconsistency.** Kotlin *does*
model identity as types: `data class LocationId(val value: String)`, `DeviceId`, `UnitId`,
`ConnectorId` (all in `data/location/…/model/`), each wrapping a single `String`. That is
structurally the **same idea** as C#'s `public readonly record struct LocationId(string Value)`
— and `LocationId` is one of the 23 exact-name matches. This is a genuine triangulation
candidate, not a divergence.

The divergence is narrower and more interesting than "one side types identity and the other does
not": **both codebases apply the convention inconsistently, in different places.** On the C#
side only `LocationId` of four sibling identity types declares `: IStringIdentity`. On the
Kotlin side four identity types exist under `data/location`, while `InstallationId` is a bare
`typealias … = String` in `app/…/io/model/Models.kt`. So the extractor would surface a
**partially-applied identity convention on each side independently**, before any cross-codebase
comparison runs — two instances of the same escaped decision, in two languages. The `typealias`
case remains irrecoverable by any LSP route; the four real identity types are recoverable, but
only through `hover` (see above), since Kotlin `typeHierarchy` returns `null`.

**Where Kotlin's subClassOf edges actually are — the finding that matters most.** Census over
965 non-test `.kt` files:

| Measure | Count |
|---|---|
| `data class` / `data object` declarations | **606** |
| …carrying a supertype | **201** |
| …bare, no supertype | **405** |
| `class … : Supertype` declarations | 123 |
| sealed class / sealed interface / interface / enum class / abstract class | 50 / 6 / 56 / 94 / 13 |
| `typealias` | 2 |
| `@JvmInline value class` | 0 |

Of the 51 files holding supertype-carrying data declarations: **33 under `app/`**
(UI/presentation), 5 under `designsystem/`, 6 under `core/`, **5 under `data/`**, 2 under
`api/`. So **~75% sit in the UI and design-system layers** — `ProfileAction`,
`InstallationUiStatus`, `BadgeIcon`, `ButtonType` — precisely §5.3's "client-only → UI state,
or a divergence" row, which the reviewer classifies and which does *not* enter the shared
domain.

Sharper still: across the three `data/*/domain/model/` directories there are **10 data classes
and not one carries a supertype**. `Session` is typical — 11 properties, no base type.
The five `data/` files that *do* carry supertypes are four identity types plus one sealed
status (`CustomerProfileStatus`), and the identity types' supertypes are the `by`-delegation
noise described above. **The shared-domain layer of this client contributes essentially no
genuine `subClassOf` edges.**

**Consequence: implementing Kotlin `subClassOf` synthesis buys comparatively little for the
shared-domain intersection**, whichever route is taken. It is worth doing for completeness of
the client's own graph; it is not what makes or breaks n=2.

**A second shape finding — DTO/domain twinning.**
`data/sessions/domain/model/Session.kt` and
`data/sessions/remote/model/SessionResponse.kt` declare **structurally identical**
11-property data classes, differing only in the nested price type
(`SessionPrice` vs `PriceResponse`). The Backend has its own entity/DTO split.
The intersection is therefore computed over a 2×2 of (backend entity, backend DTO) ×
(client domain, client DTO), and naive class-candidate extraction **inflates the candidate
count on both sides** before matching runs.

**Cross-codebase overlap, measured.** Declared type names: **942** (C#, `src/`) vs **1090**
(Kotlin, non-test). **Exact-name intersection: 23** (~2% of either side). By character, the
matches split into three groups: **identity and location concepts** (the largest group, and the
one carrying the identity finding below), **subscription and payment concepts**, and a tail of
**generic infrastructure names** that are coincidental rather than domain agreement — a
reviewer would reject that tail on sight. *(The literal names are sector-specific and are
withheld here; they are recoverable from the corpus at the pinned refs.)* This is the concrete
size of the naive intersection, and it is why §5.2's lexical-plus-structural ontology matching —
not exact-name matching — is the load-bearing step. A reviewer told "here are 23 agreements"
would be seeing a small fraction of the real overlap, **and a fraction that is itself part
noise**.

---

## Probe 2 — V5: closing the C# adapter's gap

### How the live probe was obtained

The PRD pins `roslyn-language-server` at 5.11.0. That tool is not on nuget.org, and the official
prerelease feed (`pkgs.dev.azure.com`) is **403 under this session's egress policy**. The server
was instead obtained by downloading the `ms-dotnettools.csharp` VS Code extension **v2.148.23
(linux-x64)** from the marketplace `vspackage` endpoint and extracting the bundled
**`Microsoft.CodeAnalysis.LanguageServer 5.11.0-1.26380.4`** — the exact pinned 5.11.0 line,
exposing the same `--stdio --autoLoadProjects` entry point the adapter declares as its
`default_command`. .NET SDK **10.0.110** installed (the repo targets `net10.0`, `global.json`
sdk `10.0.0`).

**Scope of the probe.** The Backend's own solution includes projects that depend on the private
`AcmePackageFeeds` Azure feed (403 here). A **restorable subgraph** was used instead:
`…Backend.Sql` → `…Options`, `…Primitives` — zero private `PackageReference`s, restored from
nuget.org, driven through a scratch `.sln` created **outside** the repo. `…Sql` is the EF-entity
and CQRS project, i.e. precisely the declaration shapes the extractor cares about. This is
**real production C#**, reduced to a subset — not a synthetic or substitute project. The
subset excludes `…Domain`, `…Dto`, `…Providers` and the API host, which is stated as a limit
below rather than papered over.

All requests were issued **through the shipped adapter host layer** (`ddd_lsp::host::Host` with
`adapter::for_language("csharp")`), so the readiness signal, solution-open handshake and retry
path are the ones in the repo, not a bespoke client.

### Item 1 — the two unwired operations, live

Adapter readiness fired unchanged: `{"state":"ready","elapsed_ms":2604}` — Roslyn sent
`workspace/projectInitializationComplete`, which is exactly the `ReadySignal` the C# adapter
declares. No adapter change was needed to reach a ready host.

| Operation | Roslyn answers? | Result shape | Evidence (anchor → result) |
|---|---|---|---|
| `textDocument/definition` | **Yes** | `Location[]` | `UpsertProfileSettingsCommand` → `array[1]`, self-location at 4:14 |
| `textDocument/prepareTypeHierarchy` | **Yes** | `TypeHierarchyItem[]` | → `array[1]`, item carries `data.ProjectGuid` + `data.SymbolKeyData` |
| `typeHierarchy/supertypes` | **Yes** | `TypeHierarchyItem[]` | `UpsertProfileSettingsCommand` → **`ICommand`**; `LocationId` → **`IStringIdentity`** |
| `typeHierarchy/subtypes` | **Yes** | `TypeHierarchyItem[]` | `ICommand` → **`array[3]`** (the three command records); leaf types → `array[0]` |

**This closes the PRD's open question.** §5.4's C# row says "probe whether the Roslyn server
answers typeHierarchy". It does — **both directions**, with correct results on production
declarations. §5.2's `rdfs:subClassOf` row therefore needs **no synthesis for C#**; it needs
wiring only, and stays on the "high — declared" assurance line.

**Wiring the adapter needs** (concrete, read against `d2bf37d`):

1. **No host-layer change.** `Host::request(method, params)` is method-generic (`host.rs:200`);
   both operations are additive call sites, as the PRD assumed.
2. **`definition` is nearly free.** The result is `Location[]`, already handled by
   `protocol::normalize_locations`, which accepts `Location | Location[] | LocationLink[]`
   (`protocol.rs:147–174`). Position params already exist (`protocol::position_params`).
   **No new normaliser.**
3. **`typeHierarchy` is three call sites, not one** — `textDocument/prepareTypeHierarchy`, then
   `typeHierarchy/supertypes` and `typeHierarchy/subtypes`.
4. **`typeHierarchy` does need a new normaliser.** Items are not `Location`s: they carry
   `name`, `kind`, `detail`, `uri`, `range`, `selectionRange`, `data`. Reusing
   `normalize_locations` would silently keep uri+range and **drop the symbol identity** — the
   one thing the subClassOf row needs.
5. **The opaque `data` field must round-trip verbatim.** Roslyn returns
   `data: {ProjectGuid, SymbolKeyData, TextDocument}`; supertypes/subtypes resolve only when the
   prepare item is passed back unchanged. This is the single wiring detail most likely to be
   got wrong.
6. **Client capability declaration is not the blocker.** `host.rs::initialize` does *not*
   advertise `textDocument.typeHierarchy`, and Roslyn answered anyway. Declaring it remains
   correct practice, but its absence is not what has been stopping the operation.

### Item 2 — declaration-level coverage on real code

All six operations return usable results on this codebase's actual declarations.

| Operation | Usable? | Example (real declaration at `83c9339a`) |
|---|---|---|
| `documentSymbol` | **Yes** | `Entities/ProfileSettings.cs` → 15 symbols: the namespace, the class, and 13 properties **carrying their types in the symbol name** — `Id : ProfileId`, `CustomerId : CustomerId`, `DisabledReason : int?`, `DepartureTime : TimeOnly?` |
| `workspaceSymbol` | **Yes** | `"ProfileSettings"` → `array[15]`, each with `containerName: "project Acme.App.Backend.Sql (net10.0)"`; `"ICommand"` → `array[2]` |
| `typeHierarchy` | **Yes** | `ICommand` → 3 subtypes; `LocationId` → supertype `IStringIdentity` |
| `references` | **Yes** | `UpsertProfileSettingsCommand` → `array[9]`; `ICommand` → `array[5]`; `LocationId` → `array[8]` |
| `definition` | **Yes** | `LocationId` → `array[1]`, resolving to its declaration in `…Primitives/ValueObjects/LocationId.cs` |
| `hover` | **Yes** | `LocationId` → `"readonly record struct …LocationId\nIdentifies a location."` — **the XML doc summary comes through**, feeding the low-assurance `skos:altLabel` row |

The `documentSymbol` result is stronger than §5.2 assumes: the **"typed properties, foreign
keys" row lists `documentSymbol, hover`, but on this codebase `documentSymbol` alone already
carries the property type**, including nullability. `hover` is a refinement, not a requirement,
for that row.

### Item 3 — extraction-relevant shape, and which §5.2 rows fire

How domain entities are actually declared here (census over `src/`, 951 `.cs` files): **662
classes, 127 interfaces, 98 records** (74 positional), **74 enums, 4 structs**; **113
declarations carry a base/interface list**.

The Backend uses four distinct declaration idioms, and they do not extract equally:

- **EF entities** — `internal sealed class ProfileSettings` with `required public`
  auto-properties, and `internal class Location` with a private parameterless constructor for
  EF materialisation. Note **`internal`**, not public.
- **CQRS messages** — positional records implementing marker interfaces:
  `public record UpsertProfileSettingsCommand(…) : ICommand`.
- **Generic handler interfaces** — `ICommandHandler<TCommand> where TCommand : ICommand`.
- **Strongly-typed identities** — `public readonly record struct CustomerId(string Value);`

| §5.2 row | Fires here? | On what |
|---|---|---|
| Classes, records, entities, aggregates → `owl:Class` | **Fires strongly** | 662 classes + 98 records + 4 record structs; `documentSymbol`/`workspaceSymbol` return them all with container paths |
| Inheritance, interface implementation → `rdfs:subClassOf` | **Fires, but thinly** | Only **113 of ~890** type declarations carry a base/interface list. Confirmed live via typeHierarchy in both directions |
| Typed properties, foreign keys → object/datatype properties with ranges | **Fires strongly** | Property types arrive inline on `documentSymbol` (`Id : ProfileId`), so ranges are recoverable without hover |
| Composition and reference → relations (mid, usage-inferred) | **Fires** | `references` returns 5–9 sites per type on this subset; this is the CONSTRUCT-to-fixpoint input |
| Namespaces, modules → domain axis registries | **Fires** | `workspaceSymbol` returns `containerName: "project …Sql (net10.0)"`; the 16-project split is a clean structural axis |
| Naming, comments → synonyms / `skos:altLabel` | **Fires** | XML `<summary>` reaches `hover` verbatim |

**Two findings the extraction table did not anticipate.**

**(a) Foreign keys are not declared, so the FK half of row 3 finds nothing.** The row reads
"typed properties, **foreign keys**". This codebase has no navigation properties and no
`[ForeignKey]` attributes on the entities probed: `ProfileSettings.CustomerId` is a
`CustomerId` value object, not a relationship to a `Customer` entity, and there is no
`Customer` type in the model at all. Relational structure lives in `AppDbContext` /
`Migrations`, i.e. in Fluent-API configuration and generated migration code, **not** in the
declaration surface the extractor reads. Row 3 will therefore yield **datatype properties with
value-object ranges and no object properties** from entities like these. This is a real limit
of the declaration-level subset, not a defect in the server.

**(b) The identity marker is applied asymmetrically — an extraction-surfaced inconsistency.**
Of four sibling identity types in `…Primitives/ValueObjects/`, **only `LocationId` declares
`: IStringIdentity`**; `CustomerId`, `ProfileId` and `DocumentId` do not. The live
probe reflects this exactly: `LocationId` has a supertype edge, the other three would have
none. This is not cosmetic — `StringIdentityJsonConverterFactory.CanConvert` gates on
`typeof(IStringIdentity).IsAssignableFrom(typeToConvert)`, so only `LocationId` receives the
plain-JSON-string treatment the interface's own doc comment describes as its purpose. Whether
that asymmetry is intentional is the reviewer's call, not this session's. It is reported here
because it is a clean instance of §5.3's "escaped decision in the wild" surfacing from a single
codebase before any cross-codebase comparison has run.

### Limits of this probe, stated

- Ran on the `Sql`/`Options`/`Primitives` subgraph only; `Domain` (165 `.cs` files), `Dto`,
  `Providers` and the API host were **not** loaded, because they need the private
  `AcmePackageFeeds` feed which this session's egress policy denies. The mapping-rule verdicts
  above are sound for the shapes probed and should be re-confirmed over the full solution on a
  machine with feed access.
- The server is the bundled `Microsoft.CodeAnalysis.LanguageServer 5.11.0-1.26380.4` from the
  C# extension, not the `roslyn-language-server` global-tool wrapper the adapter's
  `default_command` names. Same server, same protocol surface, different packaging; the thin
  wrapper's `--stdio --autoLoadProjects` flags are accepted by the underlying server as shown.
- `csharp-ls` 0.26.0 (the PRD's named fallback) was also installed and probed. It advertises
  `definitionProvider: true` but **`typeHierarchyProvider` absent**, and it never completed an
  MSBuild workspace load for this `net10.0` solution within 7 minutes, so it produced no
  operation results. **The fallback is not equivalent for the subClassOf row.**

---

## What each probe means for G0

### Probe 1

Kotlin is **closer to clearing than G-1 could establish, with one half of one condition still
open.** Three of the four things that were unknown are now settled, and settled favourably:
Gradle is 8.14.3, well clear of the silent 8.8 floor; the project is conventional (no KMP,
one flavour dimension, six variants) and its variant model configures and enumerates cleanly;
and the JDK 25 requirement turns out to be a non-issue because the distribution bundles its own
runtime, so it never meets the project's JDK 17 target.

Two things changed the shape of the question rather than just answering it. First,
**typeHierarchy is advertised but dead** — which makes §5.4 condition (b) not merely
"preferable" but **mandatory**, and makes capability-flag gating an active hazard. Second,
**`hover` already returns the supertype clause**, so condition (b)'s route is cheaper than the
PRD assumed and needs no workspace import to work. Against that, the census finding cuts the
other way: ~75% of Kotlin's subClassOf edges sit in the UI and design-system layers, and across
the three `data/*/domain/model/` directories **10 data classes carry not one supertype between
them**. **The subClassOf row is worth implementing for completeness, but it is not what
determines whether Kotlin is a useful second leg** — the entity and typed-property rows are,
and those work today, per-file, without an import.

The genuinely open item is narrow: **can kotlin-lsp be made to import this Gradle/AGP
workspace?** Everything downstream of that — `workspaceSymbol`, cross-file `references`, the
"composition and reference" row — depends on it, and all three were empty in the un-imported
mode. Per-file extraction alone would yield class candidates and typed properties but no
cross-file relations.

### Probe 2

**V5 is closed, and closed better than the PRD's expectation.** The question §5.4 posed —
"probe whether the Roslyn server answers typeHierarchy" — is answered **yes**, in both
directions, on production declarations. The C# `rdfs:subClassOf` row needs no synthesis and
stays on the "high — declared" line. The remaining work is genuinely additive wiring, and one
of the two operations (`definition`) needs no new normalisation code at all.

The more interesting result is that **the extraction table survives contact with production
C#, with one row narrower than written**: `documentSymbol` carries property types inline
(better than expected), while the "foreign keys" half of the typed-property row finds
**nothing**, because this codebase declares no navigation properties and keeps relational
structure in Fluent-API configuration and migrations — outside the declaration surface. That is
a limit of the declaration-level subset, and it should be written down before G0 rather than
discovered as a shortfall in the first proposal graph.

---

## Consequence — both branches, not chosen

Emil rules. Stated so that whichever way the ruling goes, the consequence is already written.

### If Kotlin clears — G0 runs C# + Kotlin, n = 2

- **Two-root agreement is evidence, not the three-root assurance upgrade.** §5.3's first row
  makes the upgrade conditional on independent roots, and §5.3's own n=2 qualifier already says
  the upgrade's full form applies from the third root onward. With C# and Kotlin only,
  agreement is a **review signal**; it does not carry the Q11/Q27 triangulation upgrade, and
  proposal graphs should not be marked as though it does.
- **The intersection over a backend and one client is narrower than over a backend and two
  clients** — and this session can now say how much narrower in concrete terms. The naive
  exact-name intersection is **23 type names** against 942 C# and 1090 Kotlin declarations
  (~2%). With two clients, a backend-only element can be classified by whether *both* clients
  lack it; with one client, "backend-only" and "this client happens not to model it" are
  indistinguishable without the reviewer supplying the judgement. §5.3's third and fourth rows
  ("backend-only → server concern, or a divergence" / "client-only → UI state, or a
  divergence") both collapse toward "reviewer classifies", with less structural evidence to
  classify from.
- **The "should be identical" decision still has something to discharge against**, which is the
  main thing n=2 buys: one backend and one client either do carry the same entities and
  relations or they do not. This session already found concrete material for it to bite on —
  **`LocationId` is modelled as a single-string identity type on both sides**
  (`readonly record struct LocationId(string Value)` / `data class LocationId(val value: String)`),
  a real agreement; while **each side applies its identity convention inconsistently** (C#: one
  of four types declares `IStringIdentity`; Kotlin: four identity types under `data/location`
  but `InstallationId` left as a bare `typealias … = String`). That is the decision's falsifier
  finding usable evidence on the very first run.
- **Independence looks genuinely satisfiable here** (§5.3's "independence is a field, not an
  assumption"): the two models are written in different idioms — `by`-delegating data classes
  against `readonly record struct`s — with only ~2% exact-name overlap, which is not the
  signature of one model generated into two languages. That is a reviewer's declaration to
  make, not this session's, but the
  structural evidence does not suggest a correlated-failure trap.

### If Kotlin does not clear — G0 is single-source pipeline validation

- Everything on the pipeline side **still exercises**: the §5.2 mapping rules (and this session
  has shown five of six rows fire on production C#, with the FK half of one row empty), RDFS/OWL-RL
  entailment, the CONSTRUCT-to-fixpoint relation rules, proposal graphs as registry branches,
  per-triple review, and the founding ratification. The first registry instance is generated and
  ratified exactly as planned.
- **What is lost is the whole comparison instrument**: no divergence finding, no shared-domain
  intersection, no dissent rows, and — the sharpest loss — **the "they should be identical"
  decision has nothing to discharge against**. §11 currently calls G0 "where the first real
  finding lands — three codebases that should carry one domain, and the extractor saying
  whether they do". With one codebase, that sentence is false and the phasing note should say
  so rather than quietly under-deliver.
- **Proposed gate for the single-source branch** (proposal, not a ruling): replace the
  cross-codebase gate with a **re-run determinism and yield gate** — (a) Emil ratifies a first
  ontology from one codebase by merging, per-triple; (b) the extractor is re-run at a second,
  later ref of the same repo and the diff is reported as candidates-added / symbols-disappeared,
  which exercises §5.5's re-runnability and the Q21 decay mechanism without a second language;
  (c) **bootstrapping yield reported per assurance row** — the proportion of proposed triples
  accepted, which is the one §9 success criterion that does not need n≥2; and (d) the
  "should be identical" decision is **filed but explicitly left undischarged**, with its
  falsifier named and the reason recorded, so it is a known open rather than a silent gap.
  This keeps G0 honest about what it did and did not demonstrate, and it leaves the comparison
  finding as the first thing the second leg buys whenever it arrives.

---

## Proposed PRD edits — ruled in, and applied

*Written against `docs/g-track/prd-ground-as-ontology.md` at `d2bf37d` and proposed as text.
**Emil ruled these in on 2026-08-17 and they are applied in the same PR as this report**, with
two additions and one carry-along noted in the status block at the top. The text below is
retained as the rationale for each edit.*

### Edit A — §5.4, the Swift row: deferred, reason typed as an arrangement limit

Replace the Swift row's **Status** and **Extractor precondition** cells with:

> | Swift | SourceKit-LSP, bundled with Xcode and swift.org toolchains | **deferred at G0 — arrangement limit, not a tooling limit.** All six operations implemented, typeHierarchy included; SwiftPM projects background-index by default since Swift 6.1. The server is not the constraint and nothing about it is in doubt | **Deferred: no macOS machine with the client's Xcode is in the programme's arrangement at G0 entry.** An xcodeproj app needs the `xcode-build-server` BSP shim and a completing build to materialise the index (unit/record files + IndexStoreDB); there is no Linux path for iOS-SDK code. This is a property of *who has which machine*, not of the adapter set: it falsifies nothing about SourceKit-LSP, requires no code change, and lifts the moment a macOS runner with the client's Xcode enters the arrangement. Re-enters the corpus as the third root, at which point §5.3's independence-triangulation upgrade becomes available for the first time |

And add, after the table:

> **On typing this as an arrangement limit.** The three adapter rows fail for different reasons
> and the distinction is load-bearing. C# is *wired or not wired* — engineering in this repo.
> Kotlin is *implemented or not implemented upstream* — a tooling limit we do not control.
> Swift is *neither*: the instrument is complete and verified, and only the arrangement is
> missing. Recording it as a tooling limit would misattribute the gap to the adapter set and
> would make the eventual third root look like new capability rather than a machine becoming
> available.

### Edit B — §11, the G0 row: made conditional on this probe

Replace the G0 row's **Delivers** and **Gate** cells with:

> | G0 | First registry instance generated from the template (parameters supplied at generation; birth provenance in its first commit; founding decision filed by its ratifier); **Oxigraph 0.5 upgrade as task one**; local embedded store over a clone; Reading and Act types; PROV-O wiring; **code extractor over the corpus — C# first (probed and closed 2026-08-17), Kotlin second *conditional on the import question below*; Swift deferred as an arrangement limit (§5.4)**; proposal graphs as registry branches; per-triple review; **shared-domain intersection only if the Kotlin leg lands (n = 2 qualifier, §5.3) — otherwise G0 is single-source and the gate below changes** | **G0-entry checklist, as re-verified 2026-08-17:** generate the first instance (parameters supplied then); the Oxigraph 0.5 upgrade as task one; **Roslyn `typeHierarchy` probe — DISCHARGED** (answers both directions on `acme-app/backend` @ `83c9339a`; `definition` also confirmed; wiring only); **Android Gradle read — DISCHARGED** (`acme-app/android` @ `67340f69`: Gradle 8.14.3, AGP 8.13.0, six variants enumerate cleanly under Gradle); **kotlin-lsp workspace import — OPEN**, the single remaining entry condition, and the one that decides whether G0 is n = 2 or single-source; ~~the iOS build completing on Emil's machine~~ **struck as a gate — re-filed as a deferred leg (§5.4)**. Gate, **if the Kotlin leg lands**: Emil ratifies a first shared-domain ontology by merging; contradiction count reported; the "should be identical" decision filed with the extractor as its discharge. Gate, **if it does not**: the single-source substitute gate — first ontology ratified per-triple from one codebase; extractor re-run at a second ref with the diff reported; bootstrapping yield reported per assurance row; the "should be identical" decision filed but explicitly left undischarged, with its falsifier named |

### Two further edits this probe implies, offered for the same ruling

**Edit C — §5.2, the `subClassOf` row.** The row currently records kotlin-lsp's gap as
"typeHierarchy — where a server lacks it (kotlin-lsp, verified 2026-08-17)". That is now
imprecise in a way that could cost an implementer a day. Suggested replacement for the
parenthetical:

> *(re-verified live 2026-08-17 against kotlin-lsp v262.9593.0:* the server **advertises
> `typeHierarchyProvider: true` and then returns `null`** — the gap is a silent empty result,
> not a capability denial or an error. **Adapters must probe, never trust the capability flag**,
> or Kotlin will contribute zero `subClassOf` edges while reporting success. Synthesis for Kotlin
> is cheaper than assumed: `hover` returns the server-rendered declaration header including the
> supertype clause (`data object NoIcon : BadgeIcon()`), so no raw-text slicing is required and
> the route works without a workspace import. For **C#, this row needs no synthesis at all** —
> Roslyn 5.11.0 answers `prepareTypeHierarchy` / `supertypes` / `subtypes` correctly.*

**Edit D — §5.2, the typed-properties row.** Add the measured limit:

> *(measured on production C# 2026-08-17:* `documentSymbol` alone returns property **types and
> nullability** inline (`Id : ProfileId`, `DisabledReason : int?`), so `hover` refines
> rather than enables this row. **The "foreign keys" half found nothing**: the probed codebase
> declares no navigation properties and no FK attributes — relational structure lives in
> EF Fluent-API configuration and generated migrations, outside the declaration surface. Expect
> datatype properties with value-object ranges and **no object properties** from entity
> declarations of this shape.*

---

## Appendix — reproduction

Everything below ran on Linux x86-64 with no macOS and no Android Studio. Nothing was written
into either corpus repository.

| Component | Version / source |
|---|---|
| Roslyn LSP | `Microsoft.CodeAnalysis.LanguageServer` **5.11.0-1.26380.4**, extracted from `ms-dotnettools.csharp` **v2.148.23** (linux-x64), fetched from the marketplace `vspackage` endpoint |
| .NET SDK | **10.0.110** (apt `dotnet-sdk-10.0`); repo `global.json` asks for 10.0.0 |
| csharp-ls (fallback) | **0.26.0** via `dotnet tool install --global csharp-ls` |
| kotlin-lsp | **v262.9593.0** standalone Linux-x64, `download-cdn.jetbrains.com`, SHA-256 `2d99d8e1…7cc5e` verified against the published digest; bundles JBR 25.0.2 |
| Android SDK | cmdline-tools `11076708`, `platforms;android-36`, `build-tools;36.0.0`, `platform-tools` — exposed via `ANDROID_HOME`, **no `local.properties` written** |
| Gradle | **8.14.3** via the repo's own wrapper, run with `-g`/`--project-cache-dir` pointed at scratch |

**Egress notes** (this session's policy): `github.com`, `api.github.com`,
`pkgs.dev.azure.com` (both the Microsoft prerelease feed and the private `AcmePackageFeeds`)
and `ms-dotnettools.gallerycdn.vsassets.io` all return **403** and were not routed around.
`raw.githubusercontent.com`, `download-cdn.jetbrains.com`, `marketplace.visualstudio.com`,
`api.nuget.org`, `services.gradle.org`, `dl.google.com` and `repo.maven.apache.org` are
reachable. The third-party vendor Maven repository fails to connect — a connection failure, not a policy denial.

**Probe harnesses** (scratchpad, not committed): a Rust binary depending on `ddd-lsp` by path
that drives `ddd_lsp::host::Host` with `adapter::for_language("csharp")` — so V5 exercised the
**shipped** adapter host layer, not a bespoke client — plus two Python raw-JSON-RPC harnesses
for kotlin-lsp and for reading `InitializeResult` capabilities (which `Host::initialize`
discards).

**Solution scoping for V5:** `dotnet new sln --format sln` in the scratchpad, adding
`…Backend.Sql`, `…Options`, `…Primitives` by absolute path, restored from nuget.org. This keeps
the private-feed projects out of the graph without editing the Backend's own solution.
