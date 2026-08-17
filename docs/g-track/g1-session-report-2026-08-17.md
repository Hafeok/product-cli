# G-1 session report — reconcile, verify, initialise (2026-08-17)

**Session:** G-1, per the session prompt of 2026-08-17. Interactive; Emil rules at every gate.
**Branch:** `claude/g1-reconcile-verify-initialise-g0uno2` (all three repositories).
**This file grows gate by gate.** Step 1 (canon reconciliation) is below; Steps 2–4 append after
their gates open.

## Sources pinned (the staleness discipline, applied to this session's own inputs)

Every artefact this report projects is named here with its ref. A row without a ref would fail the
projection-as-source diagnostic the session prompt applies to its own outputs.

| Source | Ref / identity | Role |
|---|---|---|
| `actor-indexed-determination` (upstream canon) | `110bf10ff8fc0de0d71440310c869df78e34d8ef` — v5.5.0 (merge `e8663b8`, tagged 2026-08-16) plus two post-tag merges (PR #11 release CI, PR #12 session reconciliation); no canon content moved after the tag | reconciliation target |
| `decision-driven-design` (downstream canon) | `4848b9e15ea49bc923b2d23933e2c05a21202ba0` — merge of PR #21; `graph/upstream.yaml` pinned at v5.5.0 (DDD-dec-18) | reconciliation target |
| `product-cli` | `d506ac94310bb24e3c6a1b786034046ac0d024b0` (branch point) | the G-track home |
| PRD `prd-ground-as-ontology.md` | Emil's upload, 589 lines, written 2026-08-14 against holding-note revisions 13–14 | the document reconciled |
| Ground-axes holding note | Emil's upload, revision 18 (2026-08-15), 2,093 lines | **context only — canon outranks it everywhere they touch** |
| `meta/corpus-test-results-2026-08-14.md` | downstream, at `4848b9e` | evidence, fetched not uploaded |
| `meta/vocabulary-delivery-session-2026-08-15.md` | downstream, at `4848b9e` | evidence, fetched not uploaded |
| `meta/session-reconciliation-2026-08-16.md` | upstream, at `110bf10` | evidence, fetched not uploaded |

Queue position: satisfied. The vocabulary-and-delivery scope merged as v5.5.0 (upstream PR #10,
downstream PR #21); this session reconciles against that canon, not against the holding note.

---

# Step 1 — canon reconciliation

## 1.1 What v5.5.0 filed, and the deltas against what the PRD assumed

The PRD was pinned to holding-note revisions 13–14. The vocabulary-and-delivery session filed from
holding-note revision 8 plus the applicability note of 2026-08-12, with every filing scoped to what
the corpus test of 2026-08-14 evidenced. The deltas that matter to the PRD:

| Canon object | What it says | Delta against the PRD's assumption |
|---|---|---|
| `DDD-ground-01` (upstream, normative, projected) | A governing decision must declare a **resolvable applicability predicate**, or explicit universal applicability; non-evaluation never silently becomes non-applicability; implemented axes are marked mechanically-evaluable or judgement-evaluable | The PRD assumed Q1's named-axis gate. Canon amended it: the gate is predicate-general, and factored axes are one implementation, not the ontology of every region. The corpus put beyond-region predicate cases at ~27–36% |
| `DDD-ground-01` evidence (the matched pair) | The axis-type mark is a **maturity state**, not a fixed type — an axis moved nameable→resolvable in ~2.5 weeks | This answers open ruling 12 in canon's favour, which the PRD's §9.1 treated as open. Erratum noted in canon: the corpus document's "five weeks" was wrong; the repo dates govern |
| `DDD-ground-02` (upstream, conceptual, projected) | Source coverage (covered · declared-empty · undeclared · unknown), resolution (resolved · deliberately-open · unknown), and assurance (adequate · inadequate · unknown) are **orthogonal**; Unknown is never a pass | The PRD's §6.2 used Q3's four-state typing as UI. Canon rules the orthogonal typing governing; the four states survive only as the corpus's recorded projection, with "inert" replaced by **declared-empty** (ruled in, zero corpus draws recorded as its evidence status; the filing of a declared-empty is a claim-layer act) |
| `DDD-ground-03` (upstream, conceptual, projected) | Timing carries a fourth value, **"—(open)"**, for decisions whose resolution is deliberately-open | New since the PRD; touches the status line and any timing display |
| `DDD-ground-04` (upstream, normative, projected) | A retro-filed decision carries **two fields** — when the gap was uncovered, and that it was retro-filed | Ruled in, as the PRD hoped. Replaces holding-note §13.4 as the authority for §5.3's retro-filed decision set. Both sub-rulings recorded: retro-filing is the ledger-side discharge mechanism for the escape generators, and the retro-filing act is a claim-layer act |
| `DDD-delivery-01…03` (upstream) + `DDD-delivery-04` (downstream) + `core/13-delivery.md` + `term:delivery` / `term:undelivered` / `term:presumed-discharge` | Filing is not encoding; undelivered governance is **escape that presents as governance** (a generator, answering Q18's open question); unretrieved decision + unretrieved check are correlated failures; maturation's paid-once property holds only where the channel delivers per act-site | The PRD's delivery references (Q15/Q16/Q18/Q19) re-pin to canon terms and claims. The delivered-vs-emitted comparison the PRD names as its primary evidence output now has its stake in canon (`DDD-delivery-02`) |
| Term-collision repair | `00-primitives`' closing aside renamed "A note on delivery" → "A note on **presentation**" — the registry owns "delivery" | The PRD's uses of "delivery" were audited: every instance is in canon's sense (how authored governance reaches an act). No edit required. Spelling: canonical texts take "judgment"; claim and projection prose keeps British "judgement" — the PRD conforms |
| `graph/axis-registry.yaml` (downstream) | axis-registry/v1; **artefact-not-canon**; 22 axes (19 resolvable, 1 resolvable-partial, 2 nameable); promotion path: a validator reads it plus a ratification act | The first concrete axis registry exists — the framework program's own instance, not the G-track registry. It is the format precedent G0's proposal graphs inherit: quality marks and extractor sketches per axis |

One finding the PRD did not anticipate, in the other direction:

**The provenance typing is not in canon.** The PRD's §4.1 calls the four-value set (controlled /
observed / inferred / institutional) "canon's provenance typing", and §10 listed it among ratified
canon the PRD stands on without qualification. A search of both repositories at the pinned commits
finds no such typing — not in the core documents, not in `core/graph/terms.yaml`, not in any claim.
The holding note itself presupposes it rather than defining it (Q12 "the existing provenance typing
already supplies the scaling"; Q27 says canon "has carried this slot empty since the foundation
document"). The mechanism in the PRD stands; its basis is unratified. Marked in the PRD as
UNVERIFIED — Emil review, with two resolution paths: file the typing upstream before G0 consumes
it, or let the G-track SHACL shapes own the value set as track vocabulary with no canon claim
behind it. This is a mis-pin, not a design collision.

## 1.2 The §10 walk, row by row

Statuses read against upstream `110bf10` and downstream `4848b9e`. The PRD's fallback column
applies only on rejection; nothing was rejected, so no fallback fires.

| §10 construct | Status | Canon citation / absence | PRD sections touched |
|---|---|---|---|
| Position / region retrieval (Q2, Q19) | **ratified amended** (gate half); not filed (retrieval mechanism) | `DDD-ground-01` — predicate-or-explicit-universal, axis marking; region states axes are one implementation. No claim files the evaluator or subsumption retrieval itself | §7.2 (edits 2, 7), §7.3 |
| Extraction verification (Q20) | **not filed** | No claim; `DDD-ground-01`'s marking clause anchors the closure commitment; open ruling 13 open | none — dependency stands as flagged |
| Reading three-tuple (Q11 amended) | **not filed** | Assurance appears only as `DDD-ground-02`'s orthogonal property of ground; the per-reading tuple is note-only | §4.1 (provenance note, edit 11) |
| Trust decision backing (Q27) | **not filed** | Open ruling 27 open | none |
| Emitted proxy (ruling 22) | **not filed** as obligation; stake now canon | Open ruling 22 open; `DDD-delivery-02` + `term:presumed-discharge` file the comparison's stake | §4.2 (edit 8) |
| Registry = existing ontology (ruling 25, A1) | **not filed** | Open ruling 25 explicitly open — "the fork … the retrieval PRD waits behind" | none — A1 stands as assumption |
| Triangulation-with-independence (Q27 amended) | **not filed** | No filing | none |
| Decay-of-relevance under a pin (Q21) | **not filed** | Open rulings 14, 15 open | none |

## 1.3 Named holding-note items outside §10

| Item | PRD section | Status | Canon citation / absence | Edit |
|---|---|---|---|---|
| Q1 gate at proposal time | §4.3 | **ratified amended** | `DDD-ground-01` — named-axis became resolvable-predicate-or-explicit-universal | edit 2 |
| Q3 four-state typing | §6.2, §7.6 | **ratified amended** | `DDD-ground-02` — orthogonal typing governs; four states demoted to recorded projection; declared-empty ruled in; `DDD-ground-03` adds "—(open)" | edits 3, 4 |
| Q6 declaration discipline | §6.1 declare row | **not filed** | Precedence discipline is note-only; `DDD-ground-04`'s before/after evidential-status reasoning is the adjacent canon | none |
| Q11 three-tuple | §4.1 | **not filed** | as §10 row above | edit 11 (provenance only) |
| Q21/Q12 remedies | §6.1 declare row, §5.5 | **not filed**; the note's Q12/Q18 tension is resolved | `DDD-delivery-02` rules undelivered a generator of escape; ground-not-as-expected stays outside escape — both rulings now compatible, delivery is what distinguishes them | none |
| Q23 contract | §4.2 | **not filed**; one field anchored | `unevaluated_axes` is `DDD-ground-01`'s non-evaluation clause mechanised; the exposure profile stays note-only | edit 7 |
| Q26 ontology reading | §3, substrate | **not filed** | The design substrate remains unratified; axis-registry/v1 is the first concrete instance (artefact) | edit 9 |
| Q27 trust decisions | §4.4 | **not filed** | as §10 row above | none |
| Q30 authority/projection, "ground registry" | §3, §5.3 | **not filed** as canon term | Open ruling 32 is a freight-list item; Emil's rulings embedded in the PRD header are settled and unaffected | none |

## 1.4 The re-pin table — edits applied to the PRD copy

The edited PRD is `docs/g-track/prd-ground-as-ontology.md` on this branch. Every edit carries the
marker *(re-pinned at G-1)* in place. COLLISION count: **0**. One basis error (provenance typing)
is marked UNVERIFIED — Emil review rather than COLLISION, because it is a mis-pin, not a design
conflict between canon and the PRD.

| # | PRD section | Construct | Canon status | Edit |
|---|---|---|---|---|
| 1 | Provenance header | pinning | — | re-pin note added: commits named, holding note demoted to context |
| 2 | §4.3 proposed-decisions row | Q1 gate | ratified amended | gate restated as `DDD-ground-01`'s predicate-or-explicit-universal with axis marking |
| 3 | §6.2 status line | Q3 typing | ratified amended | orthogonal typing governing; four-state kept as canon's recorded projection; inert→declared-empty, uncovered-undeclared→undeclared; "—(open)" added (`DDD-ground-03`) |
| 4 | §7.6 halt row 2 | Q3 typing | ratified amended | trigger restated as source coverage = `undeclared`; escalation options renamed to canon values |
| 5 | §5.3 identity decision | retro-filing §13.4 | ratified as `DDD-ground-04` | authority re-pinned from the note to the claim; two fields and both Gate 4 sub-rulings cited |
| 6 | §9.1 evidence criterion | ruling 12 | answered in canon | criterion reframed from type-vs-maturity to transition rate; matched-pair evidence cited |
| 7 | §4.2 `unevaluated_axes`; §7.2 step 3 | Q23 field; Q2 regions | anchored / partially filed | `DDD-ground-01` citations added; predicate-generality of regions noted |
| 8 | §4.2 comparison; §7.2 close | delivery vocabulary | filed (draft) | stake cited to `DDD-delivery-02` / `term:presumed-discharge`; "holding note's Q19 sense" → `term:delivery`, `core/13-delivery.md` |
| 9 | §5.2 axis-registries row | axis registry format | artefact exists | axis-registry/v1 named as format precedent; quality-mark vocabulary adopted; scope distinction stated (framework's own instance, not this registry) |
| 10 | §10 table and closing line | all eight rows | as walked above | canon-status column added; provenance typing removed from the stands-without-qualification list; v5.5.0 filings added as named canon the PRD now stands on |
| 11 | §4.1 provenance row | provenance typing | **not in canon** | row re-marked as holding-note vocabulary; UNVERIFIED — Emil review note added with the two resolution paths |

## 1.5 What was checked and not edited

- The PRD's uses of "delivery" — all in canon's term sense; the term-collision repair requires
  nothing here.
- Emil's rulings embedded in the PRD header — settled; untouched.
- The PRD's remaining "verify at G-1" annotations (ratatui, GitHub Models, LogMap-class tooling,
  reasoner mechanics) — Step 2's work, not Step 1's.
- The fallback column of §10 — never applied; no construct was rejected.

**GATE 1 — closed** (Emil, 2026-08-17). Commit `077790d` stands; both structural amendments
accepted as canon's improvement on the draft. Rulings recorded:

1. **Provenance typing: the track owns it, pending the wave.** The value set does not file
   upstream from this session; the owning session is the queued Q25/Q27/Q30 filing wave (Q27's
   trust-decision mechanism is the backing institutional provenance needs). The G-track SHACL
   shapes own the value set as track vocabulary; filed as track decision `g-dec-01`
   (`docs/g-track/decisions/g-dec-01-reading-vocabulary.yaml`) with a `revisit_if` pinned to the
   wave. The finding is forwarded to the wave as evidence — its Q27 filing now has a named
   consumer waiting on it. The same disposition covers the Reading tuple's other unfiled halves
   (Q11 three-tuple, assurance-on-reading); one decision file carries all of it. The PRD's §4.1
   is marked down from "canon's provenance typing" to "track vocabulary, candidate for the Q27
   wave" — the mis-pin corrected in the artifact, not just the report.
2. **Status line: projection for the human, orthogonal for the machine.** §6.2 keeps the
   four-state projection (a status line is a projection for a human arrangement mid-act; compact
   and absorbable beats complete); halt logic and the act log run on the orthogonal values; a
   future detail view displays the orthogonal triple raw. Recorded in the PRD at §6.2.

---

# Step 2 — verification worklist (V1–V9)

**Method.** Every row verified against live primary sources on 2026-08-17 (release feeds, official
docs, source code at named commits), per the session prompt's discipline: asserted-from-knowledge
claims in the PRD treated as unverified regardless of confidence. Web verification ran through four
parallel research passes; local verification ran against the product-cli workspace at `d506ac9`.
Consequence vocabulary per the prompt: none / edit / open-item answered / new risk. **PRD edits are
marked, not applied** — Step 1 applied its edits on the prompt's instruction; Step 2's edits hold
for the Gate 2 ruling, and scope cuts are proposed, never applied.

**Access limit, stated first.** The corpus repos are visible to the account
(`CleverAS-App/Android`, `CleverAS-App/Backend`, `clever-dk/Clever-iOS`) but this session cannot
attach them: cross-tier `add_repo` is refused (session sources are Hafeok-tier). Per the prompt, no
public repo was substituted. V3, V4 and V5 therefore split into a verified public/local half and a
**blocked repo-dependent half**; the blocked checks are enumerated at the end of this section with
the ask.

## 2.1 Summary

| # | Item | Verdict | Consequence for the PRD |
|---|---|---|---|
| V1 | Oxigraph | **verified — holds**, one new risk | open item 8 answered (embedded half); new risk: workspace pin 0.4 is a superseded line; CONSTRUCT-fixpoint performance undocumented |
| V2 | ratatui | **verified — holds** | open item 12 (first half) answered; edit: drop the "asserted from knowledge" flag |
| V3 | kotlin-lsp | **public half verified — the trouble row, as expected**; repo half blocked | new risk: typeHierarchy not implemented; AGP import experimental, silent below Gradle 8.8; fwcd fallback is end-of-life; scope cut proposed below |
| V4 | SourceKit-LSP | **capability half verified — holds**; build/index half blocked | edit: iOS extraction requires a macOS runner with the client's Xcode and a completing build; no Linux path (confirmed) |
| V5 | C# adapter | **local half verified — two wiring gaps**; live-run half blocked | edit: `definition` and `typeHierarchy` are not wired (additive work); Roslyn-server typeHierarchy support unverified |
| V6 | GitHub Models / Copilot API | **verified — closes negative** | open item 9 answered: GitHub Models retired 2026-07-30; **Scaleway-only**, per the PRD's own fallback |
| V7 | Scaleway structured outputs | **verified — holds** | open item 10 ready: three rungs with structured outputs + tool calling confirmed; Emil picks |
| V8 | Ontology matching | **verified** | open item 11 (matching half) answered with recommendation: reimplement the small lexical+structural core in Rust; LogMap as offline calibration oracle |
| V9 | OWL-RL at projection build | **verified — options established** | open item 11 (reasoner half): three credible options ranked; ruling stays Emil's; architecture indifferent, as the PRD stated |

## 2.2 Rows in detail

### V1 — Oxigraph (verified 2026-08-17)

**Checked:** crates.io release history, repo activity, README conformance statement, Store API
docs, published benchmarks, issue tracker. **Found:** healthy and active — latest 0.5.9
(2026-06-18), eight releases in twelve months, ~monthly cadence; effectively one maintainer (Tpt)
with commercial sponsors — bus-factor ~1 is the honest risk signal. SPARQL 1.1 Query, Update and
Federated Query implemented, "nearly fully conformant" per README, preliminary SPARQL/RDF 1.2
draft support; no documented conformance gap list. Full quad/named-graph support (`insert_named_graph`,
GRAPH clauses, dataset management) — the PRD's named-graph-per-source design is covered.
Embeddability is not a hypothesis: `product-core/Cargo.toml` already carries
`oxigraph = "0.4"` driving the pf rule engine. **New risk:** the 0.4 line is superseded — last
0.4.11 (2025-05-21), 0.5.0 shipped 2025-09-13 as a breaking line; staying on 0.4 means no fixes
since May 2025. Upgrade to 0.5.x is a G0 task (API migration; 0.5.9 added thread-safe
Transaction/BulkLoader). **CONSTRUCT performance:** no CONSTRUCT-specific or rule-fixpoint
benchmark exists anywhere in Oxigraph's published material; the README's standing caveat is
"SPARQL query evaluation has not been optimized yet"; BSBM (~35M triples, SELECT-heavy, run on
0.2/0.3) is the closest proxy; known blank-node CONSTRUCT correctness issue (#220). At registry
scale (thousands to low millions of triples) this is a watch-item, not a blocker — V9's options
hedge it. **Consequence:** open item 8 answered for the embedded store; edit marked (upgrade note,
§3 serving row); new risk logged (0.4 pin; fixpoint perf undocumented).

### V2 — ratatui (verified 2026-08-17)

**Checked:** crates.io, repo governance, licence, widget ecosystem. **Found:** the live successor
of tui-rs (forked 2023, org-governed, ~22.3k stars); latest 0.30.2 (2026-06-19); licence **MIT**.
The §6.1 pipeline's needs map to shipped pieces: prompt window — `ratatui-textarea` v0.9.2
(2026-06-12), adopted into the ratatui org, or `tui-input` 0.15.4 (2026-08-10) for single-line;
panels, status line — core `Layout`/`Block`/`Paragraph`. Production users include gitui, atuin,
yazi, bottom. **Consequence:** open item 12's framework half answered; edit marked — §3 TUI row
drops "asserted from knowledge; verify at G-1".

### V3 — JetBrains kotlin-lsp (public half verified 2026-08-17; repo half blocked)

**Checked:** the actual repo (`Kotlin/kotlin-lsp`, read-only mirror), RELEASES.md, source at HEAD
`bd8bca2` (2026-08-16) including the Gradle importer and the registered LSP providers,
kotlinlang.org docs, fwcd fallback repo and issues. **Found:**

- Still **Alpha** (README badge and warning at HEAD); latest release v262.9593.0 (2026-07-27);
  weekly pre-alpha builds; JDK 25 required (platform builds bundle a runtime); headless launch
  first-class (`bin/intellij-server`; `kotlin-lsp.sh` deprecated). Apache-2.0 binary, still
  partially closed-source (IntelliJ/Fleet proprietary parts).
- **AGP import is now native but explicitly experimental** (moved from community workaround to
  in-tree importer ~April 2026); no compatibility matrix exists. Source-level evidence: Android
  variant collection **silently returns when Gradle < 8.8** — a client repo below that imports
  with no Android variants and no error; in-repo Android fixtures use AGP 9.1.0.
- **Operations: five of six.** definition, hover, references, documentSymbol, workspaceSymbol all
  registered. **typeHierarchy is not implemented** — API scaffolding exists, no provider is
  registered (inferred from source and feature-list absence; flagged as inference). Call
  hierarchy exists; type hierarchy does not.
- **Fallback assessment: fwcd/kotlin-language-server is end-of-life.** Self-declared deprecated
  in favour of kotlin-lsp; last release 1.3.13 (2025-01-18); no typeHierarchy either; known-broken
  Android classpath resolution (generated R classes, AGP variant blindness). Not a viable Android
  fallback; plain-JVM repos only.

**What breaks in the PRD's phasing.** §5.2's `rdfs:subClassOf` row names typeHierarchy as its LSP
operation; for Kotlin that operation does not exist, so the highest-assurance hierarchy row cannot
be extracted as specified. §5.4's "AGP support is the flag for the Android repo specifically"
is still the right flag and still cannot be cleared from public information: whether the Android
repo's Gradle/AGP versions clear the ≥ 8.8 floor is the **blocked repo-dependent check**.

**Narrowest scope cut, proposed not applied:** G0 runs **C# + Swift**; Kotlin joins when two
conditions clear: (a) the Android repo's Gradle version is ≥ 8.8 and kotlin-lsp demonstrably
imports its variants (one `bin/intellij-server` run against the repo answers it), and (b) the
Kotlin `subClassOf` extraction path is settled — either synthesised from
implementation/typeDefinition/references plus declaration-text (the C# adapter already slices
declaration text, so the pattern exists in-house), or deferred until kotlin-lsp ships
typeHierarchy. Cross-codebase comparison (§5.3) degrades gracefully: agreement analysis over two
codebases still surfaces contradictions, minus the third leg's triangulation. The cut is Emil's.

### V4 — SourceKit-LSP (capability half verified 2026-08-17; build/index half blocked)

**Checked:** repo at HEAD `7b387eb` (2026-08-13) — server capabilities in
`SourceKitLSPServer.swift`, background-indexing docs, BSP docs; swift.org and Xcode release notes;
xcode-build-server. **Found:** **all six operations implemented, typeHierarchy included**
(prepare/supertypes/subtypes). Ships with Swift toolchains (current stable Swift 6.3, Xcode
26.4). The index story splits by project type: SwiftPM projects get **background indexing on by
default since Swift 6.1** — no user build required; xcodeproj/xcworkspace apps (the realistic
shape of a client iOS repo) are **not natively supported** — they need the BSP shim
(`xcode-build-server`, macOS-only) and **a completing Xcode/xcodebuild build is a hard
precondition**: references, workspaceSymbol and typeHierarchy read IndexStoreDB over the unit and
record files the build writes, and go stale until rebuilt. "The index materialises" concretely
means: index-store unit/record files exist and IndexStoreDB serves occurrences over them. **No
Linux path for iOS-SDK code — confirmed** (by absence, flagged as such: the iOS SDK exists only
inside Xcode; xcode-build-server is macOS-only). **Consequence — edit marked:** §5.4's Swift row
precondition sharpens from "must build first" to "must build first, on a macOS runner with the
client's Xcode version, through xcode-build-server for an xcodeproj app; the extraction pipeline
cannot run iOS extraction in a Linux container". The **blocked repo-dependent check**: the actual
iOS repo's project shape (SwiftPM vs xcodeproj, Xcode version) and a real build+index run — both
need the repo and a macOS machine.

### V5 — the M-track C# adapter (local half verified 2026-08-17 at `d506ac9`; live-run half blocked)

**Checked:** `ddd-lsp/src/adapter/csharp.rs`, `ddd-lsp/src/host.rs`, every real (non-mock)
`request(` site in the ddd stack, `docs/ddd-v1-spec.md`. **Found:** the adapter drives
`roslyn-language-server` (official prerelease .NET global tool, pinned expectation 5.11.0,
readiness on `workspace/projectInitializationComplete`, `csharp-ls` documented as fallback;
host fragility is a *standing risk* in the spec's own words). Of the declaration-level six, the
stack **issues four**: documentSymbol (revdiff, intercept, MCP lang-tools), workspace/symbol,
references, hover (plus signatureHelp/rename/diagnostic beyond the set). **Gaps: `definition` is
never issued anywhere, and typeHierarchy is neither issued nor capability-declared.** Both ride
the generic `host.request(method, params)` layer, so the wiring is additive, not structural.
Unverified: whether `roslyn-language-server` answers typeHierarchy at all — needs a live probe.
**Consequence — edit marked:** §5.4's C# row "extractor precondition: none" becomes "wiring:
definition + typeHierarchy requests to be added; Roslyn typeHierarchy support to be probed".
The **blocked repo-dependent check**: the six operations returning real results on
`CleverAS-App/Backend` (needs the repo plus a .NET installation for the host).

### V6 — GitHub Models / Copilot CLI API (verified 2026-08-17)

**Checked:** docs.github.com/en/github-models, GitHub changelog, Copilot docs, plans and billing
pages, Copilot SDK repo, ToS pages. **Found:** **GitHub Models was fully retired on 2026-07-30**
— playground, catalogue, inference API and BYOK all gone, for existing customers included
(closed to new customers 2026-06-16; brownouts July 16/23). GitHub's stated alternatives:
Microsoft Foundry for API access, Copilot for GitHub-integrated workflows. Copilot itself exposes
**no supported OpenAI-compatible chat-completions endpoint**: the official surfaces are the
Copilot CLI programmatic mode and the Copilot SDK (GA 2026-06-02) — agent-session runtimes billed
in AI credits, not a `/chat/completions` with `json_schema` that the PRD's provider layer could
consume; community Copilot proxies are reverse-engineered and unsupported. **Consequence — open
item 9 answered by the PRD's own fallback clause:** "if no or unclear, Scaleway-only and the open
item closes that way". It closes that way: **Scaleway-only at G0**; a second provider, if wanted
later, is a new evaluation (Microsoft Foundry being the successor GitHub itself names). Edit
marked: §3 model-access row and §12 item 9.

### V7 — Scaleway structured outputs (verified 2026-08-17)

**Checked:** Scaleway Generative APIs docs — OpenAI compatibility, structured outputs, function
calling, supported models, per-model cards, pricing, rate limits, data privacy. **Found:**
OpenAI-compatible at `https://api.scaleway.ai/v1`; structured outputs in strict `json_schema` mode
and tool calling documented as supported across the chat catalogue, with per-model cards
confirming; EU (Paris) processing, zero-retention by default, no extraterritorial exposure —
the PRD's GDPR rationale holds. Rate limits at verified identity: ~1–2M tokens/min, 600
requests/min. **Three candidate rungs, each with structured outputs + tool calling + parallel
tools confirmed on its model card:**

| Rung | Model | Size | Price €/1M in/out |
|---|---|---|---|
| 1 — small instruct | `pixtral-12b-2409` (literal ≤12B) — or `gemma-4-26b-a4b-it` / `qwen3.6-35b-a3b` on the active-params reading (4B/3B active) | 12B · 26B(A4B) · 35B(A3B) | 0.20/0.20 · 0.25/0.50 · 0.25/1.50 |
| 2 — mid coding | `qwen3-coder-30b-a3b-instruct` | 30B (A3B), 128k ctx | 0.20/0.80 |
| 3 — best open-weight coding | `glm-5.2` (docs: best open-weight for long-horizon/coding at release, June 2026); cost runner-up `deepseek-v4-flash-0731` | frontier, 256k ctx | 1.80/5.50 · 0.40/0.80 |

Caveat: `gpt-oss-120b` tool calling requires the Responses API — avoid for the ladder. First 1M
tokens free; no production-use restriction. **Consequence:** open item 10 is decision-ready —
verified candidates on the table, Emil picks the rungs.

### V8 — ontology matching (verified 2026-08-17)

**Checked:** LogMap repo and commit history, OAEI 2025 results, AML/Matcha/MELT/LLM-matcher
landscape, Rust crate coverage, the 2026 stable-matching result. **Found:** LogMap is actively
maintained (commits to 2026-08-07, OAEI-2025 participant) and invocable as `java -jar` — but it is
built for the wrong problem here: its differentiator is logic-based repair over rich OWL
axiomatisation, worthless on shallow code-extracted ontologies; its lexical layer leans
biomedical and does nothing for camelCase/snake_case identifiers; the 2021 binary is stale
(build-from-source + JVM in every deploy target). At registry scale (hundreds of classes,
10⁴–10⁵ pairs) the big-ontology machinery buys nothing — runtime is a non-issue either way. No
Rust ontology-matching crate exists; every ingredient does (strsim/rapidfuzz, petgraph,
horned-owl, oxigraph already in-workspace, thesaurus for WordNet). Evidence bound: a
lexical+structural core with stable-matching extraction reaches F1 0.832 on Anatomy
(arXiv:2605.09184) — and signal weights barely matter once extraction is stable matching.
**Consequence — open item 11's matching half answered, recommendation:** reimplement the small
lexical + structural core as rules in Rust (~1–2 weeks: identifier-aware normalisation,
Jaro-Winkler + token-Jaccard + TF-IDF, domain synonym table, one round of neighbourhood
propagation, stable-matching 1:1 extraction with per-signal score breakdown for the review UI);
run LogMap **once, offline, as a calibration oracle** for thresholds. An optional LLM-judge pass
over top-k candidates (+2–3 days) is where the field's marginal quality now comes from and fits
the per-triple review stage naturally — a later choice, not a G0 need.

### V9 — OWL-RL at projection build (verified 2026-08-17)

**Checked:** the sidecar landscape (owlrl, Jena, RDFox, EYE, HyLAR, Soufflé, Nemo, Rust crates)
and the CONSTRUCT-rules prior art. **Found — the options, ranked, no measurement taken:**

1. **`reasonable`** (Rust crate 0.4.4, 2026-05-28, BSD-3) — OWL 2 RL materialiser, the only
   maintained in-process no-new-runtime option; caveat: not 100% of RL rules — the supported
   subset must be checked against the entailments §5.2 needs (subclass transitivity, domain/range,
   inverses, property chains). Integration cost: low (Cargo dependency).
2. **CONSTRUCT-to-fixpoint on Oxigraph** — zero new dependency; reuses the exact shape of the
   workspace's existing `pf::sparql_rules`; the OWL 2 RL/RDF rule table translates mechanically
   (SPIN OWL-RL is the prior art). Caveats: no published fixpoint performance data (V1) and the
   blank-node CONSTRUCT issue (#220). Integration cost: low, rules vendored in-repo.
3. **Nemo** (knowsys, Rust, v0.10.1 2026-08-01, MIT/Apache-2.0) — Datalog engine with native RDF
   I/O, library or `nmo` subprocess; RL rules written once in its dialect. Integration cost:
   low-medium.

Sidecars dragging in runtimes (Python owlrl 7.6.2 — slow but now with Oxigraph-store
integration; Jena 6.2.0 — JVM and no stock OWL CLI, custom wrapper needed; RDFox — commercial,
Samsung-owned, licence-gated) are all viable but strictly worse fits for a Rust workspace. EYE,
HyLAR, Soufflé, whelk-rs: not credible here. **Consequence:** open item 11's reasoner half has
its options established with integration costs; the architecture is indifferent, as the PRD
stated (inference is a projection concern) — the ruling stays Emil's, and options 1+2 are the
two that add zero runtimes.

## 2.3 Blocked checks — the ask

Cross-tier `add_repo` refuses the corpus repos from this session. The blocked checks, exactly:

| Check | Needs | Discharged by |
|---|---|---|
| V3: Android repo's Gradle and AGP versions against the ≥ 8.8 silent floor; one kotlin-lsp import run confirming variants resolve | `CleverAS-App/Android` | a session seeded with the repo as an initial source (version read), plus any machine with JDK/`bin/intellij-server` for the import probe |
| V4: iOS repo project shape (SwiftPM vs xcodeproj), Xcode version; a completing build + index materialisation; the six operations returning real results | `clever-dk/Clever-iOS` **and a macOS machine with the client's Xcode** | Emil's machine, or a macOS runner; not any Linux session |
| V5: the six operations returning real results on the backend solution via `roslyn-language-server` | `CleverAS-App/Backend` + .NET installation | a session seeded with the repo, or Emil's machine |

Options: (a) Emil authorises fresh sessions seeded with the CleverAS-App/clever-dk repos as
initial sources for the two non-macOS checks; (b) the checks run on Emil's machine at G0 entry;
(c) both — the version reads via (a) now, the build-dependent runs via (b). The session makes no
choice.

## 2.4 PRD edits marked by Step 2 (held for the Gate 2 ruling, not applied)

| PRD section | Edit |
|---|---|
| §3 substrate, serving row | Oxigraph verified (0.5.9, 2026-06-18); note the workspace's 0.4 pin as superseded — upgrade at G0; drop "both store candidates asserted from knowledge; verify at G0" |
| §3 substrate, TUI row | ratatui verified (MIT, 0.30.2; `ratatui-textarea` for the prompt window); drop "asserted from knowledge" |
| §3 substrate, model access row | GitHub Models retired 2026-07-30; Scaleway-only; second-provider slot closes (Microsoft Foundry named as the successor evaluation if ever wanted) |
| §5.2 subClassOf row | for Kotlin, typeHierarchy does not exist in kotlin-lsp; extraction path synthesised or deferred (per the Gate 2 ruling on the scope cut) |
| §5.4 adapter table | C# row: definition + typeHierarchy wiring to add, Roslyn typeHierarchy to probe. Swift row: precondition sharpened — macOS runner, client's Xcode, xcode-build-server for xcodeproj, no Linux path. Kotlin row: Alpha; AGP experimental; Gradle ≥ 8.8 silent floor; JDK 25; no typeHierarchy; fwcd fallback end-of-life |
| §12 open items 8, 9, 10, 11, 12 | 8: answered (Oxigraph holds; 0.5 upgrade). 9: closed Scaleway-only. 10: candidates verified, Emil picks. 11: options established (V8/V9 above), Emil rules. 12: ratatui verified (framework half) |

**GATE 2 — holding** on this verification report. The rulings the gate needs: (1) the V3 scope
cut — G0 as C# + Swift with Kotlin joining on its two conditions, or hold G0 for the Android
checks; (2) how to discharge the blocked checks (§2.3 options); (3) leave to apply the §2.4 PRD
edits; (4) optionally now or at G0: ladder rungs (V7 table) and the reasoner option (V9).
