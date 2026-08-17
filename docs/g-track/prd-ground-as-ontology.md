# PRD — Ground as Ontology: Bootstrapping, Entity Surfacing, and the Governed Loop

**Product:** `product-cli` (Hafeok/product-cli), new track. Working name **G-track**.
**Status:** Draft for Emil's ratification. Nothing here is built.
**Provenance:** Written 2026-08-14 against the ground-axes holding note at revision 13 and canon as
last verified by the related-work gate-pass session (a312a55). Constructs from the holding note are
**unratified** unless the corpus test's canon session has since filed them; the PRD names which it
depends on (§10). Refresh rule: re-derive at a named commit, never hand-patch.
**Re-pinned at G-1 (2026-08-17):** reconciled against live canon — `actor-indexed-determination`
at `110bf10` (v5.5.0, tagged 2026-08-16, plus two post-tag merges), `decision-driven-design` at
`4848b9e` (pin advanced to v5.5.0, DDD-dec-18). The corpus test's canon session has filed:
`DDD-ground-01…04`, `DDD-delivery-01…04`, `core/13-delivery.md` with `term:delivery` /
`term:undelivered` / `term:presumed-discharge`, and `graph/axis-registry.yaml` (axis-registry/v1,
artefact-not-canon). Edits below carry the marker *(re-pinned at G-1)*; the full re-pin table is
in the G-1 session report (`docs/g-track/g1-session-report-2026-08-17.md`). The ground-axes
holding note is context only from this revision on; canon outranks it everywhere they touch.
**Rulings already taken (Emil, 2026-08-14):** existing entities only — the loop never adds to the
ontology; the whole session's output is a review bundle of new ground, new decisions, and code, for a
reviewer; entity extraction runs on prompt submission, not on user request; **the ontology is
bootstrapped by a code extractor run over multiple codebases sharing a domain**, with the reviewer
ratifying per triple; where codebases share a domain the overlap is a shared ontology they import;
**a codebase is a projection of the domain and cannot be its authority — the domain is the decisions
that came before, and the shared-domain ontology is the retro-filed decision set the extractions are
evidence for; "domain" is a lacking term — the ontology reaches for the organisation's ground
registry across all axis layers, of which software's domain is one projection; the registry is not
part of the what-specification — the what starts at the Event Model, and the registry is the space
the what is expressed over; **extraction is model-free** — RDF inference and rules cover what a model
was penciled in for, and Feature 3 is the ladder's rung zero.**

---

## 1. Purpose

Three features, one substrate. The substrate is the RDF/OWL ontology already validated by product-cli's
SHACL/SPARQL stack, read as the **ground axis registry**: entities are classes, axes are properties
with ranges, regions are class expressions, and subsumption carries specificity.

**Feature 1 — entity surfacing, as stage one of the TUI.** The product surface is a **TUI with a
prompt window** (Emil's ruling). Composing is stage one: as the user types, recognise entities that
already exist in the registry — model-free, no tokens — show what they entail, and let the user
attach ground and see the governing decisions. When the prompt is ready and its metadata attached, it
is a **declared act**; execution (Feature 2) consumes declarations, never raw text.

**Feature 2 — the governed loop.** A controlled LLM loop whose only tools are ground reads, decision
reads and proposals, and LSP access to a code graph. Every tool call is an act with a position, so the
governing set is delivered mechanically per call. The session's output is one review bundle — proposed
ground, proposed decisions, code — for a human reviewer to accept, amend, or reject.

**Feature 3 — bootstrapping by code extraction.** A code extractor reads existing codebases through
their LSP adapters and proposes ontology objects — entities, hierarchies, relations, axis registries —
into a proposal graph the reviewer ratifies per triple. This populates the software layer of the
ground registry (§3, terminology); other layers are out of scope. Run over multiple codebases sharing one
domain, agreement across independent codebases yields higher-assurance ground and disagreement
surfaces the escaped domain decisions that no ratifier ever took. **Feature 3 is G0**: it produces the
code-to-ontology mapping Feature 2's extractors need before their first act.

**Why build it now.** The loop is the first act-site where the holding note's constructs run live and
generate their own corpus. Every session emits delivered-set-versus-emitted-proxy comparisons,
extraction closure records, and proposed-ground provenance — the evidence that ruling 22 (surfacing
obligation), Q23 (remedy routing), and the hostile corpus test all want and do not have. **The loop
pays for itself in evidence before it pays for itself in code.** That is the primary success criterion
(§9), and the code output is second.

## 2. Non-goals

- Ontology growth by the model. Candidate entities the model believes are new are **not** filed
  anywhere by the loop — not as intents, not as proposals. They appear in the session's report as
  observations for the reviewer, untyped. (Emil's ruling; keeps the registry a controlled vocabulary.)
  Feature 3 is not an exception: the code extractor emits a *proposal graph*, never a merge, and the
  reviewer's per-triple acceptance is what populates the ontology.
- Resolving cross-codebase disagreement. Where extractions disagree, the extractor reports the
  dissent; deciding which model is authoritative is a domain decision filed by the reviewer with a
  region and a falsifier. The tooling never picks.
- Populating the ground registry beyond the software layer. Organisational, regulatory, and technical
  registries are named in §3 and not extracted here; G0's output is one layer and is labelled as such.
- **What-extraction.** Recovering commands, events, and workflows from code — bootstrapping the Event
  Model layer the way G0 bootstraps the registry — is a real future instrument with a different
  target and different LSP surface (call graphs, control flow, handlers). It does not ride along.
  Behavioural divergence across the corpus codebases is expected (§5.3) and is not a G0 finding.
- The coverage sweep, the demand map, the projection matrix, arrangement profiles, the escalation
  queue. The loop is the act-site that generates their input; building them here bundles.
- Any canon filing. Decisions the loop proposes are `[PROPOSED]` in the ledger's existing sense.
- Autonomy. The loop halts on the conditions in §7.6 and never self-approves.

## 3. Substrate

**Terminology (Emil's ruling).** "Domain" is used in this PRD in Evans's sense — the business-problem
vocabulary a piece of software addresses — and it is a **lacking term** for what the ontology is. The
ontology reaches for the organisation's **ground registry**: the full set of declared axes across all
layers named in the holding note's §13.9 — domain, technical, organisational, regulatory — plus the
decisions that region them and the trust decisions (Q27) that let ground be supplied by citation.
Software's domain is one ground projection of that registry, and it is the one this PRD bootstraps
first because it is the one extractable from code. Where "domain" appears below, read *the software
layer of the ground registry*. The registry is regioned by decisions; that — not entity coverage — is
what distinguishes it from enterprise-knowledge-graph efforts, which modelled entities without
modelling who decided anything about them.

**The registry is not the what (Emil's ruling).** The registry declares the space; it holds entities,
relations, and axes, and no acceptance predicates about behaviour. **The what starts at the Event
Model** — the first verdict-bearing layer, where behaviour-decisions are filed in store-pair form
(blueprint as constraint, Given/When/Then as criterion). Code is the how: a projection of the what.
The chain, with the projection-as-source diagnostic applying at every step:

| Layer | Holds | Decisions at this layer are about | Falsifier subject |
|---|---|---|---|
| **Ground registry** | entities, relations, axes; trust decisions; naming rulings | the space — regioning other decisions | representational fidelity (does `Order` mean one thing) |
| **Event Model (the what)** | commands, events, read models, GWTs | behaviour — verdicts over the space | outcomes (does ordering behave correctly) |
| **Code (the how)** | implementations | realisation of the what | conformance to the what |

The registry is not decision-free — it holds the trust decisions and the retro-filed shared-domain
set. The split is by subject: **registry decisions region other decisions; what-decisions resolve
behaviour.**

| Component | Source | Note |
|---|---|---|
| Ontology vocabulary | product-cli's existing RDF vocabulary | **Assumption A1:** the axis registry uses this vocabulary, extended with act-relevant properties, not a separate one. Open ruling 25; the PRD takes the extend position. If ruled otherwise, §7.2 changes and nothing else does. |
| **Registry authority** | a dedicated **registry repository** per instance, generated from the versioned **registry template** *(re-typed at G-1 Gate 3: `docs/g-track/registry-template/`; the first instance generates at G0 entry with its parameters — owner, name, base IRI, ratifier — supplied then)* | canonical store of ratified triples; per-triple files in Turtle (the per-claim file pattern — the pattern transfers, the format was never the load-bearing part; Gate 3 ruling); ratification by PR merge; supersession, never rewriting; CI runs SHACL on every change; the founding-decision slot is the instance's first ratified content. **Not** inside any codebase and **not** inside the framework canon repos — it is the owning organisation's ground, not the framework's. |
| **Registry serving** | SPARQL endpoint rebuilt from the registry repo at each merge | read-only; **pinned to a ref** — every Reading served carries the ref, giving Q11 as-of semantics with no extra machinery. Local mode: product-cli embeds a triple store (Oxigraph — Rust, SPARQL 1.1, fits the workspace natively) over a pinned clone; works offline. Central mode: the same store or Fuseki/GraphDB in Azure Container Apps behind Entra ID — the O-track stack arriving early. Store choice is swappable because **the endpoint is a projection of the repo, not the source** (Q30's diagnostic, applied to our own database). **Inference runs at projection build**: entailments are recomputable, so they never enter the authority repo — each endpoint rebuild derives the inferred graph for that ref, every entailed triple `prov:wasDerivedFrom` the asserted triples plus the rule id, assurance = the rule. The repo stays pure assertion; inference inherits as-of discipline for free. *(Verified at G-1, 2026-08-17:)* Oxigraph holds — 0.5.9 (2026-06-18), active ~monthly cadence, SPARQL 1.1 Query/Update/Federated, full named-graph support; honest risk signal is a bus-factor of ~1. **The workspace's `oxigraph = "0.4"` pin is a dead line** (last 0.4.11, 2025-05-21; 0.5.0 shipped 2025-09-13 as a breaking line) — a live stale-ground finding in our own ground; **the 0.5 upgrade is G0 task one** (Gate 2 ruling). *(Ruled at G-1 Gate 2:)* inference mechanism is **CONSTRUCT-to-fixpoint on Oxigraph as primary** — it reuses the `pf::sparql_rules` shape already in the codebase and adds no dependency — with the Rust `reasonable` crate (0.4.4, BSD-3) as the named fallback; recorded as track decision `g-dec-02` with a `revisit_if`: fixpoint wall-time exceeding an acceptable projection-build budget at G0 scale flips to the fallback. The no-benchmark hedge is discharged by measuring at G0, not by choosing conservatively now. |
| Validation | SHACL shapes, SPARQL | as-is; shapes gain the act properties in §4.2 |
| Provenance | W3C PROV-O | `prov:wasDerivedFrom`, `prov:wasAttributedTo`, `prov:generatedAtTime`; named graph per source |
| Decisions | the ledger (L-track) | read for governing sets; write only as proposals |
| Code | LSP-backed language adapters (M-track) | C#, Bicep, Rust, HTML/CSS as they exist; **Swift and Kotlin added for Feature 3** (§5.4) |
| **TUI** | Rust terminal UI in the product-cli workspace — **ratatui, verified at G-1** (2026-08-17: MIT, 0.30.2, live successor of tui-rs; `ratatui-textarea` v0.9.2 in the ratatui org covers the prompt window) | prompt window, enhance panel, ground-state status line, rung and α pickers, bundle summary |
| **Model access** | provider-agnostic layer over OpenAI-compatible chat completions + tool calling + structured outputs | The loop must run on cheap and small models; Claude is not required (Emil's ruling — cost, and the experiment in §7.7). **Scaleway Generative APIs verified 2026-08-14**: OpenAI-compatible with tool calling and structured outputs, EU-hosted (GDPR; data does not leave Europe — relevant for client corpora), catalogue spans small instruct models through current open-weight coding models on one endpoint. *(Verified and ruled at G-1, 2026-08-17:)* **GitHub Models was fully retired on 2026-07-30**; Copilot's official surfaces (CLI programmatic mode, SDK) are agent runtimes, not OpenAI-compatible chat-completions endpoints — the second-provider slot **closes Scaleway-only** per this PRD's own fallback (open item 9). A second provider, if ever wanted, is a fresh evaluation (Microsoft Foundry is the successor GitHub itself names). Scaleway re-verified at G-1: `json_schema` strict mode plus tool calling confirmed per-model across the ladder rungs (§7.7). Structured-output support is load-bearing: §7.5's required emitted-proxy field survives on small models only if the provider constrains output shape. |

## 4. Data model additions

### 4.1 Reading

Every ground reading the loop or the surfacing pass makes is recorded as a three-tuple, per the
holding note's Q11 as amended:

| Field | Type | Source |
|---|---|---|
| `value` | per axis range | the read |
| `as_of` | timestamp | when read — clock, not context |
| `provenance` | controlled / observed / inferred / institutional | **track vocabulary, candidate for the Q25/Q27/Q30 filing wave** (G-track decision `g-dec-01`; ruled at G-1 Gate 1) *(re-pinned at G-1)* |
| `assurance` | instrument / method / trust-decision ref | at what assurance the reading was made |

`institutional` provenance requires a `trust_decision` reference (Q27). A reading with `institutional`
and no reference fails SHACL.

*(Re-pinned at G-1; ruled at Gate 1.)* The PRD as drafted called the four-value set "canon's
provenance typing". The G-1 reconciliation searched both canon repositories at the pinned commits
and found no such typing filed anywhere — the holding note presupposes it (Q12, Q27) but nothing in
live canon carries it. **Emil's Gate 1 ruling:** the value set does not file upstream from this
session; the session that owns it is the queued Q25/Q27/Q30 filing wave, where Q27's trust-decision
mechanism is the backing institutional provenance needs. Until that wave files, the G-track SHACL
shapes own the value set as **track vocabulary**, filed as track decision `g-dec-01`
(`docs/g-track/decisions/`) with a `revisit_if` pinned to the wave — superseded the moment canon
files provenance typing, the track re-pinning by this session's own discipline. The same
disposition covers the Reading tuple's other unfiled halves (the Q11 three-tuple, assurance-on-
reading): track vocabulary, candidate input to the wave, `wasAttributedTo` the G-track. The
finding is forwarded to the wave as evidence — its Q27 filing now has a named consumer waiting.

Readings served from the registry carry: `as_of` = the endpoint's ref, `provenance = institutional`,
`assurance` = the trust decision by which this arrangement connected (§4.4). Local-clone readings are
identical in form — the ref is the clone's, which makes staleness visible rather than silent.

### 4.2 Act

Every tool call in the loop is an act:

| Field | Content |
|---|---|
| `act_id`, `session_id`, `sequence` | identity and order |
| `tool`, `arguments` | the call |
| `position` | asserted properties over registry axes, each a Reading (§4.1) |
| `unevaluated_axes` | axes the extractor could not evaluate for this act — legible partial delivery (`DDD-ground-01`: non-evaluation must never silently become non-applicability — the field is that clause mechanised) *(re-pinned at G-1)* |
| `governing_set` | decision ids retrieved by position (§7.2) |
| `delivered_form` | how the governing set was rendered into the model's context |
| `emitted_proxy` | what the model states it checked against, captured per §7.5 |
| `outcome` | tool result summary |

This is the Q23 output contract, logged. The `governing_set`/`emitted_proxy` pair is the mechanisable
comparison — the primary evidence output. *(Re-pinned at G-1:)* the comparison's stake is now canon:
`DDD-delivery-02` files undelivered governance as escape that presents as governance, and
`term:presumed-discharge` names the record state — a pass indistinguishable from a skip — that the
delivered-vs-emitted check exists to break.

### 4.3 Session bundle

| Section | Content | Reviewer action |
|---|---|---|
| Proposed ground | new Readings the loop asserts, each with full tuple and `prov:wasAttributedTo` the session | accept into named graph / reject |
| Proposed decisions | ledger entries, `[PROPOSED]`, each carrying the `DDD-ground-01` gate enforced at proposal time — a resolvable applicability predicate or explicit universal applicability, with each implemented axis marked mechanically-evaluable or judgement-evaluable *(re-pinned at G-1: canon amended Q1's named-axis gate to predicate-or-explicit-universal)* — and the acts that motivated them | accept / amend / reject through the ledger's existing path |
| Code changes | diff, with each hunk linked to the act(s) that produced it and their governing sets | ordinary review |
| Observations | candidate entities and anything the loop believed was missing from the registry — untyped, for the reviewer only | read; file by hand if warranted |
| Halts | every halt (§7.6) with its reason and what was escalated | resolve or dismiss |
| Evidence | act log (§4.2) in full; summary table of delivered-vs-proxy | none required; it is the corpus |

### 4.4 Connecting is a trust decision

"Opt to connect" is Q27's filing, literally, not by analogy. Configuring an arrangement to use the
central registry files a trust decision — *rely on this registry for ground of these layers* — with
the organisation as the accountable principal behind it. The config entry is typed as that decision
and referenced by every institutional Reading the arrangement makes (§4.1). An arrangement that has
not opted in reads only its local ground. Revoking the trust decision is supersession, and every
inference standing on registry readings inherits the revocation through the assurance field — the
propagation problem (holding note ruling 27) gets its mechanism here: the reference is explicit, so
the affected set is a query.

Multi-organisation registries (Context& serving several clients, each with their own registry) are
out of scope; one registry, one organisation, one ratifier.

## 5. Feature 3 — bootstrapping by code extraction (G0)

### 5.1 Behaviour

For each configured codebase, run the code extractor through the language's LSP adapter and emit a
**proposal graph** — a named graph, `prov:wasAttributedTo` the extraction run, `prov:generatedAtTime`
the run, and pinned to the codebase's commit ref. Proposal graphs land as **branches of the registry
repository**: per-triple files on a branch, so per-triple acceptance is PR review line by line and
ratification is the merge — the mechanism the registry already has, not a second one. Nothing merges
without the reviewer. Accepted triples enter the canonical graph or a shared-domain ontology (§5.3);
the endpoint rebuilds on merge.

### 5.2 What is extracted, and at what assurance

| Code structure | Proposed ontology object | Assurance | Derivation | LSP operations |
|---|---|---|---|---|
| Classes, records, entities, aggregates | `owl:Class` candidates | high — declared | mapping rule | documentSymbol, workspaceSymbol |
| Inheritance, interface implementation | `rdfs:subClassOf` | high — declared | mapping rule | typeHierarchy — *(ruled at G-1 Gate 2:)* where a server lacks it (kotlin-lsp, verified 2026-08-17), the hierarchy is **synthesised** from implementation/typeDefinition/references plus declaration-text slicing, the pattern the C# adapter already carries; no G0 leg sits on JetBrains' roadmap |
| Typed properties, foreign keys | object/datatype properties with ranges | high — declared | mapping rule | documentSymbol, hover |
| Composition and reference | relations | mid — inferred from usage | SPARQL CONSTRUCT rules to fixpoint over raw reference edges | references, definition |
| Namespaces, modules, bounded contexts | domain axis registries (holding note §13.9's layers) — proposed axes carry the axis-registry/v1 quality mark, resolvable / nameable, with an extractor sketch per resolvable axis *(re-pinned at G-1: the format precedent is `decision-driven-design/graph/axis-registry.yaml`, seeded at v5.5.0, artefact-not-canon, 22 axes — the framework program's own instance, not this registry; its promotion path — a validator reads it, plus a ratification act — is the discipline G0's proposal graphs inherit)* | mid — structural | mapping rule + RDFS entailment | workspaceSymbol |
| Naming, comments, string usage | synonyms, `skos:altLabel` | low — heuristic | lexical rules; model optional, default off | hover, textual |

**Extraction is model-free (Emil's ruling).** The pipeline is: deterministic LSP reads, per-language
mapping rules, RDFS/OWL-RL entailment (subclass transitivity, domain/range propagation, inverses,
property chains), CONSTRUCT rules iterated to fixpoint for usage-inferred relations, and SHACL plus
disjointness axioms for the cross-codebase contradiction surfacing of §5.3. Cross-codebase entity
alignment (Swift `Client` vs C# `Customer`) uses **ontology matching** — lexical plus structural
similarity — producing *candidates* for per-triple review, which is where judgement was always going
to sit. *(Verified and ruled at G-1, 2026-08-17:)* the lexical/structural core is **reimplemented as
rules in Rust** (identifier-aware normalisation, Jaro-Winkler + token-Jaccard + TF-IDF, a domain
synonym table, one round of neighbourhood propagation, stable-matching 1:1 extraction with
per-signal score breakdown for the review UI; every ingredient exists as maintained crates, and the
evidence bound is F1 0.832 on OAEI Anatomy for exactly this recipe, arXiv:2605.09184). LogMap is
alive (OAEI-2025 participant) but built for rich axiomatised ontologies and biomedical lexicons —
wrong problem at this scale; it runs **once, offline, as the calibration oracle** for the Rust
core's thresholds, and never ships. The model's residual role is naming suggestions on the lowest-assurance row, and it defaults off.

**Why this does not trip canon's retirement of "closed predicates make intelligence unnecessary":**
extraction is **constructively closed** — the verdict is *computed* by rule; there is no candidate
generation step to make cheap or expensive. The loop's code edits are the opposite case: acceptance
is checked mechanically but candidates are searched, which is exactly why Feature 2 needs a model and
Feature 3 does not. Constructive versus verification closure is the crisp answer to "when is a model
needed at all".

The declared rows are why extraction closes as a predicate for the top half: the language semantics
say what a class is and the LSP returns it. The inferred and heuristic rows are where review earns its
keep. Every proposed triple carries its assurance row as a Reading (§4.1), so acceptance can be
graded rather than wholesale.

**The extractor uses the declaration-level LSP subset only** — documentSymbol, workspaceSymbol,
typeHierarchy, references, definition, hover. Not completion, not refactoring, not diagnostics. This
is what keeps adapter maturity risk (§5.4) off the extractor.

**The extractor's own limit, stated so G0 is not misread.** A codebase projects only the decisions
that reached software. Pricing rules never implemented, organisational structure, procurement
constraints, the reason a field exists — none of it is in LSP output. The code extractor therefore
bootstraps **one layer** of the ground registry, and §5.3's "unfiled decisions" are the ones that
reached code but were never filed. Decisions that never reached code are not unfiled — they are
invisible to this instrument entirely. This is the holding note's §1.2 (the declared-space limit)
applied to the extractor: complete relative to the codebases, silent about every layer that has no
projection in them. Other layers are populated by other extractors or by filing, out of scope here.

### 5.3 Multiple codebases, one domain

Emil's case: one app, backend in C#, iOS in Swift, Android in Kotlin, one domain. Run the extractor
over all three and compare the proposal graphs on **graph structure in ontology terms**, never on
source text — which is why the languages need not match and a fourth costs an adapter and nothing
else.

| Agreement | Reading | Output |
|---|---|---|
| Present in all codebases, same structure | triangulated ground; assurance upgraded per the Q11/Q27 amendments **if roots are independent** | candidate for the shared-domain ontology |
| Present in a majority, absent or different in one | candidate with a named dissent | reviewer decides; the dissent is a finding, not a merge conflict |
| Backend-only | server concern, or a divergence | reviewer classifies |
| Client-only | UI state, or a divergence | reviewer classifies |
| Contradictory relations across codebases | **an escaped domain decision in the wild** — no ratifier said which model is authoritative and each team resolved locally | reviewer files the decision; region and falsifier |

**A codebase is a projection of the domain, not an authority for it (Emil's correction).** The
domain is the decisions that came before — most unfiled, some escaped, made when the domain was
designed and every time a team resolved a divergence locally. The extractor recovers *evidence* of
those decisions from three projections; it does not recover the decisions themselves. Where the
projections agree, they are jointly evidence of an unfiled decision. Where they diverge, they are
evidence that no decision was made — or that one was made and never delivered to two of the three
teams. Asking which repo is authoritative is asking which shadow is the object.

**Shared domain as an ontology object — correctly typed.** The intersection of the proposal graphs is
*not* the shared domain; it is evidence for it. The shared-domain ontology is the **retro-filed
decision set** the intersection supports: the reviewer names and ratifies it as an ontology in its own
right, each triple `prov:wasDerivedFrom` the code extractions and `prov:wasAttributedTo` the
ratification — and **the ratification is the decision, not the extraction**. Each codebase's domain
`owl:imports` it and adds local extensions. That makes "the client and backend models are one model"
a **testable statement**: the shared domain is a SHACL shape, and each codebase conforms or does not.
Consequence: when a fourth codebase appears with a different `Order`, the question is not "which of
four wins" but "does the filed decision govern this projection, and did it arrive there" — a delivery
question, not an authority question.

**"They should be identical" is a decision, and Feature 3 discharges it — scoped to the registry
layer.** Without the registry/what split (§3), any legitimate behavioural difference between clients
would falsify the identity decision: the iOS app and the backend *do* different things, and should.
With it, the decision's region is precisely the registry layer — entities and relations identical;
behaviour expected to diverge per client. File it with that region and a falsifier (any client-only or
backend-only *entity or relation* not declared a local extension).
It is a **retro-filed** decision in `DDD-ground-04`'s sense *(re-pinned at G-1: ratified at v5.5.0,
replacing the holding note's §13.4 as the authority)* and carries its two fields: when the gap was
uncovered, distinct from when the act occurred; and that it was retro-filed. Without the fields,
retro-filing launders escape into coverage; with them it is the ledger-side discharge mechanism for
the escape generators (Emil's v5.5.0 Gate 4 ruling), and the retro-filing act is a claim-layer act —
attributable, falsifiable by one act in the ungoverned interval, subject to claimant calibration.
Re-running the extractor at each ref is
the mechanical check that discharges it — registry divergence becomes a shape violation with a
decision behind it, not a bug report. **Behavioural divergence is not a violation of this decision.**
The declaration-level LSP subset (§5.2) extracts almost only registry-layer structure, so the
extractor and the decision's region are aligned — now on purpose rather than by accident.

*(Ruled at G-1 Gate 2 — the n=2 qualifier.)* While G0 runs two codebases (C# + Swift, Kotlin
joining on its conditions — §5.4), two-codebase comparison still surfaces contradictions, but the
independence-triangulation assurance upgrade **weakens at n=2**: agreement between two independent
roots is evidence, not the three-root upgrade. The upgrade's full form applies from the third root
onward.

**Independence is a field, not an assumption.** Three repos by three teams in three type systems are
genuinely independent roots. Client models *generated* from a shared schema, or a Kotlin model written
by copying the C#, are one source presenting as three — the correlated-failure trap. `prov:
wasDerivedFrom` on the proposal graph is populated by the reviewer where generation or copying is
known. Independence declared, not inferred; the assurance upgrade in the first row applies only where
it is.

### 5.4 Adapters for the G0 corpus

*(Table re-verified at G-1, 2026-08-17; the scope cut below is Emil's Gate 2 ruling.)*

| Language | Server | Status (verified 2026-08-17) | Extractor precondition |
|---|---|---|---|
| C# | existing M-track adapter (`roslyn-language-server`, pinned expectation 5.11.0; `csharp-ls` fallback) | in use; of the declaration-level six, four are wired (documentSymbol, workspace/symbol, references, hover) | wiring: add `definition` + `typeHierarchy` requests (additive on the generic host layer); probe whether the Roslyn server answers typeHierarchy; live-run check on the backend repo pending (Gate 2 route c) |
| Swift | SourceKit-LSP, bundled with Xcode and swift.org toolchains | all six operations implemented, typeHierarchy included; SwiftPM projects background-index by default since Swift 6.1 | **must build first, on a macOS runner with the client's Xcode** — an xcodeproj app needs the `xcode-build-server` BSP shim and a completing build to materialise the index (unit/record files + IndexStoreDB); no Linux path for iOS-SDK code. **The iOS build is a named G0-entry precondition on Emil's machine** (Gate 2 ruling), not a session task |
| Kotlin | JetBrains kotlin-lsp (official; K2; IntelliJ platform) | **alpha** (v262.9593.0, 2026-07-27); partially closed-source; AGP import native but **experimental**, with a **silent no-Android-import floor at Gradle < 8.8**; JDK 25 runtime; **typeHierarchy not implemented** (five of six present); fwcd fallback is end-of-life (self-deprecated, Android-broken) — not a fallback for this corpus | **joins after G0 entry, on two conditions** (Gate 2 scope cut, ruled): (a) the Android repo's Gradle clears ≥ 8.8 and one `bin/intellij-server` import run confirms variants resolve; (b) the Kotlin `subClassOf` synthesis route (§5.2) is implemented. **G0 runs C# + Swift** |

Adapter maturity risk lands on the *loop's* future code-edit surface for Android, not on the
extractor, because the extractor uses only the declaration-level subset (§5.2). Precedent for the
adapter set exists: LSP plugins for Claude Code already cover C#, Kotlin, Rust, and Swift.

### 5.5 Re-runnable by design

Design the extractor to run at each ref and diff against the ratified ontology. New candidates
surface; **disappearing symbols flag decisions whose entity is gone** — the holding note's Q21
decay-of-relevance under a pin, mechanised. Drift detection is out of scope until G4, but the
extractor is built re-runnable from day one because it costs nothing then and everything later.

### 5.6 Guard

The reviewer's acceptance path is **per triple, never per run**. A single review of a large codebase
that accepts wholesale is a rubber stamp, and rubber-stamped ground is manufactured ground.

## 6. Feature 1 — the TUI: compose, declare, execute

### 6.1 The pipeline

The TUI is the delivery mechanism made visible and human-gated before any model runs.

| Stage | What happens | Model |
|---|---|---|
| **Compose** | the user types in the prompt window; the entity extractor matches continuously against registry labels, `skos:altLabel`, and the code identifiers Feature 3 mapped. **Match only** — no open-vocabulary NER, no model-proposed entities | none |
| **Enhance** | per match: entailments (already materialised at projection build — lookups, not inference), related entities via declared properties, governing decisions by region, each with as-of and provenance. The user tabs through candidates, accepts or removes. Accepting is **ratifying the act's position by hand** — a wrong match caught here costs a keystroke; caught after execution it costs a review cycle | none |
| **Declare** | the prepared artifact: prompt text + position + attached Readings (full §4.1 tuple) + governing set + **working set** (per attached entity: symbols, definitions, and reference sites — the Feature 3 mapping as candidates, resolved live through LSP at declare time, each site a fresh Reading) + chosen model rung (§7.7) + α for the act. This is a **declared act** — declaration precedes act, the Q6 discipline as product spine. **Dead symbols surface here**: a mapped entity with no current references is the §5.5 decay detector firing at the useful moment, shown in the TUI before a token is spent | none |
| **Execute** | the loop (§7) consumes the declaration against the selected provider, **context seeded with the working set — the model starts at the code, not searching for it**. "Exploring the codebase" is judgement-mediated retrieval at act time, occasioned supply for what extraction already paid for as standing; the declaration delivers it instead. The cheap model never sees an unprepared prompt: **delivery is complete before the generator enters** | the rung chosen at declare |
| **Review** | the session bundle (§4.3) lands as a registry-repo branch; the TUI shows the summary; ratification stays in PR review | none |

### 6.2 Ground state in the status line

Stage two shows the **ground state at the declared position** before execution. *(Re-pinned at
G-1:)* the typing is canon's orthogonal one (`DDD-ground-02`), not the holding note's four states —
source coverage (covered · declared-empty · undeclared · unknown), resolution (resolved ·
deliberately-open · unknown), and assurance (adequate · inadequate · unknown), with **Unknown never
rendered as a pass**. The status line shows the four-state projection canon recorded for exactly
this use — governed (covered ∧ resolved) / deliberately-open / declared-empty / **undeclared** —
with the note's "inert" renamed to canon's declared-empty and "uncovered-undeclared" to undeclared.
Where a governing decision's resolution is deliberately-open, any timing display uses the
"—(open)" value (`DDD-ground-03`): a deferred verdict is not a position in time. *(Ruled at G-1
Gate 1: projection for the human, orthogonal for the machine — the status line keeps the four-state
projection because a status line is a projection for a human arrangement mid-act, compact and
absorbable over complete; the halt logic and the act log run on the orthogonal values; a future
detail view is where the orthogonal triple displays raw.)* Acting into
ungoverned ground gets its warning at compose time, before the loop's empty-governing-set halt would
catch it. Pre-act escape detection in a status line, and the warning is dismissable: declaring an act
into uncovered ground knowingly is legitimate (exploration), and the dismissal is recorded on the
declaration.

### 6.3 What it is not

Not a suggestion engine and not a chat feature. The enhance panel is an **act projection** in the
holding note's Q28 sense — position-scoped, per prompt, disposable, precision over recall — rendered
before the act. If it carries noise the user stops reading it, which is the §7.1 failure; matches are
ranked by subsumption distance and decisions by exposure, and the panel is capped, not paged. The TUI
is not a chat client: there is no conversation, there are declared acts and their bundles.

### 6.4 Extractor status

The prompt-entity extractor is a position extractor (§7.3) whose act-site is `prompt.submit`. Its
predicate closes over the ontology's label set. Where the prompt contains a term that resembles an
entity but does not match, the panel shows nothing — silence, not a guess. That term appears in
nothing; there is no observations channel on Feature 1 because there is no reviewer in the loop.

## 7. Feature 2 — the governed loop

### 7.1 Tool surfaces, and only these

| Surface | Reads | Writes |
|---|---|---|
| Ground | SPARQL over the registry endpoint (central or local clone, per §4.4) + named graphs; entailment (subclass, related) | propose a Reading into the session's proposal graph — a registry-repo branch, never the canonical graph |
| Decisions | governing set by position; decision by id; predicate text | propose a `[PROPOSED]` entry |
| Code (LSP) | symbols, references, definitions, diagnostics, hover | edits, staged to the session's working tree |

No shell, no web, no file system outside the working tree, no other MCP.

### 7.2 Mechanical delivery per act

Before each tool call is executed:

1. **Extract position** — the extractor for the act-site reads coordinates from the call itself
   (target file/symbol → domain entity via the code-to-ontology mapping; tool name → act kind;
   arguments → whatever axes they carry). Each coordinate is a Reading with as-of = now.
2. **Verify extraction** (Q20) — the extractor's own closing predicate runs: coordinate matches the
   axis's declared type and range; the act sits there. Failure halts (§7.6).
3. **Retrieve** — SPARQL: which decision regions (class expressions) subsume this position. Result is
   the governing set. Subsumption is what makes a decision on `Order` govern an `OrderLine` act.
   *(Re-pinned at G-1: class expressions are one implementation of `DDD-ground-01`'s applicability
   predicates — the gate is predicate-general; factored axes are an implementation where the ground
   admits them, not the ontology of every region. Graph and temporal predicates beyond simple regions
   were ~27–36% of the corpus.)*
4. **Deliver** — render the governing set into the model's context in **decision-then-predicate**
   form: the resolution, then its acceptance predicate text where stated. Record `delivered_form`.
   Where Event Models exist for the region, their GWTs enter the governing set as predicate artifacts
   (the what-layer's criterion store); EM-to-registry conformance — every event field typed by a
   registry entity — is a later feature, not built here.
5. **Execute** the call.

The user never triggers this. It is the act triggering retrieval — mechanical delivery in canon's
sense *(re-pinned at G-1: `term:delivery`, `core/13-delivery.md` — the trigger, not the index, is
what distinguishes the values, and delivery is a property of a decision at an act-site, never of the
decision alone)*.

**Exploration acts.** `lsp.read` beyond the declaration's working set stays available — an unnamed
helper, a config file — and every such use is typed **exploration act** in the log. Not a failure: the
completeness measure of the declaration. *Exploration acts per session* is a §9.1 metric and a ladder
metric; a declaration whose working set is complete shows zero, and the count is the per-session
empirical answer to how good the extraction mapping is.

### 7.3 Position extractors

One per act-site, each a small pure function with a declared closing predicate, committed alongside
the ontology mapping it depends on. Initial set: `lsp.edit`, `lsp.read`, `decisions.propose`,
`ground.propose`. Extractors are governed artefacts — each has a decision file naming its axes.

### 7.4 Ground writes

The loop proposes Readings; it does not assert them. Every proposed Reading carries
`provenance = inferred`, `assurance = model-session:<id>`, `prov:wasAttributedTo` the session, and
`prov:wasDerivedFrom` the Readings it stood on. The reviewer accepting a Reading into a named graph is
the trust decision (Q27) — the acceptance is what converts model output into ground with a principal
behind it. **The loop cannot manufacture ground into the canonical graph. Structurally, not by
policy.**

### 7.5 Emitted proxy

Per act, the model is required to state, in a fixed field before the tool call executes: which
governing decisions it is honouring, and what additional acceptance checks it is applying that were
not in the delivered set. That statement is the `emitted_proxy`. Enforcement is by tool schema — the
call is malformed without it. This is ruling 22's surfacing obligation, implemented as a required
argument rather than a norm.

Code-edit acts additionally emit the predicate the edit is meant to satisfy, in GWT-shaped form where
the model can state one.

### 7.6 Halts

The loop stops and writes a halt record when:

| Condition | Why | Escalation |
|---|---|---|
| Extraction fails to close for a needed axis | the axis is judgement-evaluable at this act-site; delivery would be judgement-mediated | reviewer states the coordinate or marks the axis out of scope |
| Governing set is empty for a code-edit act *and* the position's source coverage is `undeclared` (`DDD-ground-02`; the four-state projection's uncovered-undeclared) *(re-pinned at G-1)* | acting in ungoverned ground | reviewer files a decision, declares the region deliberately-open or declared-empty, or authorises the act explicitly |
| A governing decision's predicate is stated and open | the loop cannot verify against it | reviewer discharges the predicate or accepts the act at judgement |
| Emitted proxy contradicts a delivered decision | improvisation against governance | reviewer rules |
| Reading assurance below the act's declared α (Q11 amendment) | acting on low-assurance ground | reviewer supplies a higher-assurance reading or lowers α for the act |

Halts are not failures. They are the escalation channel the holding note said every detector needed —
and the observations they produce are the corpus.

### 7.7 The model ladder — arrangement variation as the experiment

**Emil's requirement, read through the framework:** run the loop on the smallest model that works, not
the best model available. This is not a budget compromise on the design; it is the design's own
thesis under test. The loop's structure — mechanical delivery of governing sets, closing extraction
predicates, GWTs and SHACL as dense verdicts, retry on halt — is the generator/checker arrangement
(upstream H3), and the framework predicts that **model capability matters less as operational
evaluability rises**. Small-model viability is therefore the fidelity-ceiling claim (§13.10 —
delivery bounds proxy quality before skill enters) run as an experiment: if a small model with
mechanically delivered governance approaches a large model without it, the arrangement is doing what
the framework says arrangements do. Economic closure says the rest: a retry-based arrangement is
economically closed only if generation is cheap — small models are not the fallback, they are what
makes the loop's economics close.

**Protocol.** Same corpus, same delivered governance, same verdicts; vary only the model. Ladder from
a small instruct model up through the current open-weight coding models; measure the §9.1 evidence
outputs per rung — proxy fidelity, halt rate, malformed-call rate, review yield — plus cost per
accepted change. The interesting result is the *knee*: the rung below which the arrangement stops
compensating.

**Rungs (ruled at G-1 Gate 2; all on the verified Scaleway catalogue, structured outputs + tool
calling confirmed per model card; prices are ground with a drift rate — read 2026-08-17, € per 1M
tokens in/out):**

| Rung | Model | Rationale | Price (as-of 2026-08-17) |
|---|---|---|---|
| 1 — smallest viable instruct | `gemma-4-26b-a4b-it` (4B active) | wins the session's small-rung candidates on active-params pricing for a coding task: same input price as `qwen3.6-35b-a3b` (3B active) at a third of its output price; `pixtral-12b-2409` excluded as vision-oriented, not coding | 0.25 / 0.50 |
| 2 — mid open coder | `qwen3-coder-30b-a3b-instruct` (30B, A3B, 128k ctx) | code-specialised | 0.20 / 0.80 |
| 3 — best open agentic | `glm-5.2` (256k ctx) | the catalogue's best open-weight long-horizon/coding model at release (June 2026) | 1.80 / 5.50 |
| 4 — cost-alternate (when budget allows) | `deepseek-v4-flash-0731` | cost-efficient long-horizon point between rungs 2 and 3 | 0.40 / 0.80 (0.08 cached input) |

**The working set is the biggest lever on the knee.** The explore phase is disproportionately why
agentic coding needs large models: long-horizon search over a repo is what small models are worst at.
The declaration removes it — the remaining work, editing within a delivered working set against
delivered predicates, is exactly the shape the ladder predicts small models can hold. Exploration acts
per session (§7.2) is the metric that shows whether the removal held.

**Rung zero exists and is Feature 3.** Extraction runs with no model at all (§5.2) — constructively
closed, rule-derived, model bill zero. The ladder is therefore a Feature 2 instrument only, and the
track's model spend concentrates entirely where generation is genuinely generative. Rung zero is also
the ladder's control: it marks what the arrangement produces before any model capability enters.

**Small-model accommodations, each typed by the framework:**

- **Schema enforcement over trust.** Tool calls are constrained by structured outputs, not by asking
  nicely. Malformed-call rate is recorded per rung — it is the absorption signal, not noise.
- **Delivered form adapts per arrangement (Q28).** Small contexts are small-`C` arrangements: the act
  projection must be compact, precision over recall hardening from a preference into a budget. The
  `delivered_form` field already records what was rendered; per-rung form variants are legitimate and
  logged.
- **Extraction needs almost no model.** Feature 3's high-assurance rows are deterministic (LSP →
  mapping); the model enters only at synonym and usage-inference rows. The ladder is a loop
  experiment; extraction cost is near-zero at every rung.
- **A failed rung is a finding.** A model that cannot hold the tool schema, or improvises past
  delivered governance at a high rate, is an arrangement whose absorbable forms exclude the current
  rendering — Q28's fourth failure mode, measured. The remedy menu is the Q23 table: change the form
  before changing the model.

## 8. Layer and repo boundaries

| Concern | Layer | Where it lives |
|---|---|---|
| Ontology vocabulary, extractors, SHACL shapes | Layer 1 (synchronic) | product-cli, `product-core` |
| Registry (ratified triples) | Layer 1 content, Layer 2 history | the **registry repository** (org-level, new); endpoint is a rebuildable projection |
| Act log, session bundle, halts | Layer 2 (ledger) | product-cli, L-track ledger extended with the Act type |
| Decisions proposed | Layer 2 | ledger, existing path |
| Canon | — | untouched by this track |

The Act log is a new ledger table, not a claim-graph addition. Acts are numerous; they do not enter
the canon graph.

## 9. Success criteria

Ordered. The first is the reason to build.

1. **Evidence.** After N sessions, the delivered-vs-proxy corpus supports or falsifies: whether the
   model's improvisations beyond the delivered set are frequent (ruling 22), where extraction fails
   to close and how fast those axes mature *(re-pinned at G-1: ruling 12 is answered in canon —
   the axis-type mark is a maturity state, not a fixed type (`DDD-ground-01`, the matched-pair
   evidence, ~2.5 weeks nameable→resolvable); the open question is now the transition rate over a
   larger corpus, not the type)*, and whether halts cluster by
   region (Q23 remedy routing). Report, not verdict; the rulings are Emil's.
2. **Review yield.** Proposed decisions accepted per session; proposed Readings accepted; halts that
   led to a filed decision. If reviewers reject nearly everything, the loop is proposing badly and the
   delivered form is wrong before the model is.
3. **Code.** Edits pass their emitted predicates and the existing test suites. Third, deliberately.

4. **The ladder (§7.7).** Cost per accepted change by model rung, and where the knee sits. If the
knee is low — small models suffice under delivered governance — that is the framework's most
commercially legible result to date, and it is the loop demonstrating its own thesis.

And for G0 specifically: **bootstrapping yield** — proportion of proposed triples accepted per
assurance row; number of cross-codebase contradictions surfaced; whether the shared-domain shape, once
ratified, validates all three codebases or reveals divergence the teams did not know about. The last
is the finding the G0 corpus exists to produce.

## 10. Dependencies on unratified constructs

The PRD leans on the following holding-note items. Each is named so that a ruling against it can be
traced to the affected section:

*(Re-pinned at G-1: the table gains a canon-status column — statuses read against upstream `110bf10`
(v5.5.0) and downstream `4848b9e`. "Not filed" means the construct remains holding-note-only and the
dependency stands as originally flagged; the fallback column applies only on rejection, and nothing
was rejected.)*

| Construct | Section | Canon at v5.5.0 | If ruled against |
|---|---|---|---|
| Position / region retrieval (Q2, Q19) | §7.2 | **partially filed** — the gate half is `DDD-ground-01`, ratified amended (predicate-or-explicit-universal; axes marked mechanically-/judgement-evaluable; axes one implementation, predicates general); the retrieval mechanism itself is not filed | the loop degrades to unfiltered decision listing — build stops |
| Extraction verification (Q20) | §7.2 step 2, §7.3 | not filed — the closure-as-filing-commitment consequence is anchored by `DDD-ground-01`'s marking clause; open ruling 13 (extractor decision files) open | halts on closure failure become warnings; corpus loses ruling-12 evidence |
| Reading three-tuple (Q11 amended) | §4.1 | not filed — assurance enters canon only as `DDD-ground-02`'s orthogonal property of ground; the per-reading tuple is note-only, and the provenance value set is not in canon at all (§4.1 note) | assurance halt (§7.6 last row) is dropped |
| Trust decision backing (Q27) | §4.1, §7.4 | not filed — open ruling 27 open | reviewer acceptance of Readings loses its typed meaning; keep the mechanism, drop the name |
| Emitted proxy (§13.10, ruling 22) | §7.5 | not filed as obligation (open ruling 22 open) — but the comparison's stake is canon: `DDD-delivery-02`, `term:presumed-discharge` | the tool schema still requires the field; the *interpretation* as proxy is dropped |
| Registry = existing ontology (ruling 25, A1) | §3, §4.2 | not filed — open ruling 25 explicitly open (the fork this PRD waits behind); A1 stands as assumption | a mapping layer is added between the two vocabularies; §7.2 step 3 queries through it |
| Triangulation-with-independence (Q27 amended) | §5.3 | not filed | the assurance upgrade for cross-codebase agreement is dropped; agreement stays a review signal only |
| Decay-of-relevance under a pin (Q21) | §5.5 | not filed — open rulings 14 and 15 open | disappearing-symbol detection stays a diff, unnamed |

Ratified canon the PRD stands on without qualification: operational closure, the source/assurance
split, the four stores as partition, accountability completeness, C_resolve. *(Re-pinned at G-1:
provenance typing is removed from this list — the four-value set was not located in live canon at
v5.5.0 and is re-marked as unratified holding-note vocabulary; see §4.1.)* Canon the PRD now
additionally stands on, filed at v5.5.0: `DDD-ground-01…04`, `DDD-delivery-01…04`, `term:delivery`,
`term:undelivered`, `term:presumed-discharge` (all draft/projected — filed, not yet settled).

## 11. Phasing

| Phase | Delivers | Gate |
|---|---|---|
| G0 | **First registry instance generated from the template** (parameters supplied at generation; birth provenance in its first commit; founding decision filed by its ratifier); **Oxigraph 0.5 upgrade as task one** (Gate 2 ruling — the 0.4 pin is a dead line, a live stale-ground finding); local embedded store over a clone; Reading and Act types; PROV-O wiring; **code extractor over the corpus — C# first, then Swift** (Kotlin joins on its two §5.4 conditions; Gate 2 scope cut); proposal graphs as registry branches; per-triple review; shared-domain intersection (n=2 qualifier, §5.3) | **G0-entry checklist (Gates 2–3 rulings):** generate the first instance (parameters supplied then); the iOS build completing on Emil's machine (macOS + client Xcode); the Oxigraph 0.5 upgrade as task one; the Android Gradle read and Roslyn typeHierarchy probe discharged via seeded sessions. Gate: Emil ratifies a first shared-domain ontology **by merging**; contradiction count reported; the "should be identical" decision filed with the extractor as its discharge |
| G1 | TUI compose + enhance + declare over the ratified ontology (`prompt.submit` extractor; enhance panel; ground-state line; declarations persisted, not yet executed) | panel renders for a corpus of real prompts; precision measured by Emil's hand-check; a declared act round-trips to disk |
| G2 | Execute stage: the loop (ground + decisions surfaces, no LSP) consumes declared acts against one verified provider; sessions produce bundles | first declaration executed end to end; first bundle reviewed; act log complete |
| G3 | LSP surface; code-edit acts with emitted predicates; halts complete | first code change lands through review |
| G3.5 | **Central endpoint** (container, Entra ID) for arrangements that cannot clone; connection-as-trust-decision config (§4.4) | one non-local arrangement reads the registry through it |
| G4 | Evidence report from N sessions against §9.1, **including the model ladder (§7.7)** run on at least three rungs | Emil reads; rulings 12 and 22 get their evidence; the knee is reported |

G4 is the point. G3 is where it looks like a product. **G0 is where the first real finding lands** —
three codebases that should carry one domain, and the extractor saying whether they do.

## 12. Open before G0

Emil's rulings needed, in order of how much they change the design:

1. **A1 / ruling 25** — registry is the existing ontology extended. If separate, §4.2 and §7.2 gain
   a mapping layer.
2. **Act-site set for G0** — `prompt.submit` only, or `lsp.read` too, so G0 exercises subsumption on
   code entities from the start.
3. **The α default per act kind** — code edits high, reads low; the numbers are yours.
4. **Whether observations (§4.3) should carry any structure at all**, or stay free text by design so
   nothing about them can be mistaken for a proposal.
5. **Corpus for G0** — the app's three repos are the obvious choice; confirm. *(Struck: "which repo
   is authoritative" — a repo is a projection of the domain and cannot be its authority; the authority
   is the retro-filed decision set the reviewer ratifies. See §5.3.)*
6. **Independence declarations for the G0 corpus** — whether any client model was generated from or
   copied from another, so §5.3's assurance upgrade is applied only where earned.
7. **Registry repository location and name** — **re-typed at G-1 Gate 3, not answered**: the
   skeleton is a **parameterised template** (`docs/g-track/registry-template/`, versioned), and
   ownership is a **generation parameter, answered per instance** — the owning organisation is the
   accountable-principal field of every trust decision referencing that instance; the base IRI
   derives from a host the owner controls durably; the ratifier is named at generation. The
   G-track's first instance is generated at G0 entry, its parameters supplied then, and its first
   commit records its birth provenance (template version, parameters, generated-by, date).
8. ~~**Store verification**~~ — **answered at G-1** (2026-08-17): Oxigraph holds (0.5.9, active,
   SPARQL 1.1 Query/Update, named graphs); the workspace's 0.4 pin is a dead line and the 0.5
   upgrade is G0 task one (§3, Gate 2 ruling).
9. ~~**Second provider**~~ — **closed Scaleway-only at G-1** (2026-08-17): GitHub Models retired
   2026-07-30; Copilot's official surfaces are agent runtimes, not OpenAI-compatible endpoints.
   Closed per this PRD's own fallback (§3).
10. ~~**Ladder rungs**~~ — **ruled at G-1 Gate 2**: `gemma-4-26b-a4b-it` / `qwen3-coder-30b-a3b` /
    `glm-5.2`, plus `deepseek-v4-flash-0731` as the cost-alternate fourth point when budget allows
    (§7.7, prices as-of 2026-08-17).
11. ~~**Reasoner mechanics**~~ — **ruled at G-1 Gate 2**: CONSTRUCT-to-fixpoint on Oxigraph as
    primary, `reasonable` as named fallback, `revisit_if` on projection-build wall-time at G0
    scale (`g-dec-02`); ontology matching reimplemented as a Rust lexical/structural core with
    LogMap once-offline as calibration oracle (§3, §5.2).
12. **TUI framework** — ratatui **verified at G-1** (framework half answered). Still open: whether declarations persist as files in
    the working tree (reviewable, diffable, re-executable) or only in the act log — the file option
    makes a declaration a reusable artifact and is the default position.

---

*Not in this PRD by ruling: the sweep, the map, the projection matrix, arrangement profiles, intent
as an object, ontology growth by the model, resolution of cross-codebase disagreement by tooling. Each
is a consumer of what this track emits, or a decision the reviewer owns.*
