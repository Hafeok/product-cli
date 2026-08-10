# PRD — Decision Ledger

**Working name:** `ledger` (binary TBD — see Open Decision OD-1)
**Status:** projected. Derived, not exercised. Falsifiers stated in §13.
**Audience:** an implementer or a fresh session with no prior context. §1–3 are the minimum canon needed to work from this document cold.
**Document stack:** DAD (`way-of-working-decision-allocated-delivery.md`) is the process; the `ddd` tool (`ddd-cli-prd.md`) is the governance mechanism; this document is canonical for the decision **record substrate**. The ddd tool is a producer and consumer of ledger entries (its M8 is the integration); this PRD's own milestones (§11, prefixed L0–L5 to disambiguate from the ddd tool's M-series) run as their own track and L0 is independently useful.

---

## 1. Problem

Software has no depositional record.

Archaeology works because physical processes leave stratigraphy whether or not anyone intended to record anything — spoil layers, tool marks, post-holes. Software overwrites in place. Git records what changed, not what was weighed. A commit that *settled* twenty decisions is byte-identical to one that *defaulted* them.

This matters because a chosen value and an unchosen value produce identical artifacts. They diverge only when something goes wrong: one is a decision you can revisit, the other is archaeology. Every other property of a system can be recovered by inspection given enough effort. The deliberation record cannot be recovered at any effort, because there was never anything to find.

So it must be deposited deliberately or not at all.

Three prior attempts and why they failed:

| Attempt | Deposits deliberately | Fails on |
|---|---|---|
| Commit messages | yes | prose not entities; commit-granular not decision-granular; no criterion form; no coverage denominator |
| ADRs | yes | granularity (architecture only); no discharge; no coverage |
| Prompt/memory files (`CLAUDE.md`, rules) | accidentally | append-only with no supersession, no provenance, no criterion form, no tolerance scoping |

Each got the location right (in the repo, reviewed with the code) and the structure wrong.

---

## 2. Conceptual foundation

Compressed canon. Sufficient to implement against; not the full framework.

### 2.1 The principle

For a task at a declared assurance level, **specification demand is constant** — fixed by the task, never by the system or the actor. It is fully allocated across four stores:

| Store | Form | When | Decided by |
|---|---|---|---|
| `constraint` | constraint form | before the act | extra-actor (compiler, analyzer, policy) |
| `criterion` | criterion form | after the act | extra-actor (test, assertion, telemetry) |
| `judgment` | per-run | during the act | a **named** accountable actor |
| `escaped` | — | — | **nobody** |

Total never shrinks. Demand is denominated in **governing decisions**. `escaped` is the only forbidden state — but *priced* escape (exposure stated, acceptor named, review date set) is a legitimate allocation. Silent escape is not.

### 2.2 Assurance level

Sets the **granularity bound**. A decision is *governing* iff varying it moves the outcome past tolerance. Assurance level is a reading of consequence, not a dial — declaring a lower tier does not make a task cheaper, it makes the declaration false, and every downstream gate still passes because gates are relative to the declared tier.

**This is the one place in the system with no mechanical check.** See §13.

### 2.3 Ground

The read-only surface an actor inspects in order to act. A fact is one element of ground. Ground is substrate, not demand: facts are inspected, not decided.

Existing code **is** ground — but only relative to an actor that did not produce it (the closure principle: an actor's own prior output is not ground for that actor). Code is ground with respect to *behaviour* and silent with respect to *decisions*: it never records whether a site was a decision at all, what alternatives were rejected, or what tolerance it was governing at.

### 2.4 Coverage, not pass-rate

A green board over an unenumerated set means nothing. The denominator is the governing decision set. **Coverage is a claim about absence relative to a declared set** — which is precisely the operation git cannot perform, and the reason this tool cannot be git with different object types.

---

## 3. What this tool is

**A version-control system for decisions**, with git's interaction and trust model, a semantic entity store underneath, and one primitive git does not have (coverage).

### 3.1 It is

- Append-only, attributed, content-addressed record of governing decisions
- Local-first, offline-capable, branchable, mergeable
- The source for four standing instruments (§10)
- A CI gate: readiness (nothing unallocated) and completeness (everything disposed)
- Federated across repos, because seam decisions belong to no single repo

### 3.2 It is not

- A test runner or verifier — it *records* discharge, it does not perform it
- A project tracker — decisions are not tasks and have no assignee or status board
- An authoring tool — it does not propose decisions; models and humans do, through it
- A wiki or documentation system — every entry is machine-checkable or it does not belong

### 3.3 Design stance taken from git

Content-addressed immutable objects · local-first with cheap branching · explicit staging between "noticed" and "accepted" · `blame` as a first-class primitive · distributed replication.

### 3.4 Design stance explicitly rejected from git

| Git does | Ledger does instead | Why |
|---|---|---|
| Commits snapshot the whole tree | Per-entity versioning, grouped by change-set | Decisions have independent lifecycles; one act touches 4 of 200 |
| Diff by line | Semantic diff (added, reallocated, superseded, escape-converted, acceptance-expired) | Textual diff on YAML is noise |
| Three-way text merge | Decision-level conflict requiring a named acceptor | Text-merging allocations produces plausible garbage — the worst failure mode for this artifact |
| Rewritable history (rebase, squash, force-push) | Append-only with explicit revocation entries | An acceptance a rebase can destroy is not an acceptance |
| Single merge base (a commit) | Merge base **per decision** | Two branches can share no change-set ancestry and still merge cleanly on 198 of 200 decisions |

---

## 4. Data model

### 4.1 Identity

**A decision has identity independent of any repository.** This is settled: seam decisions cross repos, and the highest-consequence decisions frequently have no natural repo home.

```
dec:<namespace>/<ulid>
e.g. dec:acme.billing/01J9Z4KQ7M8XTBVR3N2P6WCDHF
```

- **ULID** (or UUIDv7): offline-generatable, lexicographically sortable by creation time, no coordination required.
- **Namespace**: an owning scope, not a repo path. Repos may reference decisions in namespaces they do not own.
- The ID is **stable forever**. Supersession creates a new decision with a `supersedes` edge; it never mutates or reuses an ID.

### 4.2 Entities

```
Decision           stable id, namespace, created-at, created-by
DecisionVersion    content-hash; statement, allocation, discharge ref,
                   tolerance-override (optional, up-only), exposure (if escaped),
                   parent version
ChangeSet          attributed, timestamped, append-only; groups N version
                   transitions; references parent change-set(s)
Acceptance         (decision-id, version-hash, actor, timestamp, scope, expires-at)
Revocation         explicit reversal of a prior Acceptance, with reason
DecisionSet        a declared scope: task/feature, tolerance floor, ground state,
                   owner, membership
DischargeRef       typed pointer: analyzer:ID | test:FQN | policy:ID |
                   whatif:assertion | otel:metric+expectation | actor:identity
Instrument         a recorded reading (§10)
```

### 4.2.1 Tolerance: floor and up-only override (OD-4, resolved)

The set's tolerance is a **floor**, not a default. A decision's **effective tier** is the set
floor unless the version carries a `tolerance-override`, which is valid **only above the floor**
— a below-floor override is a schema violation, rejected at write, not flagged at review. There
is no representable state in which a decision sits below its set's regime: silent tier-shopping
is not caught, it is *unexpressible* (the predicates-carry-no-status move, applied to
governance). Up-overrides need no guard — over-classifying upward costs the author ceremony,
which is self-punishing — and are logged, not gated.

Consequences, priced:

- The **effective tier is part of the hashed content** (via floor-at-version-creation +
  override), so acceptance binds to the tier and a tier change is a new version requiring
  re-acceptance.
- **Raising a set's floor is a set-level governed act** that re-opens acceptance on every member
  whose effective tier falls below the new floor. Stranded entries are re-accepted at the new
  floor, never grandfathered.
- The escape pressure relocates to **set design**: the residual gaming move is filing a
  consequential decision into a low-floor set and not overriding up — far more visible than a
  silent enum (set membership is *where the entry lives*, reviewable at acceptance), and
  instrumented: **override-rate per set** (§10) — a set whose entries constantly override upward
  has its floor drawn wrong, a finding about the decomposition, not the author.

### 4.3 The version hash

`DecisionVersion` content is canonicalised and hashed. **Acceptance signs the hash, never the id.** This is the property that makes acceptance mean something: it names an exact state, not "whatever DEC-004 currently says." Any edit to an accepted decision invalidates the acceptance and requires re-acceptance by a named actor.

### 4.4 Constraints enforced by the store

- Exactly one `allocation` per decision-version; redundancy across stores is permitted but one is primary
- `tolerance-override` below the set floor is rejected at write (schema, not review); effective
  tier participates in the content hash
- Raising a set floor re-opens acceptance on members stranded below it; `verify` fails on
  stranded-unreaccepted entries
- `escaped` requires `exposure`, `accepted-by`, `review-by`
- `criterion` requires a `DischargeRef` and a `discharge-stage`
- `judgment` requires a named `actor`
- `accepted-by` **must not** resolve to a model identity (§9.3)
- Expired acceptances are a verify failure, not a warning
- No mutation of any object after write; corrections are new versions, reversals are Revocations

---

## 5. Storage architecture

**Files in git are the log. The graph is a materialized view.**

```
.decisions/
  sets/<set-id>.yml            declared scope, tolerance, ground, owner
  log/<changeset-ulid>.yml     append-only; the source of truth
  index/                       gitignored; rebuildable cache
```

- The log is plain text, diffable, and travels with the code through normal review. This preserves the one property git supplies and ADRs-in-Confluence lose: **the ledger is amendable only in the same reviewed unit as the code it governs.** A ledger that can drift from the artifact is prose again.
- The graph (embedded RDF store; Oxigraph assumed) is derived and disposable — `ledger reindex` rebuilds it from the log with no loss. It exists solely to answer coverage and provenance queries, which are not expressible over a file tree.
- Acceptance provenance modelled with **PROV-O**; structural validity with **SHACL**.

**Test of correctness for this split:** deleting `index/` and rebuilding must produce a byte-identical graph. If it cannot, state has leaked into the cache.

---

## 6. CLI surface

```bash
# scope
ledger init
ledger declare --set FT-104 --tolerance-floor T2 --ground characterised --owner emil@

# authoring
ledger add --set FT-104 --statement "Monetary amounts use decimal, never double."
ledger import --set FT-104 --from proposals.yml    # model-proposed enumeration
ledger allocate <id> --store constraint --discharge analyzer:DEC001-no-float-money
ledger escape   <id> --exposure "..." --review-by 2026-09-01
ledger supersede <id> --by <new-id> --reason "..."

# staging and acceptance
ledger status                       # working set vs accepted; what awaits acceptance
ledger accept <id> --as emil@ --expires 2027-08-01
ledger accept --class analyzer:DEC001 --as emil@ --expires 2027-08-01   # precommitment
ledger revoke <acceptance-id> --reason "..."

# inspection
ledger blame <id>                   # who accepted, when, which version
ledger log --set FT-104
ledger diff <ref>..<ref>            # semantic, not textual
ledger show <id> --history

# the primitive git lacks
ledger coverage --set FT-104        # disposition coverage over the declared set
ledger coverage --stage prod        # discharge coverage by pipeline stage

# gates
ledger verify --gate readiness      # nothing unallocated; blocks produce
ledger verify --gate completeness   # everything disposed; blocks release
ledger verify --expired             # stale acceptances

# distribution
ledger merge <branch>
ledger fetch <remote-namespace>     # seam decisions from another repo
ledger instruments                  # the four numbers (§10)
```

---

## 7. Merge semantics

Merge is per-decision. For each decision present on either side, find the common ancestor version.

| Situation | Resolution |
|---|---|
| One side changed | Fast-forward that decision |
| Both sides identical | No-op |
| Both changed, same allocation, different discharge | Conflict — surfaced |
| Both changed, different allocation | Conflict — surfaced |
| One superseded, other discharged | Conflict — surfaced |
| One accepted, other edited | Conflict; acceptance invalidated by hash |
| Added on one side only | Merge in, unaccepted |

**A conflict is a question for a named acceptor, never an automatic resolution.** No `-X ours`. No auto-merge strategy. The tool presents both allocations, the ancestor, and requires an `accept` to resolve. This is the single most important behavioural constraint in the design: silently merging two different allocations of the same decision manufactures escape while displaying success.

---

## 8. Coverage

The operation git cannot perform, and the reason the graph store is not optional.

```
coverage(set S) = |{d ∈ S : disposition(d) ≠ ∅}| / |S|
```

Reported by allocation, by discharge stage, and by acceptance freshness. Also reported: decisions with a discharge stage *later* than their ground allows (waste — see the ground table in the companion pipeline document) and *earlier* than their ground allows (fiction).

**The honest limit, stated in every coverage report:** coverage is measured against the *enumerated* set. Nothing verifies the set itself. Enumeration completeness has no mechanical check; late-discovery rate (§10) is a lagging proxy. This is the load-bearing gap in the whole design and the report must not imply otherwise.

---

## 9. Integration

### 9.1 Toolchain (companion document covers .NET/Bicep in detail)

- **Roslyn analyzer `DEC001`** — escape sites (`catch (Exception)`, `!`, `#pragma warning disable`) must cite a `dec:` id; a build task validates the id exists and is not expired
- **CI gates** — `ledger verify --gate readiness` before produce; `--gate completeness` before release
- **`PublicAPI.Shipped.txt` and Verify `.approved.txt`** — existing attributed acceptance ledgers in .NET; import as discharge refs rather than duplicating
- **Production discharge** — decisions with `discharge-stage: prod` carry an OTel metric ref and a stated expectation; a scheduled reconcile joins observed against expected and opens findings

### 9.2 Git

Commit trailers carry change-set references so `git log` and `ledger log` can be correlated. **Git is the deposition and distribution mechanism, not the ledger.** No git-notes dependency (they do not propagate by default and are routinely lost).

### 9.3 Model actors

Models may propose enumerations, allocations, and discharge procedures — this is recognition-over-recall on an artifact, the low-transfer-floor direction, and the right task shape for a model.

**Acceptance cannot be delegated.** The residual must land on an actor pinnable finely enough for the assurance level. Route acceptance to a model and the composite becomes pinnable only distributionally, at which point outcome-accountability is *unavailable*, not merely absent. The store enforces this: `accepted-by` must resolve to a human identity. Model-proposed entries land in the staging area and require `ledger accept` by a named human.

Precommitment at class level (`ledger accept --class`) is how human load falls without accountability leaving. Delegation is not.

### 9.4 Upstream basis manifest (federation configuration)

Repos base their claims and decisions on other repos' canon. The corpus itself is the motivating
instance, five levels deep: `product-cli → product-framework → ai-foundation-development →
decision-driven-design → actor-general determination` — the Stable Dependency Principle at repo
scale: dependencies point from volatile toward stable, acyclic by construction.

A repo declares its bases in a local **upstreams manifest** (`.decisions/upstreams.yaml`):

```yaml
upstreams:
  - repo: https://github.com/Hafeok/decision-driven-design
    pin: <commit SHA>          # content-addressed; the basis. Never a branch.
    ref: main                  # readable, advisory only
    provides: [DDD-, pred/, dec:ddd.]   # namespace prefixes; no two upstreams may overlap
```

Rules:

- **Pin by SHA.** "Based on X@main" is a basis that mutates under you; a SHA is a basis. Same
  law as acceptance-signs-hash, one level up.
- **Transitive resolution with a lockfile.** Resolving a repo walks its upstreams' own manifests
  upward and flattens into `upstreams.lock`. When two paths pin *different* SHAs of the same
  upstream — the diamond — that is a **basis conflict**: surfaced as a finding for a named human,
  never auto-resolved. §7's merge stance, applied between repos.
- **Basis-cone loading, not repo loading.** From any entry, walk `based_on`/`depends_on` edges
  upward through the pinned SHAs and load only the entries reached. "Everything at a given
  level" is that walk rendered; five levels never means five full graphs.
- **Offline-first.** A content-addressed clone cache keyed by SHA; no network at verify time;
  explicit `ledger upstreams refresh` is the only fetch. The pinned corpus is reproducible on any
  machine — the byte-identical rebuild test extends across repos.
- **Cross-repo drift is basis loss at repo granularity.** Pinned SHA ≠ current head *and* an
  entry you base on changed status or content in between → a `report escapes` finding, computed
  by the same hash comparison as in-repo pins. One law, three granularities: claim pins,
  decision pins, repo pins.
- **Publishing is pinning's dual.** Promotion to a shared catalog is publishing a repo that
  downstreams pin — no catalog service. This resolves the ddd PRD's `shared/` open note.

### 9.5 Acceptance workbench (the ledger's client)

The principal is the arrangement's scarcest actor and currently has its worst interface: a linear
chat transcript — proposer-optimised, session-bound, no overview, no queue, no diff. The
workbench corrects the allocation. It is a **client of the ledger, never a second store**: every
mutation it performs — accept (sign the content hash), edit-then-accept, reject with reason,
defer-with-price, promote status, set cadence — is an append through §4's discipline. It differs
in kind from `ddd render` (a read-only projection artifact): the workbench is interactive, and
its every write is a ledger entry.

Surfaces, in priority order:

1. **Queues** — staged entries awaiting acceptance, batch diffs against prior versions, expiring
   acceptances, cadence-due revalidations, basis-loss and upstream-drift findings, standing
   priced escapes. Acceptance work is queue work; the tree is context.
2. **The cross-repo tree** — claims status-coloured, decisions with `based_on` edges and pin
   health, supersession chains, rendered as the basis-cone walk over §9.4's pinned upstreams,
   rooted where the principal is working.
3. **LLM as staff, never as the centre** — verify a boundary clause against its cited ground,
   summarise a batch, draft rejection wording, answer "what breaks if I retire this" by graph
   query, propose cadences. The inversion of a coding session: the human is the primary actor;
   models are clerks. Acceptance remains non-delegable per §9.3 regardless of what the staff
   drafted.

Delivery: local-first web app extending the workspace's existing served-graph pattern
(`product mcp --http` + SSE) over multi-repo roots. **Interim, available now:** the git-native
staging discipline makes a PR review surface a v0 workbench — per-entry files, per-line comments
as rejection reasons, merge as acceptance. Crude but queue-shaped; curation sessions run on it
until L5 ships.

---

## 10. Instruments

Five numbers. Without them the tool is ceremony.

| Instrument | Definition | Reads on |
|---|---|---|
| **Late-discovery rate** | Decisions added to a set *after* its readiness gate, by stage | Enumeration quality. The headline number. Should fall with practice on a task type. |
| **Escape-conversion rate** | Priced escapes that proved governing and moved to `constraint`/`criterion` | Whether the ledger is alive or an accreting footgun list |
| **Analyzer-vs-human catch ratio** | Escape classes caught mechanically vs. in review | Descent measure of the encode–exercise loop |
| **Actor-substitution delta** | Swap the model, re-run, measure degradation | Demand riding on a model prior rather than sitting in a store. Periodic audit; the only instrument that catches this failure mode. |
| **Override-rate per set** | Share of a set's members carrying an up-only tolerance override (§4.2.1) | Whether the set's floor is drawn where consequence actually lives. A high rate is a finding about the decomposition, not the authors. |

---

## 11. Milestones

Prefixed **L0–L5** — the ddd tool's track keeps M1–M8 (its kickoff prompts shipped under those
names); unprefixed "M*n*" in any cross-document reference means the ddd track.

**L0 — Format and gate.** File schema, canonicalisation, hashing, `verify`. No graph, no merge. Deliverable: a CI gate that fails on unallocated decisions, incomplete escapes, and expired acceptances. *This alone is usable and carries a large share of the value.*

**L1 — Local operations.** `add`, `allocate`, `escape`, `accept`, `revoke`, `status`, `log`, `blame`, semantic `diff`.

**L2 — Graph and coverage.** Embedded RDF index, PROV-O acceptance provenance, SHACL validity, `coverage`, `reindex` with the byte-identical rebuild test.

**L3 — Merge.** Per-decision merge base, conflict classes from §7, no auto-resolution.

**L4 — Federation and instruments.** The upstreams manifest and lockfile (§9.4): SHA-pinned
transitive resolution, basis-cone loading, diamond surfacing, cross-repo drift in `report`
findings; cross-namespace fetch for seam decisions; `instruments`; toolchain integrations
(analyzer, OTel reconcile).

**L5 — Acceptance workbench.** The §9.5 client: queues, cross-repo tree over the basis cone,
LLM-as-staff, every mutation a ledger append. Signing UX per OD-3. Until it ships, curation runs
on the PR-review interim.

---

## 12. Open decisions

Named, not filled. Each should be settled before the milestone that depends on it.

- **OD-1 — Relationship to `decision-cli` (`dec`).** `decision-cli` is the execution-side orchestration harness. This tool is a record-side store. They share a domain and possibly a binary. Options: subcommand of `dec`, sibling binary sharing a Rust crate, or fully separate. Blocks naming and L0 packaging. *Leaning, not settled: sibling workspace member sharing crates, consistent with the ddd tool's delivery decision — requires the `dec` context to confirm.*
- **OD-2 — Relationship to `product-cli` / `.product/`. RESOLVED 2026-08-04.** The ledger is a workspace member of the `product-cli` repo, its RDF materialized view built on the same `product-core` graph infrastructure the ddd tool reuses (Oxigraph-class store, SHACL, PROV-O). `.decisions/` log remains the source of truth; the index stays rebuildable — the byte-identical rebuild test is unchanged. `.product/` is neither replaced nor consumed: it keeps its own ontology. The ddd tool's `.ddd/decisions/` is the **bootstrap form** and migrates to `dec:` ids at ddd M8; ddd manifest entries, seam declarations, and risk acceptances become ledger entries (their checkers are already `DischargeRef` types: `analyzer:ID`, `whatif:assertion`, `otel:metric+expectation`). The ddd basis-pin (claim status + `changed` at decision time) is superseded by version-hash references — basis loss becomes pinned-hash ≠ current-hash, mechanically exact. Basis: the sibling-on-`product-core` decision and the store-philosophy match (source-of-truth files, derived graph), both settled in the ddd PRD.
- **OD-3 — Signing.** Is `accepted-by` a claimed identity (git-config style, trust-on-review) or cryptographically signed (GPG/sigstore)? Trust model differs sharply for regulated tolerance tiers. Blocks L1. *The workbench (L5) raises the stakes: it is where signing UX lives, so OD-3's answer now shapes two milestones.*
- **OD-4 — Tolerance granularity. RESOLVED 2026-08-10 (principal: Emil).** Per-set **floor** with
  per-decision **up-only override** (§4.2.1). Below-floor states are unrepresentable rather than
  policed; effective tier is hashed content; floor-raising re-opens stranded acceptances;
  override-rate-per-set is the drawn-wrong-floor instrument. Supersedes the original per-set vs
  per-decision framing: the answer is both, ordered. The org product's OD-4 (step-up mapping)
  inherits: shopping out of step-up auth is likewise unrepresentable.
- **OD-5 — Seam decision ownership.** When a decision crosses two namespaces, who accepts? Both? A designated owner? Blocks L4.
- **OD-6 — Expiry default.** Is `expires-at` mandatory, defaulted, or optional? Mandatory forces re-acceptance churn; optional lets acceptances go stale silently.
- **OD-7 — Tolerance change protocol.** Changing a set's tier invalidates every escape acceptance made against the old consequence. Is that automatic invalidation or a prompted review? Also: what triggers a mandatory re-declaration (first payment, first PII, first external consumer, first SLA)?

---

## 13. Falsifiers

How we would know this is wrong, rather than merely unadopted.

- **Late-discovery rate does not fall** with practice on a repeated task type → enumeration is not a learnable skill at this granularity, or the tolerance tiers are mis-specified. Core to the whole design.
- **Escape-conversion rate stays near zero** while late-discovery stays flat → the ledger is an accreting footgun list, not a converging loop. Predicted specifically for teams with strong containment infrastructure, where cheap escape removes the pressure that produced enumeration.
- **Coverage correlates with nothing** — no relationship between disposition coverage and incident rate at matched tolerance → the denominator is wrong.
- **Merge conflicts are rare and boring** → decisions are not actually contested across branches, and per-decision merge base was over-engineering. Git's model would have sufficed.
- **Acceptance load does not fall** as precommitment classes accumulate → the asymptote (acceptance per novel decision class, not per run) is unreachable and the human bottleneck is structural rather than granular.
- **Ledger drifts from code** despite living in the same reviewed unit → co-location is insufficient and the binding must be mechanical (every code change touching a cited decision requires a ledger touch).
- **Diamond conflicts are frequent and noisy** across a small corpus under one principal → SHA-pinning at repo granularity is too coarse for how basis actually flows, and pinning belongs at entry granularity instead.
- **Acceptance throughput does not beat the PR-review interim** once the workbench ships → the bottleneck was never the interface, and the queue/tree design misread where principal demand concentrates.

---

## 14. One-paragraph summary for a cold session

Build a version-control system for governing decisions. Decisions have repo-independent stable IDs and content-hashed versions; acceptance signs a hash and is performed only by a named human. Files in git are the append-only log; an embedded RDF graph is a rebuildable materialized view whose sole purpose is answering coverage — "which decisions in this declared set have no disposition" — a claim about absence that git cannot express. Take git's local-first, content-addressed, branchable, blame-able interaction model; reject tree-snapshot commits, textual diff, textual merge, and rewritable history. Merge is per-decision with a per-decision merge base, and conflicting allocations are always a question for a named acceptor, never an automatic resolution. Ship L0 first — schema plus a CI gate that fails on unallocated decisions and expired acceptances — because it is usable alone. The known load-bearing gap: coverage is measured against the enumerated set, and nothing verifies the set.
