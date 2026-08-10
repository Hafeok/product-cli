# PRD — DDD Specification Platform CLI (v1)

**Working name:** `ddd` (final name open)
**Author:** Emil, Context&
**Status:** Draft for review
**Scope tag:** v1 — local repo, C# + Bicep, LSP-backed MCP surface

---

## 1. Summary

A CLI and MCP server that makes a repository's governing decisions explicit, versioned, and
mechanically enforced. It stores predicates, closure claims, decisions, and pattern declarations as
a graph in the repo; exposes C# and Bicep language intelligence to agents as MCP tools; and
intercepts contract-surface edits to demand seam declarations before they land.

The platform is the seam between Context& engagements: determinations resolved once — which
analyzer set, which pattern with which contract, which language closes which predicate — are filed
as graph entries and inherited by every subsequent project. Curation, not mining: the graph is the
source of truth and the code must conform to it.

---

## 2. Background

The tool operationalises Decision-Driven Design. The constructs it enforces are settled in the
framework corpus and are not restated here; the load-bearing ones are:

- **Predicates** — named, reusable acceptance relations `P(c, G)` over artifact classes, with
  ground provenance and tolerance in the signature (`predicate-definition.md`, `predicate-format.md`).
- **Closure claims** — truth-apt, statused findings about which arrangement closes which predicate,
  with boundary clauses and falsifiers. Only claims carry status; predicates never do.
- **Decisions** — volitional entries `basedOn` claims, made by a named principal.
- **Seams** — boundaries carry demand; a boundary is only justified when it encodes something about
  the verdict, and what it encodes must be declared.
- **Escaped decisions** — consequential resolutions with no governing commitment, check, or
  principal. The warning swamp (analyzer rules nobody can explain) is the canonical instance.

**Process layer:** the tool mechanises the *Decision-Allocated Delivery* way of working
(`way-of-working-decision-allocated-delivery.md`) — DAD is what an engagement runs; this tool is
its enforcement substrate. DAD's escape reporting is `ddd report escapes`, and its escape
checklists seed the M5 catalog. DAD's adoption-order steps 1–3 require no tooling and precede the
tool in any engagement.

**Record substrate:** the *Decision Ledger* (`decision-ledger-prd.md`) is canonical for the
decision record — repo-independent `dec:` identity, content-hashed versions, acceptance signing
the hash, per-decision merge, coverage as the primitive. This tool is a producer and consumer of
ledger entries; `.ddd/decisions/` is the bootstrap form until M8 integration. DAD's flat
`decisions.yml` and its ledger validator resolve into `ledger verify`; the readiness/completeness
gates and the four instruments are ledger-owned, ddd-surfaced.

**Why this tool:** prompt rules are exhortation an agent can drift past. A tool in the edit loop is
a policy-level commitment — the check is part of the arrangement, not the agent's residual
discretion. Every intercepted event is also a structured record, which makes the graph's
correspondence dataset (`I(V;S)` vs. real interface cost) accumulate as a side effect of normal work.

---

## 3. Goals and non-goals

### Goals (v1)

1. A repo-local graph store for predicates, closure claims, decisions, analyzer/linter manifests,
   pattern declarations, and seam declarations — YAML, versioned with the code, diffable in PRs.
2. Schema and ontology validation of the graph (`ddd validate`).
3. Governance diffing: declared vs. detected for analyzer rules, linter rules, and pattern
   instances (`ddd diff`).
4. Diagnostic explanation: resolve any diagnostic id to its governing decision, rationale,
   principal, and basedOn claims (`ddd why`).
5. An MCP server exposing C# and Bicep language intelligence (symbols, references, diagnostics,
   edits) to agents through one multiplexed tool surface.
6. Contract-surface interception: edits that create or change public contract surface demand a seam
   declaration before committing.
7. Per-language adapters that contain all language knowledge; the core never mentions a language.

### Non-goals (v1)

- No central/shared catalog service. The graph is repo-local; the directory layout reserves a
  `shared/` split so a remote can be added without redesign.
- No pattern **mining**. Detection exists only to check conformance against declarations.
- No languages beyond C# and Bicep **in v1**. Rust follows as M6 (adapter-cost falsification
  test) and HTML+CSS as M7 (demand-evidenced; flip condition with M6 recorded in §10); no
  TypeScript, no SQL, no YAML-pipeline governance.
- No CI server product. `ddd validate` and `ddd diff` are CI-runnable commands; pipeline templates
  are out of scope.
- No computation of information-theoretic quantities. The tool logs the structural proxies
  (fan-in/out, contract size); analysis is offline.
- No IDE extension. The IDE touchpoint is the `HelpLinkUri` on diagnostics.
- No web application. `ddd render` produces a static, self-contained HTML projection of the
  graph — no server, no SPA framework, no build step, no editable state. The graph is primary;
  HTML is a projection and can never be a second source of truth.

---

## 4. Users

| User | Mode | Primary needs |
|---|---|---|
| Coding agents (Claude Code et al.) | MCP tools | Edit code through a governed surface; resolve `why` inline; declare seams when demanded |
| Context& engineers | CLI | File and review graph entries; run validate/diff locally and in CI; audit escapes |
| Reviewers | PR diff | Graph changes visible as YAML diffs next to the code they govern |

Agents are the primary interactive user. The CLI is the human and CI surface over the same core.

---

## 5. Architecture

Three layers. Dependencies point downward only.

```
┌─────────────────────────────────────────────────┐
│  Governance layer                               │
│  graph store · validators · interceptor ·       │
│  manifest diff · seam obligations               │
├─────────────────────────────────────────────────┤
│  MCP tool surface (ModelContextProtocol 2.x)    │
│  find_symbol · references · diagnostics ·       │
│  apply_edit · why · declare_seam · graph CRUD   │
├─────────────────────────────────────────────────┤
│  LSP hosts (tool is an LSP client)              │
│  roslyn-language-server (C#) · bicep-ls (Bicep) │
└─────────────────────────────────────────────────┘
```

**Design rule:** the LSP protocol is the seam to language intelligence. The core consumes
normalized events and never touches Roslyn or Bicep APIs directly. Language semantics live
exclusively in adapters (§9).

### 5.1 Runtime and delivery

Delivered as a workspace member of the existing `product-cli` repository (Rust), per the settled
sibling-on-`product-core` decision. This reuses, rather than rebuilds: the YAML/Turtle graph store
conventions, the SHACL/SPARQL validation engine (ontology rules in §6 become declarative shapes),
the `product-mcp` MCP server plumbing, the CLI/config patterns, and the skills-install mechanism.
The DDD ontology is its own schema namespace and store; it does not extend the What/How vocabulary.

The process manages the two LSP child processes over stdio (LSP is JSON-RPC; no .NET runtime is
required in the tool itself — the .NET SDK is a prerequisite of the governed repos regardless):

- **C#:** `roslyn-language-server` — official .NET global tool, `--stdio --autoLoadProjects`.
  Known quirks to handle: solution/project open handshake (custom notification), restore
  notifications, diagnostics warm-up on first document. Prior art exists in community wrappers
  (SofusA/csharp-language-server) and may be vendored or referenced.
- **Bicep:** `bicep-ls` from the `Azure.Bicep.LangServer` global tool. Microsoft documents this
  exact use — LSP integration into AI coding tools — so treat it as a supported surface.

Server lifecycle: lazy start on first request per language, reuse across requests, health check,
restart on crash. Solution load latency for Roslyn is accepted v1 cost; `warmup` command provided.

MCP surface is served through `product-mcp`'s existing plumbing (stdio and `--http` transports),
as a distinct tool namespace (`ddd_*`) alongside the `product_*` framework tools. The official
C# MCP SDK referenced in earlier drafts is superseded by this reuse.

---

## 6. Graph store

Location: `.ddd/` at repo root (sibling to `.product/` where both are present; deliberately not
merged — separate ontologies, separate stores, shared storage conventions). All entries YAML,
format-versioned per existing conventions (`claim-format.md` v1, `predicate-format.md` v1).
Ontology rules below are implemented as SHACL shapes + SPARQL rules in the existing validation
engine, in a `ddd:` namespace.

```
.ddd/
  predicates/       # definitional objects; NO status field (schema-enforced)
  claims/           # closure claims; status, evidence, falsifier, boundary clauses
  decisions/        # basedOn edges to claims; named principal
  manifest/
    analyzers.yaml  # one decision entry per enabled Roslyn rule (severity = assurance level)
    linter.yaml     # same for bicepconfig.json rules
  patterns/         # declared pattern instances + their obligation completions
  seams/            # seam declarations harvested from the interceptor
  shared/           # entries intended for promotion to the central catalog (future remote)
  config.yaml       # adapter thresholds, interception mode, ignore globs
```

Ontology rules enforced by `validate` (beyond schema):

1. Predicates carry no status; closure lives only in claims.
2. `depends_on` and `refines` edges are acyclic; `refines` targets must exist.
3. Every decision has ≥1 `basedOn` edge to an existing claim and a named principal.
4. Every manifest entry maps to a decision; every suppression cites a risk-acceptance record.
5. Declared pattern instances reference a pattern predicate in the catalog.
6. Decisions' `basedOn` edges carry a **pin** — the claim's status and `changed` value at
   decision time; basis-loss detection compares pin to current. Claims carry an optional
   revalidation-cadence field (`revalidate_by`). Both are M2 format bumps with migration notes,
   per the format spec's own versioning rules.

Schema changes at any milestone follow the same discipline: format-version bump, migration note,
validation against the declared version so existing entries never break silently.

---

## 7. Command surface (CLI)

| Command | Function | Exit code use |
|---|---|---|
| `ddd init` | Scaffold `.ddd/`, detect solution + bicepconfig, generate manifest skeletons from current rule sets (each marked `UNGOVERNED` pending a decision entry) | — |
| `ddd validate` | Schema + ontology validation of the graph | non-zero on violation (CI gate) |
| `ddd diff` | Declared vs. detected: analyzer rules firing without manifest entries; manifest entries for rules no longer present; pattern declarations vs. DI-wiring detection | non-zero on orphans (CI gate, severity configurable) |
| `ddd why <id>` | Resolve a diagnostic id, pattern id, or seam id to decision → rationale → principal → basedOn claims | — |
| `ddd declare seam\|pattern\|suppression` | Interactive/flagged filing of entries | — |
| `ddd render` | Project the graph to a static, self-contained HTML file: claims (status-coloured), decisions with basedOn edges and pin state, manifest coverage, escapes dashboard; seam map from M4. Regenerated, never edited; single file, no server, no external assets | — |
| `ddd serve` | Start the MCP server | — |
| `ddd warmup` | Pre-load LSP hosts (solution load) | — |
| `ddd report escapes` | List detected-but-undeclared items, stale claims past revalidation cadence, decisions whose basedOn claim changed status | — |

Detection sources v1 — **unified on SARIF ingestion**: one ingestion module; per-language
invocation adapters know only how to produce a SARIF file (C# compiler errorlog; Bicep CLI SARIF
diagnostics output — exact invocations verified against current docs at implementation and
recorded in the README) and which rule-id namespace it carries. Detected state has two sources
per language: *configured* (rule ids/severities from `.editorconfig` `dotnet_diagnostic` lines
and `bicepconfig.json`) and *emitted* (build SARIF). Config parsing is deliberately minimal;
package-default rules with no config line surface via the emitted source only, and `diff` states
when only one source covers a rule.

**Finding taxonomy for `diff`/`report`:** `UNGOVERNED` (configured or emitted, no manifest
entry), `STALE` (manifest entry, rule absent from config and emissions), `UNCITED_SUPPRESSION`
(suppression without a risk-acceptance record), plus in `report escapes`: cadence violations
(claims past `revalidate_by`) and **basis loss** (decisions whose `basedOn` claim changed status
or `changed` date since the decision pinned it).

DI composition-root scan (Scrutor `.Decorate`, MediatR behaviors, Polly registrations) for
package-level pattern detection arrives with M4. LSP diagnostics supplement at edit time from M3,
joining on the same rule-id key as SARIF. Structural pattern detection (heuristic decorator-shape
recognition) is a stretch goal, filed as candidates only.

---

## 8. MCP tool surface

Tools follow the official SDK's attribute model. `language` is inferred from file extension and
overridable.

**Language intelligence (pass-through with normalization):**
`find_symbol`, `references`, `hover`, `diagnostics`, `signature`, `rename`, `apply_edit`.

**Governance:**

- `why(id)` — same resolution as the CLI, returned as structured content for the agent.
- `declare_seam(boundary, verdict_knowledge, contract_location, obligations)` — files a seam entry.
  Empty `verdict_knowledge` returns a warning: seam cost with no demand absorbed.
- `declare_pattern(pattern_id, instance, obligation_answers)` — files a pattern instance; the
  pattern's obligation list (e.g. decorator: ordering, identity, forwarding completeness) is
  fetched from the catalog and unanswered obligations are rejected.
- `graph_query(selector)` — read access to predicates/claims/decisions (e.g. "is predicate X
  closed for artifact class Y in this arrangement?").
- `accept_risk(diagnostic_id, rationale, principal)` — files the risk-acceptance record a
  suppression must cite.

**Interception semantics for `apply_edit`:** before commit, the edit is run through the language
adapter's contract-surface classifier.

- Non-contract edit → applied.
- Contract-surface edit with a matching seam/pattern declaration in the same session → applied,
  linked.
- Contract-surface edit without declaration → **rejected with a structured demand** naming the
  surface touched and the declaration required. The agent declares, then re-applies.
- Config: `intercept: enforce | warn | off` per artifact-class, so trivial repos or spikes can
  downgrade without forking the tool.

Every interception outcome is logged to `seams/` with the LSP-derived structural metadata
(symbol, kind, reference count at creation) — the correspondence dataset rows.

---

## 9. Language adapters

One adapter per language. An adapter answers exactly three questions and nothing else in the
system knows the language exists:

1. **Which symbol events are contract-surface?**
2. **What does visibility mean here?**
3. **What constitutes a signature change?**

### 9.1 C# adapter (initial policy table)

Contract surface: new `public`/`protected` type or member; signature change on same; new
constructor parameter on a public type; new interface member (forces all implementors); new
`[McpServerTool]`-style exported endpoints as configured. Non-surface: `private`/`internal` by
default (`internal` promotable to surface via config for library repos —
`InternalsVisibleTo` is a seam).

Additional C# duties: map build diagnostics ↔ manifest entries; set `HelpLinkUri` convention for
in-house analyzers to `ddd why` deep links; composition-root pattern detection (package-level).

### 9.2 Bicep adapter (initial policy table)

Contract surface: module `param` added/removed/retyped; module `output` change; `@allowed`/
decorator changes on params (tolerance change is a contract change); resource `name`/scope
expressions crossing module boundaries. Ground-provenance mapping: literal default = controlled;
`param` = observed; runtime-resolved (`reference()`, deploy-time functions) = inferred — filed on
the seam entry directly, since Bicep syntax makes provenance first-class.

Additional Bicep duties: `bicepconfig.json` linter rules ↔ manifest; note `what-if` incompleteness
boundaries on relevant closure claims.

Adapter policy tables are themselves sets of claims ("in C#, adding an exported method is
boundary-forming") and are falsifiable against where boundary defects actually occur; wrong rows
get fixed in the adapter, not the core.

---

## 10. Milestones

| M | Deliverable | Proves |
|---|---|---|
| **M1** | Graph store + `init`, `validate`, `why` on hand-filed entries | Ontology enforceable; format v1 survives contact |
| **M2** | Manifest diff from `dotnet build` + Bicep CLI output; `report escapes` | Orphan detection works; the warning-swamp claim pays rent |
| **M2.5** | `ddd render` — static HTML projection of the graph: status-coloured claims, decision chains with pins, manifest coverage, escapes dashboard | The graph is showable: the client-facing artifact exists; the warning-swamp result becomes a page, not a terminal capture |
| **M3** | MCP server with language-intelligence tools over both LSP hosts | LSP hosting viable; one surface, two languages |
| **M4** | Interceptor on `apply_edit` + `declare_seam`/`declare_pattern` with obligations | The differentiated feature; agent flow acceptable |
| **M5** | Dogfood: the tool's own repo governed by itself; first C# closure-claim seed (NRT, type conformance, disposal — with boundary clauses). **Seed source:** the DAD way-of-working's escape checklists (§6.2 C#, §6.3 Bicep) — each item files as a predicate/escape-shape entry plus a closure claim (analyzer-closable items become manifest content; the rest, review-protocol content) | Curation workflow real; catalog seeding demand-driven — the checklists *are* compiled demand evidence |
| **M6** | Rust adapter (rust-analyzer host + policy table: `pub`/`pub(crate)`/`pub(super)` grading, trait defs and impls as contract surface, new impls boundary-forming, orphan-impl cases as known escapes) — post-v1, first item | The §11 adapter-cost claim, exercised: pre-registered measure is adapter cost ≈ policy table + LSP wiring; material excess means language knowledge leaked into the core, fixed before language four. Completes self-governance: interception runs on the tool's own contract surfaces |
| **M7** | **Preamble, two tasks (permitted amendments M6's experiment constraints excluded):** (1) implement `dec/ddd/enum-member-gap-priced` — the one C# policy-row + fixture change; (2) implement `dec/ddd/enforce-matching-tightens-to-symbol` — enforce-mode matching drops the file arm, and `.ddd/seams/seam-rust-default-command.yaml`'s corrupted metadata block is corrected on the same commit. Its acceptance test is already written as its own falsifier and fired at M6 (`seam-event/4`; `DDD-arch-05`, reported). Then: HTML+CSS adapter — governed unit is the **pair**, failures live at their seam. Contract surface: shared stylesheets, design tokens (custom properties as stylesheet params), class vocabulary; `@layer` as visibility grading. Closure stack: html-validate + stylelint via the existing manifest machinery; token-discipline rules (the token file is a manifest of design decisions, each basedOn a claim, resolvable via `why`); a **class contract check** (HTML-referenced classes ⊆ owned CSS definitions, selectors ⊆ present structures — survey existing tooling before building); computable a11y (contrast). "Looks right" stays open; screenshot-diff visual regression converts it to reported-with-evidence. Survey must confirm plain HTML+CSS vs. Razor/Blazor (latter routes partly through the C# adapter) | The adapter abstraction stressed by a non-LSP-shaped artifact class; the observed agent-failure mode (ungoverned HTML↔CSS seam, zero closed predicates, eyeball-only checking) gets an arrangement. **Flip condition, recorded:** M7's basis is demand evidence (repeated observed failures), M6's is a falsification test — if HTML/CSS failures keep costing before M6 starts, the demand claim outranks the test and M6/M7 swap; the flip is a governed priority decision (DDD-org-03) filed with its basis |
| **M8** | **Decision Ledger integration** — the record substrate is the Decision Ledger (`decision-ledger-prd.md`, canonical; its own milestones **L0–L5** run as a separate track — L-prefixed to disambiguate from this document's M-series). At this milestone: `.ddd/decisions/` migrates to `dec:` ids (bootstrap form retired); manifest entries, seam declarations, and risk acceptances become ledger entries with typed `DischargeRef`s (`analyzer:ID`, `whatif:assertion`, `otel:metric+expectation`); basis pins upgrade from status+date snapshots to **version-hash references** (basis loss = pinned hash ≠ current hash, exact); readiness and completeness gates delegate to `ledger verify`; the DAD instruments are owned and computed by `ledger instruments`, surfaced through `ddd report` and `ddd render`. Acceptance discipline inherited: `accepted-by` never resolves to a model identity; model-proposed entries stage until a named human accepts | The process layer, governance mechanism, and record substrate close over each other with owned seams: DAD's flat `decisions.yml` open slot resolves into the ledger; the ddd tool becomes a producer/consumer of an append-only, hash-accepted, per-decision-mergeable record; escaped decisions are non-silent end-to-end and *coverage* — the claim about absence git cannot express — becomes queryable |

Sequencing note: M1–M2 need no LSP and deliver standalone value; M3 carries the load risk
(Roslyn solution load); M4 depends on M3.

---

## 11. Risks

| Risk | Level | Mitigation |
|---|---|---|
| `roslyn-language-server` is prerelease-labelled and editor-oriented; custom handshake may change | Medium | Consume via LSP protocol only; pin version; the handshake handling exists in Rust prior art (e.g. `roslyn-ls`) to vendor or reference; `csharp-ls` as fallback host |
| Coupling to `product-cli`'s release cadence and internals drift | Low–Medium | Depend only on `product-core`/`product-mcp` public crate surfaces; `ddd` namespace isolated; workspace CI runs both tool suites |
| Interception friction — agents or humans route around `enforce` mode | High (adoption) | `warn` default outside Context& repos; thresholded surface definition; obligation lists kept short; measure demand-per-declaration |
| Solution load latency makes the MCP surface feel broken | Medium | Lazy load + `warmup` + explicit "loading" tool responses |
| Curated catalog goes confidently stale | Medium | Revalidation cadence fields enforced by `report escapes`; claims past cadence flagged in CI |
| Scope creep toward pattern mining / more languages | Medium | Non-goals section is normative; new languages require only an adapter + policy table **for contract-surface knowledge — confirmed by M6 (leakage 2 instances / 114 lines, both host-layer, both predicted; DDD-adapter-01 reported). Host lifecycle is the false half (DDD-adapter-04): each new language server costs bespoke wiring — capabilities, readiness signalling, workspace discovery — and must be budgeted per server; rust-analyzer's 114 lines is the reference point** |
| Structural proxy (fan-in/out) is a poor stand-in for `I(V;S)` | Accepted | Logged as candidate correlate only; the correspondence claim stays untested until analysed offline |

---

## 12. Success criteria

v1 succeeds if, on the tool's own repo plus one Context& engagement repo:

1. `validate` + `diff` run green in CI, and every analyzer/linter rule resolves through `why` to a
   principal and a basedOn claim — zero orphaned diagnostics.
2. An agent completes a real feature through the MCP surface, including at least one intercepted
   contract-surface edit resolved by a seam declaration, without human unblocking.
3. The seams log contains ≥20 declarations with structural metadata — the first correspondence rows.
4. A second repo is initialised from the first's `shared/` entries and inherits ≥1 decision
   without re-litigating it — the cumulative effect, observed once.

Failure signals worth pre-registering: interception overridden to `off` within a week of dogfood
(friction claim falsified); `why` unused by agents (governance-as-ground claim weakened); manifest
diff produces only noise (warning-swamp claim needs revision).

---

## 13. Decisions embodied (to file in the graph at M1)

- Delivery as a workspace member of `product-cli`, reusing store/validation/MCP infrastructure,
  with the DDD ontology as its own namespace — the existing sibling-on-`product-core` decision,
  basis rechecked against the current repo state and confirmed, now with the runtime consequence
  (Rust host, .NET language servers as children) recorded.
- v1 scope: local repo, C# + Bicep, build-output detection first — basedOn claims about where
  Context& demand concentrates and Bicep's adapter cost.
- LSP protocol as the language-intelligence seam; no direct Roslyn/Bicep API dependency in core.
- Seam detection is an edit-flow interceptor consuming LSP events, not an LSP extension.
- Predicates carry no status; enforcement structural, not conventional.
- Curation over mining; detection is conformance checking.
- Contract surface defined per-language in adapter policy tables; tables are falsifiable claims.

- The Decision Ledger PRD adopted as canonical for the decision record substrate; ddd M8 inverted
  from "build a feature-ledger family" to "integrate the ledger" — basedOn the claims that
  acceptance must sign a content hash to mean anything, that seam decisions need repo-independent
  identity, and that per-decision merge with no auto-resolution is the record's load-bearing
  constraint. Ledger OD-2 resolved with today's architecture; basis pins upgraded to version-hash
  references at M8.
- Decision-Allocated Delivery adopted as the process layer the tool mechanises; its checklists
  seed M5, its ledger becomes the M8 feature-ledger family, and the schema seam between ledger
  decisions and graph decisions is settled at M8 deliberately rather than shipping two uncoupled
  decision formats. DAD canon-synced on adoption (chain-rule seam form; ground-distribution
  qualifier).
- HTML+CSS admitted as a governed artifact class (M7), governed as a pair at their seam — basedOn
  the demand claim from repeated observed agent failures: no closed predicates, undeclared class
  contract, eyeball-only checking. Ordered after M6 because the cheap adapter-abstraction test
  runs before the expensive stress; the M6/M7 flip condition is recorded in §10 and flipping it
  is a governed priority decision, filed with its basis when taken.
- Rust deferred to M6 rather than added to v1 core — the v1 scope decision's basis (Context&
  payload demand concentrates in C# + Bicep) was rechecked and holds; the competing dogfood-
  completeness claim is served by sequencing, and M6 doubles as the pre-registered test of the
  adapter-cost claim. M1–M2 dogfooding is language-agnostic and unaffected.
- Detection unified on SARIF ingestion with thin per-language invocation adapters — basedOn the
  claim that both toolchains emit SARIF and that one structured format beats two console parsers.
- Basis-loss detection requires status pins on `basedOn` edges; added as an M2 format bump rather
  than deferred, because escaped-decision reporting is load-bearing for the tool's thesis.
- Tests run against committed SARIF fixtures; no .NET SDK in workspace CI, with a gated
  integration test for local regeneration.
- The tool's repo is its own first user; these entries are milestone acceptance content, not
  documentation.

---

## 14. Open questions

1. Final name and command prefix (`ddd` collides with nothing on PATH but is generic).
2. `internal` default for C# contract surface: off (app repos) vs. on (library repos) — per-repo
   config default undecided.
3. Whether `apply_edit` rejection returns the declaration *template* pre-filled from LSP metadata
   (lower friction, risk of rubber-stamping) or empty (higher friction, honest declarations).
4. Seam-entry granularity for Bicep: per-param or per-module-contract.
5. Whether M5's closure-claim seed lives in `shared/` from day one or is promoted later.
