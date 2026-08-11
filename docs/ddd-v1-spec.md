# DDD v1 Specification — the implementable contract

**Covers:** M1–M7 as built (graph store, validation, diff/report, render,
MCP surface, interception, the C#/Bicep/Rust/HTML+CSS adapters).
**Companions:** [`ddd-adrs.md`](ddd-adrs.md) (settled architecture
decisions), [`ddd-research-protocol.md`](ddd-research-protocol.md)
(experiments and instruments), [`ddd-roadmap.md`](ddd-roadmap.md) (M8 and
filed follow-ups), [`ddd-cli-prd.md`](ddd-cli-prd.md) (the umbrella).
**Provenance:** split out of the PRD per `dec/ddd/prd-split`, incorporating
the 2026-08 review
([`reviews/ddd-cli-prd-review-2026-08.md`](reviews/ddd-cli-prd-review-2026-08.md)).

This document states what the tool **guarantees today**. Where a normative
invariant is not yet met, the gap is named and points at the graph entry
or roadmap item that owns it — nothing is silently claimed.

The enforcement boundary, stated honestly up front:

> **Interception governs the governed path; CI governs the repository.**

Edit-time interception binds edits that arrive through `ddd_apply_edit`
(`DDD-arch-08`: nothing binds an edit to arrive that way). The
repository-level half — contract-surface validation over a diff, in CI,
sharing the interception classifier — is M8 scope
(`dec/ddd/m8-enforcement-closure`), not a v1 capability.

---

## 1. Identity

A CLI (`ddd`) and MCP server (`ddd serve`) that make a repository's
governing decisions explicit, versioned, and checkable. The graph under
`.ddd/` stores predicates, closure claims, decisions, analyzer/linter
manifests, pattern instances, seam declarations, and interception events.
**Curation, not mining:** the graph is the source of truth and the code
must conform to it. Detection exists to check conformance against
declarations, never to infer them.

| User | Mode | Needs |
|---|---|---|
| Coding agents | MCP tools | Edit through a governed surface; resolve `why` inline; declare seams when demanded |
| Engineers | CLI | File and review entries; run validate/diff/report locally and in CI |
| Reviewers | PR diff | Graph changes visible as YAML diffs beside the code they govern |

### Non-goals (normative)

No central/shared catalog service (the `shared/` split reserves the
layout; L4 owns real federation). No pattern mining — detection is
conformance checking only. No languages beyond the shipped four adapters
without their own decision. No CI server product — `validate`, `diff`,
`report` are CI-runnable commands; pipeline templates are out of scope.
No computation of information-theoretic quantities (structural proxies
are logged; analysis is offline). No IDE extension (the IDE touchpoint is
`HelpLinkUri` deep links). No web application — `render` is a static,
self-contained projection and can never be a second source of truth.

### Standing risks

Host fragility (`roslyn-language-server` is prerelease-labelled;
consumed via LSP protocol only, version pinned, `csharp-ls` as fallback);
solution-load latency (lazy load + `warmup` + explicit loading
responses); interception friction driving `off`-mode bypass (pre-
registered failure signal — research protocol §4); the curated catalog
going confidently stale (cadence fields enforced by `report escapes`);
coupling to `product-cli` internals (public crate surfaces only — ADR 1);
the structural proxy being a poor stand-in for `I(V;S)` (accepted;
logged as candidate correlate only).

---

## 2. Normative invariants

The ten invariants below are the review's recommended set, adopted
near-verbatim as this specification's normative core. They are the
foundations automated tests are written against.

1. Every governed finding resolves to exactly one current governing
   decision or is reported as ungoverned.
2. Every governing decision has a stable identity, named principal, typed
   basis, and pinned basis version.
3. No declaration can discharge a code change other than the change it
   explicitly identifies.
4. MCP interception and repository-state validation use the same contract
   classifier.
5. Direct edits cannot bypass the CI-visible governance result.
6. The governance-core crate cannot import language-specific knowledge.
7. Projections and reports never modify the graph.
8. Validation results are deterministic for a fixed repository state and
   tool version.
9. Failure or absence of a language host is reported explicitly and never
   interpreted as "no findings."
10. Imported decisions preserve origin identity and their pinned upstream
    version.

Status against the build:

| Invariant | Status today |
|---|---|
| 1 | **Holds.** Ontology rule 4 + three-way `why` resolution (`dec/ddd/why-resolves-three-ways`): a detected-but-unfiled rule reports as ungoverned, never as not-found. |
| 2 | **Holds as of format 5** (typed basis, `dec/ddd/typed-basis`). The basis pin is the claim's status + `changed` date — a heuristic, per the review; the content-hash upgrade is filed for M8 (ruling: pulled into M8 with the enforcement chain, not earlier). |
| 3 | **Not yet met.** Matching is session-scoped and symbol-exact (`DDD-arch-09`: it discharges whatever change next touches the symbol). Declaration signing (subject symbol, before/after content hash, base revision) is M8's chain. |
| 4 | **Not yet met.** Repository-state validation does not exist; when it arrives (M8) it MUST share `ddd_lsp::classify` — two classifiers would fork the definition of contract surface. |
| 5 | **Not yet met.** `DDD-arch-08`; M8's CI half. Until then the honest sentence above is the guarantee. |
| 6 | **Holds.** `ddd-core` contains no language knowledge; adapters live in `ddd-lsp`, keyed by policy tables. |
| 7 | **Holds.** `render`, `report`, `why`, `what` are read-only over the store. |
| 8 | **Holds.** Store loads are filename-sorted; rules are declarative; `report escapes` takes `--today`-style inputs from the clock but the same store + same day reproduce byte-identically. |
| 9 | **Holds on the governed path.** A host that fails to start or answer surfaces as a tool error; `diff` states when only one detection source covers a rule. No path reports absence-of-host as a clean result. |
| 10 | **Vacuously open.** v1 has no import mechanism — see §5 `shared/`. |

---

## 3. Architecture

```
   CLI (ddd)          MCP surface (ddd serve, ddd_* tools)
        \                    /
         v                  v
        Governance core (ddd-core: store, validate,
        diff, report, why, render — no language knowledge)
                    |
                    v
        Language adapters (ddd-lsp: policy tables,
        classifier, enrichment)
                    |
                    v
        Hosts (roslyn-language-server, bicep-ls,
        rust-analyzer — LSP children; none for the
        hostless HTML+CSS pair producer)
```

CLI and MCP are **two entry surfaces into the same core**; CLI validation
is fully usable without MCP or any host. Dependencies point downward only.

Delivery: workspace members of `product-cli` (`ddd-core` / `ddd-lsp` /
`ddd-mcp` / `ddd-cli`), reusing `product-core`'s store conventions,
SPARQL validation engine, and `product-mcp`'s server plumbing
(`dec/ddd/workspace-member-delivery`). MCP tools are defined at the
**protocol level** — JSON schemas served over MCP by the reused Rust
plumbing. (An earlier draft's reference to an official C# SDK "attribute
model" described a superseded implementation choice and is retired.)

**Operational prerequisite, stated from the user's perspective:** the
`ddd` binary embeds no .NET, but governing C# or Bicep launches
`roslyn-language-server` / `bicep-ls`, so a working .NET installation is
a prerequisite of any repo governed for those languages. Rust governance
requires `rust-analyzer`; HTML+CSS requires no host.

---

## 4. Adapter capabilities

An adapter's **classifier contract** is the three questions: which symbol
events are contract-surface, what visibility means, what constitutes a
signature change. That formulation survives as the contract of exactly one
capability — not of the adapter as a whole. The monolithic "adapter"
decomposes into:

- **Language host** — child-process lifecycle, readiness signalling,
  workspace/solution discovery. Priced per server (`DDD-adapter-04`, the
  evidence that forced this split: host wiring is bespoke per server,
  ~114 lines for rust-analyzer, and is *not* covered by the policy-table
  cost claim).
- **Diagnostic provider** — normalized diagnostics from the host at edit
  time, joining SARIF's rule-id key.
- **Configuration provider** — the configured-rule source
  (`.editorconfig`, `bicepconfig.json`, `.stylelintrc.json`, …).
- **Contract-surface classifier** — the policy table + before/after
  surface comparison. The three questions live here. Policy rows are
  falsifiable claims (`dec/ddd/adapter-policy-tables`); wrong rows get
  fixed in the table, not the core.
- **Optional detectors** — composition-root / pattern detection
  (conformance checking only, `dec/ddd/curation-over-mining`).

A producer need not be LSP-shaped: the HTML+CSS pair adapter is hostless
(classification over source text; `DDD-adapter-05` prices the
producer-shape boundary at bounded serve-layer cost, never
surface-vocabulary changes).

---

## 5. The graph store

`.ddd/` at repo root; all entries YAML, format-versioned 1–5
([`ddd-format-migrations.md`](ddd-format-migrations.md) is the migration
record; validation is always against the declared version, so entries
never break silently).

```
.ddd/
  predicates/      # definitional; NO status field (schema-enforced)
  claims/          # truth-apt, statused; falsifier + evidence discipline
  decisions/       # volitional; named principal; >=1 typed basis
  manifest/        # analyzers.yaml, linter.yaml, ... one entry per rule
  patterns/        # declared instances + obligation completions
  seams/           # declarations; seams/events/ = interception rows
  shared/          # reserved: entries intended for promotion (see below)
  config.yaml      # modes, adapters, pair map, detection sources, ignores
```

Ontology rules (schema layer + SPARQL over the Turtle projection):

1. Predicates carry no status; closure lives only in claims.
2. `depends_on`/`refines` acyclic; `refines` targets exist.
3. Every decision has **≥1 typed basis** (`claim | constraint | mandate |
   preference | experiment | risk-acceptance`; a claim basis is a pinned
   `basedOn` edge; a risk-acceptance basis resolves to its record) and a
   named principal.
4. Every manifest entry maps to a decision or is `UNGOVERNED` (warning);
   every suppression cites a risk-acceptance record.
5. Pattern instances reference a pattern predicate in the catalog.
6. Claim-basis edges carry a pin (status + `changed` at decision time);
   basis loss compares pin to current.

Writes are atomic per file (temp + rename via `fileops`); a failed write
never leaves a partial entry. Ids are stable identities — a duplicate id
anywhere in the store is a violation; filenames are a slug of the id, not
the identity itself. Renaming or deleting an entry is a graph edit like
any other and lands in the same PR diff as the code it governs.
Canonical id syntax beyond these rules (reference/version syntax,
tombstones, cross-file transactions, hashing canonicalization) is
**unspecified in v1 — fails loudly**: `validate` rejects what it cannot
resolve, and the full identity model is settled at M8 where ledger `dec:`
ids arrive.

**`shared/` is import, not inheritance.** Copying an entry from another
repo's `shared/` is a v1 *import*: nothing tracks origin, upstream
version, or divergence. True inheritance — origin identity and
provenance, pinned upstream version, update detection, local override
rules, divergence handling, basis-loss/revocation propagation (the
review's bar, adopted verbatim) — arrives with the ledger's federation
layer (L4, `decision-ledger-prd.md` §9.4), where promotion is publishing
a repo that downstreams pin by SHA.

---

## 6. Command surface

| Command | Function | CI use |
|---|---|---|
| `ddd init` | Scaffold `.ddd/`; manifest skeletons from current rule sets, each `UNGOVERNED` pending a decision | — |
| `ddd validate` | Schema + ontology validation | exit 1 on violation |
| `ddd diff` | Declared vs. detected: `UNGOVERNED` / `STALE` / `UNCITED_SUPPRESSION`; per-finding severity from config | exit 1 on error-severity findings |
| `ddd report escapes` | Diff findings + cadence violations + basis loss + pair-contract check | exit 1 on escapes |
| `ddd why <id>` | id → decision → rationale → principal → typed bases (with pin drift) | — |
| `ddd render` | One static, self-contained HTML projection; regenerated, never edited | — |
| `ddd what [--strict]` | What-graph boundaries carrying no governing declaration | exit 1 under `--strict` |
| `ddd serve` | The `ddd_*` MCP surface (stdio) | — |
| `ddd warmup` | Pre-load LSP hosts | — |

Detection is unified on SARIF ingestion (`dec/ddd/sarif-unification`)
plus config parsing; sources per rule are *configured* and *emitted*, and
`diff` states when only one source covers a rule. Machine-readable
(`--json`) output for the CI-facing commands is **not yet shipped** —
named as a gap; lands with the M8 CI work, where a stable finding-id
scheme is required anyway.

### The rule-state model (specification)

A rule's detected state is five-valued:

| State | Meaning |
|---|---|
| Available | Rule exists in an installed analyzer/tool |
| Configured | Repo config assigns it severity/settings |
| Executed | Rule participated in an analysis run |
| Emitted | Rule produced ≥1 diagnostic |
| Governed | Rule maps to a governing decision |

**The shipped `diff` implements the weaker M2 semantics**: it observes
only *configured* and *emitted*, and `STALE` currently means "manifest
entry whose rule is absent from both." That conflates "not installed"
with "did not fire": absence of emission is not evidence a rule no longer
exists. Staleness SHOULD be established against the installed rule
catalog (*available*). The rework is **filed, not silently claimed** —
see the roadmap's rule-state/STALE follow-up (bounded, no ledger
dependency, schedulable independently).

---

## 7. Interception, as built

Three concepts the tool keeps distinct — the classifier detects the
first, humans file the third:

| Concept | Meaning |
|---|---|
| **Contract-surface event** | Mechanically detected symbol/syntax change (the classifier's output — what `.ddd/seams/events/` rows record; the `seam-event/<seq>` id family predates this naming and is kept for continuity) |
| **Seam candidate** | An event that may create or alter a demand-bearing boundary |
| **Seam declaration** | The filed account of the actual boundary, absorbed demand, and obligations |

A public-API change is *evidence of a possible seam obligation*, not the
creation of a seam; the tool detects a proxy and demands a declaration —
it never claims to have detected a seam.

Mechanics (`ddd_apply_edit`, single file per call):

1. Read the file's current text; resolve the proposed new text.
2. Mode per artifact class (`enforce | warn | off`, config
   `intercept_by_class`). `off` writes without classification.
3. Hosted path: open the document, snapshot `documentSymbol`, overlay the
   new text in the host, snapshot again; classify before/after through
   the adapter's policy table. Hostless path (HTML+CSS): classify source
   text directly.
4. No surface events → write atomically, done. Surface events → match
   against **same-session** declarations (the serving process's
   in-memory log; it does not survive restart). Enforce mode matches
   symbol-exact (`dec/ddd/enforce-matching-tightens-to-symbol`); warn
   mode links generously but applies regardless.
5. Enforce + all matched → write, link declaration metadata, log rows.
   Any unmatched → **reject with a structured demand** (facts pre-filled
   from the LSP, judgment fields blank,
   `dec/ddd/rejection-facts-prefilled`), restore the host overlay to disk
   state; the file is untouched (disk is written only on apply, so
   rejection needs no rollback).
6. Every classified surface outcome logs one event row (the
   correspondence dataset; reference counts queried per event where a
   host exists).

Gaps, stated: single-file edits only (no multi-file atomicity — a
multi-file change is N independent interceptions); session-scoped
matching admits discharge of unidentified changes (`DDD-arch-09`; M8's
declaration signing closes it); one serving process is assumed — two
concurrent servers race on event sequence numbers; formatter-only churn
is distinguished only as far as the policy table normalizes signatures;
and routing to the interceptor is voluntary (`DDD-arch-08`) — the
repository-level check is M8. Interception governs the governed path;
CI governs the repository.

---

## 8. Operational requirements

Expected v1 behaviour, one line each. "Unspecified — fails loudly" is a
named behaviour; silence is not.

- **Monorepos / multiple solutions:** first solution/workspace found per
  host; multiple-root selection unspecified — host errors surface as tool
  errors, never as clean results.
- **Repository-root discovery:** walk up from cwd to the dir holding
  `.ddd/`; `--root` overrides.
- **Partial/broken builds:** SARIF from a failed build is ingested as
  emitted evidence; no SARIF file → that source is absent and `diff` says
  which source covered each rule.
- **Generated code:** governed like any code unless excluded via
  `config.yaml` `ignore` globs.
- **Renames/moves:** classified as remove + add (two events); no rename
  tracking.
- **Multi-file edits:** N single-file interceptions; no cross-file
  atomicity (stated in §7).
- **Multi-language edits:** language resolved per file (extension or
  explicit arg); one adapter per call.
- **Concurrent MCP clients:** one serving process assumed; per-file
  writes atomic; cross-process store locking unspecified — concurrent
  servers may interleave event sequence numbers.
- **YAML write locking / atomic replacement:** temp + rename per file via
  `fileops::write_file_atomic`.
- **Crash recovery:** atomic per-file writes mean a crash leaves the
  store at the last completed write; no journal.
- **Git worktrees:** work — discovery is filesystem-based; the store is
  per-worktree state like any checked-out file.
- **Symlinks / path traversal:** paths resolved relative to the root;
  no hardening against hostile symlinks — unspecified, treat governed
  repos as trusted (next item).
- **Untrusted repository content:** out of scope; the tool assumes the
  repo is the operator's. Do not point it at hostile content.
- **MCP write authorization:** none beyond transport access — anyone who
  can call the tools can write the graph; the PR diff is the review gate.
- **Graph/SARIF size limits:** none set; large inputs degrade linearly —
  unspecified, fails slowly rather than loudly (named as such).
- **Stable machine-readable output / JSON for CI:** not yet shipped
  (§6); finding text is stable per version, ids are stable per store.
- **Stable diagnostic/finding IDs:** graph ids are stable identities
  (duplicate = violation); diff findings key on `namespace/rule_id`.
- **Suppression expiry:** not implemented — a suppression cites a
  risk-acceptance record but nothing expires it; filed follow-up
  (roadmap).
- **Adoption baselines:** not implemented — `init` marks everything
  `UNGOVERNED`, which floods a large existing repo; filed follow-up
  (roadmap).
- **Deterministic rendering:** `render` output is a pure function of the
  store (sorted loads, no timestamps).
- **Sensitive metadata / source paths:** event rows and declarations
  carry repo-relative paths and symbol names only; nothing leaves the
  repo.
- **Language-server absence/failure/restart:** lazy start, health check,
  restart on crash; absence or failure is an explicit error (invariant 9),
  never an empty result.

---

## 9. Success criteria (measurable)

Replacing volume-shaped criteria with checkable ones:

1. **Classifier recall** on a hand-labelled contract-change corpus
   (the named future instrument — see the research protocol): ≥ agreed
   threshold per language, measured, not asserted.
2. **False-demand rate** on non-contract changes from the same corpus,
   below an agreed ceiling.
3. **Bypass-catch:** an out-of-band contract-surface change is caught by
   repository/CI validation (M8's acceptance; until then this criterion
   is honestly unmeetable and says so).
4. **Determinism:** `validate`, `diff`, `render` byte-identical across
   two runs on a fixed store and version (CI-checkable today).
5. **Declaration quality:** share of declarations judged meaningful on
   review (sampled), not declaration count.
6. **Basis-loss detection:** at least one real basis-loss event detected
   and acted on (already exercised: the M5/M7 re-affirmations).

Failure signals stay pre-registered: interception switched `off` within a
week of dogfood (friction claim falsified); `why` unused by agents;
manifest diff producing only noise.

---

## 10. Open questions carried

- Final name / command prefix (umbrella-level).
- Bicep seam-entry granularity: per-param vs. per-module-contract.
- `--json` schema for CI commands (lands with M8's CI gate).
- Multi-root workspace selection.

Settled former questions live in [`ddd-adrs.md`](ddd-adrs.md).
