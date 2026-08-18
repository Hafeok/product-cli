# kotlin-lsp workspace import — V3(a).4, the final G0-entry open item

**Session type:** verification. Reports, does not rule. **Date:** 2026-08-17.
**Ruling authority:** Emil. Nothing here changes the PRD; §11's two branches are
both stated and neither is chosen.

> **Naming.** The client repositories are `corpus-android` and `corpus-backend`
> throughout. Domain identifiers are described structurally. Paths inside the
> corpus are written `<corpus-android>/…`.

---

## 0. The answer, up front

**The import starts.** Under the handshake the reference client actually sends,
kotlin-lsp v262.9593.0 begins a real Gradle/AGP workspace import of
`corpus-android` within ~0.2 s of `initialized`, injects its own Gradle init
script, applies its own tooling plugin, configures the included build, resolves
the plugin classpath (AGP 8.13.0 among it) and proceeds to per-module,
per-variant task realisation.

It did **not complete** in this environment. It ran 238.8 s and failed at
dependency resolution for **one third-party vendor SDK artifact** whose Maven
repository this sandbox cannot reach (`CONNECT tunnel failed, response 502`).
That artifact exists on no other reachable repository. This is an **environment
limit, precisely located** — not an AGP-experimental limit, not a corpus fault,
and not the handshake.

So V3(a).4 splits cleanly into two findings that should not be merged:

| Sub-question | Status |
|---|---|
| *Can the import be made to start at all?* — the actual open item | **Discharged. It starts.** The two earlier handshakes were wrong in transport and in option schema; the reference client's handshake works first time |
| *Does the import run to completion on `corpus-android`?* | **Not demonstrated here.** Blocked by one unreachable vendor Maven repository. Blocker named exactly; not simulated |

A reduced-corpus control — the same corpus with the single module carrying that
vendor dependency removed — separates "the mechanism cannot work" from "this
sandbox cannot reach one host". **It imports successfully**, fetches an
`AndroidProject` model, and `workspace/symbol` goes from **empty to populated**
across the import boundary. Details in §4.

So the mechanism is demonstrated end to end; what is undemonstrated is one
network hop. That is the shape of the remaining uncertainty, and it is
deliberately not resolved into a recommendation here.

---

## 1. The reference client's handshake, as extracted

**Source.** `kotlin-server-0.0.6-linux-amd64.vsix`, downloaded 2026-08-17 from
`download-cdn.jetbrains.com/language-server/kotlin-server/262.9593.0/`,
**SHA-256 `90974cd8…687ec` verified against the published `.sha256`**. This
digest is new to the G-track record; the prior session verified only the
standalone archive (`2d99d8e1…7cc5e`, re-verified here and unchanged).

`Kotlin/kotlin-lsp` `RELEASES.md`, fetched 2026-08-17, still lists
**v262.9593.0** at the top — the pin is unchanged since 2026-07-27.

**Method — capture beat inference where it could.** The extension ships
`extension/out/dist/extension.js.map` **with `sourcesContent` populated**, so the
handshake was read off the extension's *original TypeScript*
(`src/lspClient.ts`, 396 lines) and its bundled **`vscode-languageclient`
9.0.1** (`lib/common/client.js`), not off minified output or documentation.
Two independent confirmations then came from the server side (§1.3, §3.1).

### 1.1 Transport — the first thing both earlier probes had wrong

The extension does **not** run the server over stdio. From
`src/lspClient.ts:163-210, 220-264, 266-291`:

```
spawn: <ext>/server/bin/intellij-server  --socket 0  [--system-path <storageUri>]
       ↳ read stdout until a line contains "Server is listening on …:<port>"
       ↳ TCP-connect 127.0.0.1:<port>; that socket is both reader and writer
```

`--socket 0` means "bind an ephemeral port and announce it". The launcher's
port announcement is a **stdout side-channel that exists only in socket mode**.
Timeouts the extension applies: 60 s for the announcement, 10 s to connect,
100 ms retry delay.

### 1.2 `initializationOptions` — the second thing they had wrong

From `src/lspClient.ts:319-342`:

```jsonc
{
  "defaultSdk": <intellij.jdkForSymbolResolution>,      // string path | null
  "buildTools": { "<workspaceFolderUri>": <intellij.buildTool> }
}
```

`buildTools` is a **map keyed by workspace-folder URI**, one entry per folder.
Per the extension's own `package.json` contribution, `intellij.buildTool` is
*"A build tool to use for a server, workspace, folder"* with **`null` = any**
(auto-detect) and `""` = none. Default is `null`, i.e. the shipped default
**does** import.

### 1.3 Confirmed against the server's own type

`javap -p` on `com.jetbrains.ls.snapshot.api.impl.core.InitializeOptions`
(extracted from `server/lib/product.jar`) gives the wire contract exactly:

```java
private final java.nio.file.Path defaultSdk;
private final java.util.Map<URI, java.lang.String> buildTools;
private final java.nio.file.Path indexDir;
private final boolean skipWorkspaceCreationForTests;
private final LibraryRootsMode libraryRootsModeForTests;
```

Five fields, three usable. This settles a loose end from the prior probe: the
options it invented — `{importWorkspace: true, gradle: {enabled: true}}` — are
not merely ineffective, they are **not in the schema and are silently
discarded**. That is why the earlier log's only occurrence of "gradle" was the
probe's own options echoed back.

The sibling class `InitializeKt` carries the diagnostics
`No build tools selected for `, `No applicable build tools found for ` and
`Unknown build tool '` — the three ways `buildTools` can decline to import.

### 1.4 The rest of the initialize envelope

From `vscode-languageclient` 9.0.1 `lib/common/client.js:791-830` plus
`workspaceFolder.js:32-41`:

- `processId: null` (**not** the client's real pid)
- `clientInfo: {name, version}`, `locale`, `trace`
- `rootPath` **and** `rootUri` both set to the first workspace folder
- `workspaceFolders: [{uri, name}]` filled by the workspace-folders feature
- **`progressOnInitialization: true`** (`lspClient.ts:323`) ⇒ the client mints a
  UUID, sends it as **`workDoneToken` inside `initialize`**, and attaches a
  progress part to it. This is how server-side startup progress is reported
  *during* `initialize`.
- `documentSelector` covers schemes `file`, `jar`, `jrt`, and adds language
  `java` alongside `kotlin` (`lspClient.ts:293-317`).

**No post-`initialize` notification triggers the import.** There is none in the
extension. The import is driven from inside the server's `initialize` handler —
confirmed by the stack trace in §3.1.

---

## 2. What was run, on what

| | |
|---|---|
| Corpus | `corpus-android` @ **`67340f69`** — unchanged, and **verified pristine (`git status --porcelain` empty) after every run** |
| Server | `intellij-server` **build 262.9593.0** (`product-info.json`), taken from the **vsix's own bundled server**, i.e. the exact binary the reference client launches; bundles JBR 25.0.2 |
| Route | **Minimal harness**, not the shipped host layer — see §5 for why the host layer cannot express this handshake today |
| Host | Linux, JDK 21 on host (irrelevant — the server uses its bundled JBR 25), Gradle 8.14.3 via the corpus's own wrapper, Android SDK platform 36 + build-tools 36.0.0 installed to a scratch dir, `ANDROID_HOME` exported, **no `local.properties` written into the corpus** |

The harness (`probe.py`) reproduces §1 faithfully: socket transport with port
announcement parsing, the reference `initializationOptions`, `workDoneToken` in
`initialize`, a full VS Code capability block, and — importantly — it **answers
server→client requests** (`workspace/configuration`, `workspace/workspaceFolders`,
`window/workDoneProgress/create`, `client/registerCapability`) rather than
leaving them hanging.

**Two false starts of my own, recorded so they are not mistaken for findings:**

1. The harness first stripped `JAVA_TOOL_OPTIONS`, which in this sandbox carries
   the proxy CA truststore. The import started and immediately died on
   `SSLHandshakeException … PKIX path building failed` fetching the Gradle
   distribution. **My fault, not kotlin-lsp's.** Restored, the distribution
   downloaded cleanly (137 MB).
2. The harness first passed a **relative** `--system-path` while running with
   `cwd` = corpus root, so the server created a scratch directory inside
   `corpus-android`. Caught, removed, path made absolute, and the corpus
   re-verified clean. No corpus file was ever modified.

---

## 3. What the import actually did

### 3.1 It starts — timeline, run 3, full corpus

| t | event |
|---|---|
| 1.5 s | port announced, socket connected |
| 1.5 s | `initialize` sent |
| 1.5–2.3 s | `$/progress` on **my `workDoneToken`**: "Initializing server" → "Initializing IntelliJ" → "Starting indexer" → "Opening project database" → "Loading workspace model from cache" → end |
| 2.3 s | `initialize` returns |
| 2.3 s | `initialized` sent |
| **2.33 s** | **`$/progress` begin, title "Importing project"** — server-minted token |
| 2.35 s | `"Importing folder <corpus-android>"` |
| 2.5–5.5 s | downloads `gradle-8.14.3-bin.zip` per the corpus's wrapper (137 MB) |
| ~13 s | compiles `/tmp/lsp-gradle-init*.gradle`; **applies `com.jetbrains.ls.imports.gradle.IdeaGradleLspPlugin`** |
| 25–31 s | compiles `settings.gradle.kts` (CLASSPATH, then BODY) |
| ~90 s | resolves AGP **8.13.0** and the plugin classpath |
| ~127 s | builds the corpus's included `build-logic` build |
| ~168 s | realises per-module, per-variant tasks across the module set |
| 182–240 s | resolves the projects' own dependency graphs |
| **241.1 s** | `"Run build failed"` → `window/logMessage` with the stack → `"Workspace is not imported"` |
| **241.2 s** | **`$/progress` end** on the import token — duration **238.8 s** |

The importer is `com.jetbrains.ls.imports.gradle.GradleWorkspaceImporter.importWorkspace`
called from `com.intellij.ls.server.requests.core.InitializeKt$importWorkspaceFromFolder`
(`initialize.kt:240`) — i.e. **import is part of `initialize`**, per folder, exactly
as §1.4 predicted from the client side.

### 3.2 Why it failed — the exact `Caused by` chain

```
LocationAwareException: Execution failed for task ':app:dataBindingMergeDependencyArtifactsDevelopDebug'
  → TypedResolveException: Could not resolve all files for configuration ':app:developDebugCompileClasspath'
    → ModuleVersionResolveException: Could not resolve <vendor-sdk-module>
      → ResourceException: Could not get resource '<vendor-maven-host>/…/<vendor-sdk>.pom'
        → HttpErrorStatusCodeException: Received status code 502 from server: Bad Gateway
```

Independently confirmed at the network layer: `curl` to that host returns
`CONNECT tunnel failed, response 502` — the sandbox's egress proxy will not
tunnel to it. The artifact is **absent from Maven Central, JitPack and Google's
Maven** (404 on all three), so no substitution is available. The same host was
already recorded unreachable in the prior session.

### 3.3 A finding that matters independently of the network

**kotlin-lsp's import executes Gradle tasks; it does not merely configure.** The
failure is in *task execution* (`:app:dataBindingMergeDependencyArtifacts…`)
resolving a **compile classpath**.

This retires an inference in the current PRD text. The prior session's control —
`./gradlew :app:tasks --all` enumerating all six variants with no failures — was
read as "the project is not the obstacle". That control **only configures**; it
never resolves a dependency graph, a point the prior report itself made. The
kotlin-lsp importer does resolve, so **the control was not equivalent to the
import** and could not have predicted this outcome either way. The PRD's §5.4
row should not continue to lean on it as evidence that import will succeed.

---

## 4. The reduced-corpus control

To separate *mechanism* from *reachability*, the import was re-run against a
**copy** of `corpus-android` with the single module carrying the unreachable
vendor dependency excluded from `settings.gradle.kts`. `corpus-android` itself
was untouched (copy made outside it; original verified pristine).

This is a **reduced corpus, not the corpus** — stated plainly so the result is
not over-read. It retains the design-system and data layers, which is where the
sealed hierarchies and the value-object identity types that G0 cares about
actually live.

### 4.1 The import completes

| t | event |
|---|---|
| 2.14 s | `$/progress` begin "Importing project" |
| ~124.5 s | `Fetch model 'com.jetbrains.ls.imports.gradle.model.AndroidProject' for project scope succeeded` |
| 124.7 s | `Run build succeeded` |
| 125.6 s | **`Successfully imported folder <path>`** |
| **127.0 s** | `$/progress` **end** — duration **124.8 s** |
| 126.9 → 189.6 s | a **second** phase: `Indexing`, **62.8 s**, `Processed 181 471 files` |
| **189.65 s** | notification **`intellij/ready-for-test`** |

Two things follow.

**The AGP path is exercised, not just plain Gradle.** The importer fetches its
own `…imports.gradle.model.AndroidProject` model. This is the sub-check the PRD
records as undemonstrated — "variants import *through kotlin-lsp*" — and on a
fully-resolvable corpus of this shape it holds.

**Success and failure are distinguished only by the last `report` message before
`end`.** `Successfully imported folder …` versus `Workspace is not imported`.
The `end` frame itself is identical, and **no error is returned to any request**.

### 4.2 The extraction surface, before and after

Same server, same file, same position — the only difference is whether the
import had finished.

| Operation | During import (t + 20 s) | After import + indexing (t + 600 s) |
|---|---|---|
| `workspace/symbol` | **0 results** | **returns the declaration** |
| `documentSymbol` | works | works |
| `hover` on a sealed subclass | `data object NoIcon : BadgeIcon()` | identical |
| `prepareTypeHierarchy` | **`null`** | **`null`** |

Three conclusions, each load-bearing:

1. **The import is what unlocks `workspace/symbol`.** Before: empty. After:
   populated. The prior session's `workspace/symbol("…") → array[0]` was a
   correct reading of a server that had never imported — not evidence about the
   corpus.
2. **The empty answer is silent.** No error, no flag, well-formed empty array.
   Identical in shape to the advertised-then-null `typeHierarchy` trap. This is
   the concrete justification for R8.
3. **The import does not unlock `typeHierarchy`.** `typeHierarchyProvider: true`
   is still advertised and `prepareTypeHierarchy` still returns `null` with a
   **fully imported and indexed workspace**. PRD §5.2's Kotlin row stands exactly
   as written, and the `hover`-based synthesis route remains the only route.

### 4.3 What the control does and does not license

It licenses: *the handshake is right; kotlin-lsp's Gradle/AGP importer works on
a corpus of this shape, at this Gradle/AGP/Kotlin version, on this host; and a
completed import demonstrably unlocks the workspace-scoped surface.*

It does **not** license: *`corpus-android` as it stands imports.* One module was
removed to get here. Whether the full corpus imports remains **untested**, and
on this evidence the only known obstacle to testing it is network reachability
of one private repository — not the tool.

---

## 5. What an adapter must do differently — requirements

Stated as requirements, not code. Every one is a gap against `ddd-lsp` as
merged at `e830c57`.

**R1 — Socket transport with port-announcement handshake.**
`ddd-lsp/src/client.rs` is stdio-only by construction ("A live language-server
child spoken to over stdio"; `LspClient::spawn` pipes stdin/stdout). A Kotlin
host needs: spawn with `--socket 0`, read the child's **stdout** for
`Server is listening on …:<port>`, then connect a TCP socket and speak LSP over
it. Transport therefore has to become a per-adapter property.

**R2 — stderr must stop being discarded.** `spawn` sets `stderr(Stdio::null())`.
The launcher reports startup faults there. Keep it, at least to a note buffer.

**R3 — Per-adapter `initializationOptions`.** `host.rs::initialize` sends no
`initializationOptions` at all — there is no field for them on `Adapter`. Kotlin
requires `{defaultSdk, buildTools: {<folderUri>: null}}`. The `buildTools` key
**must be the folder URI in the same spelling as `workspaceFolders[].uri`**.

**R4 — Send `workDoneToken` in `initialize`.** Otherwise server startup progress
is unaddressed. Cheap, and it is what the reference client does.

**R5 — `processId: null`.** The host layer currently sends
`std::process::id()`. The reference client sends `null`. Harmless-looking, but
some servers shut down when the announced pid dies; match the reference.

**R6 — Answer `workspace/configuration` with real values, and implement
`workspace/workspaceFolders`.** Today `workspace/configuration` is answered with
an array of `null`s and `workspace/workspaceFolders` is **not handled at all** —
it falls through to `-32601 unhandled`. A server that re-reads its build-tool
choice through configuration would get "none" from a null. This is a latent
correctness bug for any host that asks.

**R7 — Readiness must be a progress-token lifecycle, not a notification name.**
This is the substantive one. kotlin-lsp signals import completion by **ending
the `$/progress` token whose `begin.title` is `"Importing project"`**, and puts
the verdict in the **last `report` message before that `end`** — `"Workspace is
not imported"` on failure. There is no `workspace/projectInitializationComplete`
equivalent.

`ReadySignal` cannot express this today:
- `Notification("$/progress")` is satisfied by the *first* progress message,
  which arrives ~0.2 s in, while the import has 4 minutes to run.
- `NotificationWhere("$/progress", …)` reads `ServerState::last_params`, which
  keys **by method**, so every one of the hundreds of progress messages
  overwrites the previous. The single `end` frame would have to be caught in the
  exact polling window, and interleaved tokens (an "Indexing" token runs
  concurrently) would collide.

Worse, readiness here is **two-phase**. The import token ending is *not*
sufficient: a separate `Indexing` token then ran **62.8 s** over 181 471 files,
and `workspace/symbol` is only reliable after that. Immediately afterwards the
server emits **`intellij/ready-for-test`** — the one plain notification that
marks the whole sequence complete. Its name suggests a test affordance rather
than a supported readiness API, so it should be treated as corroboration, not as
the contract. Note also that after the main indexing token the server emits a
**storm of same-titled zero-duration `Indexing` tokens** (thousands), so "an
Indexing token is open" is useless as a not-ready test.

Requirement: track progress **per token** — record each `begin` title, retain
that token's most recent `report` message, latch on its `end`. Readiness =
**the import token ended with a success message _and_ the substantial indexing
token ended**. The retained message is the outcome, and a failed import must
surface as **failed**, never as ready and never as merely slow.

**R8 — Gate every workspace-scoped query on R7, and never treat empty as
absent.** Measured, twice, on two corpora: while the import is running,
`workspace/symbol` returns **`0` results** with no error, while `documentSymbol`
and `hover` on an open file answer correctly. An extractor that queries early
gets a **silent, well-formed empty answer** — the same failure shape as the
advertised-then-null `typeHierarchy` trap already named in PRD §5.2. Absence of
symbols must be distinguishable from absence of an import.

**R9 — Budget a real timeout.** `REQUEST_TIMEOUT` is 15 s and `Host::request`
gives up at 30 s. Measured here:

| Run | Import | Indexing | Ready at |
|---|---|---|---|
| Full corpus, cold | 238.8 s → **failed** | — | never |
| Reduced corpus, cold | 124.8 s | 62.8 s | **~190 s** |
| Reduced corpus, warm caches | **19.1 s** | (cached) | **~37 s** (`intellij/ready-for-test`) |

Readiness is a **minutes-scale** wait on first run and a tens-of-seconds wait
warm. The adapter must not convert either into a per-request timeout, and should
not assume the warm number.

**R10 — Environment preconditions the adapter cannot fix.** An Android import
needs a reachable Gradle distribution, a JDK the Tooling API accepts, an Android
SDK (`ANDROID_HOME`), and **network reachability for every repository the build
declares** — including private/vendor ones. Failure of the last is indisputably
fatal and, on this evidence, arrives only after minutes of apparently healthy
progress. The adapter should report the retained progress message verbatim
rather than a generic timeout.

**R11 — Keep the capability discipline (PRD §5.2).** Re-confirmed live at build
262.9593.0, now also *with an import under way*: `typeHierarchyProvider: true`
is advertised and `textDocument/prepareTypeHierarchy` returns **`null`** —
before, during, and after import. The workspace import does **not** unlock it.
`hover` returns the server-rendered declaration header including the supertype
clause, so the synthesis route stands and, as the prior session established,
needs no import at all.

---

## 6. Both branches, without choosing

Emil rules. Stated as the PRD's §11 conditional requires.

### 6.1 If Kotlin joins — G0 runs two legs, n = 2

- Two roots, C# + Kotlin, with the shared-domain intersection and the n = 2
  qualifier of §5.3 in play; the "should be identical" decision gets the
  extractor as its discharge, and Emil ratifies a first shared-domain ontology
  by merging, with contradiction count reported.
- `subClassOf` for Kotlin is **synthesised from `hover`'s declaration header**,
  never from `typeHierarchy` — which stays advertised-and-null. This route is
  confirmed working here both mid-import and independent of import.
- The prior session's two cautions stand unchanged and are the real risk to the
  leg's *value*: ~75 % of Kotlin's `subClassOf` edges sit in UI/design-system
  layers, and exact-name intersection with the C# side is ~2 %.
- **Cost this session adds:** the Kotlin leg is not "wire up an adapter". It is
  R1–R9 — a second transport, per-adapter init options, and a genuinely new
  readiness mechanism — plus R10, an operational precondition that no code
  change can satisfy.

### 6.2 If Kotlin does not clear — G0 is single-source

- The §11 substitute gate applies as written: first ontology ratified per-triple
  from one codebase; extractor re-run at a second, later ref with the diff
  reported as candidates-added / symbols-disappeared; bootstrapping yield
  reported per assurance row; and the "should be identical" decision **filed but
  explicitly left undischarged**, with its falsifier named.
- §5.3 does not run at all.

### 6.3 What this session changes about the choice

The open item was *"can the import be made to start"*. **It can, and the recipe
is now known and cheap** (§1). What is *not* established is that it **completes
on `corpus-android`**, and the obstacle to establishing that is a private
repository this environment cannot reach — an **arrangement limit of the same
family as the deferred Swift leg (§5.4)**, not a tooling limit.

That distinction is the session's main contribution to the ruling, and it is
deliberately left as a distinction rather than a recommendation. The one thing
the evidence does rule out is treating V3(a).4 as "kotlin-lsp cannot import
Android": that reading is now false.

---

## 7. The recipe, in one place

For whoever picks up the adapter work. This is the whole difference between an
import that starts and one that never does.

```
1. spawn:  <server>/bin/intellij-server --socket 0 --system-path <scratch dir>
             (absolute path — a relative one lands in the repo being imported)
2. read the child's STDOUT until:  "Server is listening on …:<port>"
3. TCP-connect 127.0.0.1:<port>; speak LSP over that socket
4. initialize:
     processId: null
     rootUri / rootPath: <repo>
     workspaceFolders: [{uri: <repo-uri>, name: …}]
     workDoneToken: <uuid>
     initializationOptions: {
       defaultSdk: null,                       // or a JDK path
       buildTools: { "<repo-uri>": null }      // null = auto-detect; "" = none
     }
5. initialized          ← the import begins here, unprompted
6. answer server→client requests: workspace/configuration,
   workspace/workspaceFolders, window/workDoneProgress/create,
   client/registerCapability
7. wait for: $/progress token titled "Importing project" to END,
   whose last `report` reads "Successfully imported folder …"
   (failure reads "Workspace is not imported" — same `end` frame, no error)
8. then wait for the substantial "Indexing" token to END
   (corroborated by the `intellij/ready-for-test` notification)
9. only now are workspace-scoped queries meaningful
```

Environment, non-negotiable: reachable Gradle distribution, `ANDROID_HOME`, a
JVM trust store that works for every host the build fetches from, **and network
reachability for every declared repository**.

---

## 8. Provenance

| Artefact | Pin |
|---|---|
| `corpus-android` | `67340f69` — read-only; verified pristine after every run |
| kotlin-lsp / VS Code extension | v262.9593.0 / v0.0.6, `RELEASES.md` fetched 2026-08-17, top entry unchanged |
| `kotlin-server-0.0.6-linux-amd64.vsix` | SHA-256 `90974cd8…687ec`, verified against published digest |
| `kotlin-server-262.9593.0.tar.gz` | SHA-256 `2d99d8e1…7cc5e`, verified; matches the prior session |
| server build | `262.9593.0` per bundled `product-info.json` |
| `vscode-languageclient` | 9.0.1 (extension `package.json`) |
| `product-cli` | `e830c57` (`main`, PRs #43/#44/#46/#47 merged) |
| Gradle / AGP / Kotlin in corpus | 8.14.3 / 8.13.0 / 2.2.10 |

**Not done, and out of scope:** adapter implementation; any change to
`corpus-android`; the C# full-solution re-confirmation; PRD edits; registry
generation; the extractor.
