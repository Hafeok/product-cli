# Product (Next) — Founding Architecture

**A What-first, multi-repo tool for building software under The Product Framework.**

*Working draft 0.1. This document defines the architecture of the rebuild; it is the spine the rest of the project hangs off. It assumes [`product-framework-spec.md`](./product-framework-spec.md) and reuses its vocabulary (What / How / Delivery, the derivation contract, the conformance levels).*

---

## 0. Why a rebuild

`product-cli` is a strong instantiation of the framework's *spine* — one derived graph, deterministic verification gates, computed progress, topological build order. But it is **How-first**: features carry behaviour as prose, and the authored, enforced artifacts are ADRs and patterns. The framework is **What-first**: meaning is authored and agreed *before* realisation (§2). Retrofitting a domain + event model onto a How-centric tool fights the grain.

This rebuild inverts the order — **author the What, then realise it through the How** — and bakes in one structural decision the original could not retrofit: **the What/How split is also a repository boundary.**

We keep the *engine* and rebuild the *model*. See [§9](#9-keep-vs-rebuild).

---

## 1. The asymmetry principle

The What/How split (framework §2) is not only organisational; it is **topological**. The two halves pull in opposite directions:

- **The What is centripetal — it must live together.** Cross-context mappings (§3.1) and events that `change` domain entities (§3.2) are edges *inside* the What graph. Shard the What and those edges become dangling cross-repo references; the "one graph" property (§2) collapses. The What is the shared vocabulary of the whole system — its schema registry.

- **The How is centrifugal — it spreads out.** Each archetype (a *reusable How*, §4) has its own runtime contract (§4.2), stack, and often team. Forcing N archetypes into one repo couples independent realisations for no benefit. An operating system is just one very large How realising a large What.

> **The architecture in one line.** One What graph, pinned by N How repos. This is not a compromise — it is what §2's split *is* once made physical.

```
                        ┌───────────────────────────┐
                        │        WHAT  (1 repo)       │
                        │  domain model + event model │
                        │  bounded contexts + mappings│
                        │  features / releases        │   ← Delivery partitions
                        │  versioned by content hash  │
                        └─────────────┬───────────────┘
              pins snapshot @hash     │      pins snapshot @hash
        ┌───────────────┬────────────┴───────────┬───────────────┐
        ▼               ▼                         ▼               ▼
 ┌────────────┐  ┌────────────┐            ┌────────────┐  ┌────────────┐
 │ HOW repo A │  │ HOW repo B │    ...     │ HOW repo C │  │ HOW repo D │
 │ archetype  │  │ archetype  │            │ archetype  │  │ archetype  │
 │ + contracts│  │ + contracts│            │ + contracts│  │ + contracts│
 │ + verifs   │  │ + verifs   │            │ + verifs   │  │ + verifs   │
 └────────────┘  └────────────┘            └────────────┘  └────────────┘
```

---

## 2. Repository topology

| Repo kind | Count | Owns | Owned by |
|---|---|---|---|
| **What** | 1 (logical) | Domain model, event model, bounded contexts + mappings, the Delivery partition (features, releases) | Product & design |
| **How** | N | An archetype (reusable How) or a concrete realisation: decisions/principles/patterns, contracts, the code, the verifications | Engineering |

**Single-repo is the degenerate case.** A product with one archetype may keep What and How in one repo as a `what/` + `how/` split. The identity model ([§3](#3-identity-global-from-line-one)) is designed so this is *the same model* with one namespace and the What inlined — a product graduates from one repo to many without re-identifying anything.

**Bounded contexts are folders within the What repo, not separate repos.** This keeps cross-context mappings as in-graph edges. The resolver is therefore `1 What ↔ N How`, not `M What-contexts ↔ N How`. (Revisit only if a single What graph provably cannot scale — an explicit, later decision.)

---

## 3. Identity — global from line one

Everything multi-repo hinges on identity, and it is the sharpest break from `product-cli` (which uses repo-local string IDs like `FT-001`).

- **Every node is a URI.** RDF stops being an *export* (as in today's `rdf.rs`) and becomes the **native identity layer** — which the framework already mandates (§9).
- **Each repo declares a base namespace.** e.g. the What repo is `https://acme.example/what#`, archetype A is `https://acme.example/how/a#`.
- **Cross-repo references are URIs against a pinned snapshot.** A How artifact references `what:Order` where `what:` resolves to the What repo's namespace **at a pinned version** ([§4](#4-the-pinlock-contract)).
- **Local ergonomics preserved.** Authors still type short names; the tool expands them to URIs against the declared namespaces. URIs are the storage/resolution form, not the typing form.

---

## 4. The pin/lock contract

The framework's reproducibility chain (§1) and frozen-input rule (§5), plus §4.2's "once chosen, frozen," all point the same way: **a How repo depends on a content-hashed snapshot of the What, like a lockfile.**

- **`what.lock`** in each How repo records: the What repo's identity, the pinned **content hash** of the What graph, and the resolved URIs the repo actually references.
- **Bumping the pin is explicit** and triggers **cross-repo impact analysis**: "which How repos pinned a concept that changed between hash A and hash B?"
- **This reuses machinery `product-cli` already has** — content-hash immutability (ADR-032) and the hash-chained request log — promoted from intra-repo to cross-repo. The "core we keep" maps directly onto the new boundary.

> Live (unpinned) resolution is **non-conformant** for accepted work: it breaks reproducibility. It may exist only as a dev-time "what would bumping do?" preview.

---

## 5. Delivery across the seam

Features and releases are **partitions of the What** (§7.1), so they live in the **What repo**. But the verdicts that make them "done" live in the **How repos**. Therefore:

- **`feature_done` is inherently cross-repo.** The What defines the slice (concepts + flows); the How repos that realise it supply the domain/behavioural/verification verdicts. The predicate gathers verdicts *across* repos for a feature *defined* in the What.
- **A realisation manifest is required.** The tool needs to know *which How repos realise a given What* (and which slice each covers). Candidate: a `realises` declaration in each How repo's metadata, indexed by the tool.
- **`release_done`'s closed-cut check** (§7.2 — "no included element depends on an excluded one") is a graph traversal over the What's dependency DAG, evaluated with verdicts collected from the realising How repos.

---

## 6. Verification across the boundary

The boundary *creates* the verification kinds that were absent in `product-cli`:

| Framework §6.2 kind | Where it lives in this architecture |
|---|---|
| Internal coherence | Within a How repo's work-unit output (as today) |
| Contract conformance | How repo: realised code vs. its own contracts |
| **Seam** | **The pin itself**: the How repo's realisation conforms to the What snapshot it pinned — verified at the boundary |
| **Domain conformance** | How repo code vs. the **pinned** What domain model — now a concrete thing to check against |
| **Behavioural conformance** | How repo behaviour vs. the **pinned** What event model |

The redesign turns the framework's hardest-to-implement clauses from "absent" into "the boundary you already have to cross." The What snapshot is the fixed reference these checks were missing.

---

## 7. The What metamodel (first build target)

Authored first, before any How. The artifact kinds and their typed edges:

**Structure (domain model, §3.1)**
- `BoundedContext` — a namespace within the What; terms have one meaning inside it.
- `Entity` (has identity) / `ValueObject` (compared by value) / `Aggregate`.
- `Relation` — carries cardinality **and** rationale.
- `Invariant` — a machine-checkable constraint (SHACL or equivalent shape).
- `ContextMapping` — explicit cross-context correspondence (never assumed).

**Behaviour (event model, §3.2)**
- `Event` —`changes` an `Entity`.
- `Command` — `targets` an `Aggregate`.
- `ReadModel` — `projects` `Entity`(s).
- `Flow` / `InterfaceStep` — the timeline a non-engineer signs.

**Delivery (partition, §7)**
- `Feature` — a subgraph of the What (typically one `Flow` over its concepts).
- `Release` — a coherent set of features; carries the closed-cut requirement.

Every edge above is a derivation-contract link (§9): `changes`, `projects`, `targets`, plus `derived_from` / `conforms_to` for downstream How.

---

## 8. The How metamodel (second build target)

Realises the pinned What. Reuses the *strongest* part of `product-cli` (its How is already good):

- `Decision` — rationale, rejected alternatives, scope, when-it-applies/when-not (today's ADR, kept).
- `Principle` — stated checkably (new first-class kind; was implicit in ADRs).
- `Pattern` — implements a principle; `applies`/`realizes` edges (today's PAT, kept).
- `ApplicationContract` / `RuntimeContract` — the realisation surface (§4.2), with the seam between them verified.
- `InterfaceContract` — **generated from** the pinned domain model where a standard exists (OpenAPI/AsyncAPI/etc., §4.3).
- `WorkUnit` — references What + How by pointer (now cross-repo URIs), emits a **rationale trace** whose claims survive only if an enforcing verification passes (§5).

---

## 9. Keep vs. rebuild

**Keep — this is "the core" (≈ `product-core` minus the artifact slices):**
- Derived-graph-from-front-matter (ADR-003) — now assembled from *local repo + pinned What snapshot*.
- Atomic write + hash-chained request log → basis for What versioning.
- The verify *pipeline shape*: deterministic gates, never short-circuit, 0/1/2 exit contract.
- RDF/oxigraph — promoted from export to native identity.
- Slice+adapter discipline, no-`unwrap`, file-length/SRP fitness gates.

**Rebuild:**
- Taxonomy: `FT/ADR/TC/PAT/DEP` → **What** kinds ([§7](#7-the-what-metamodel-first-build-target)) then **How** kinds ([§8](#8-the-how-metamodel-second-build-target)).
- Local string IDs → URIs ([§3](#3-identity-global-from-line-one)).
- Constraint validation: hand-coded Rust → declarative shapes (SHACL or equivalent) for invariants.

**Net-new — the riskiest piece:**
- **The cross-repo resolver**: URI resolution, the `what.lock` pin format, cross-repo impact, and cross-repo verdict collection for `feature_done` / `release_done`. Everything else is reshaping what exists.

---

## 10. Open decisions

1. **What-internal scaling** — bounded contexts as folders in one What repo (recommended) vs. eventual context-repos. Sets whether the resolver is `1↔N` or `M↔N`. *Recommend: folders; revisit only on proven scale limits.*
2. **Shape language** — SHACL (framework reference) vs. a lighter embedded constraint DSL. SHACL buys ecosystem tooling; a DSL buys ergonomics.
3. **Realisation manifest format** — where a How repo declares what it `realises`, and how the tool indexes the 1↔N mapping.
4. **Pin transport** — vendored snapshot file vs. git submodule/subtree vs. a What registry. All satisfy the content-hash contract; they differ in workflow.

---

## 11. Build order

Meaning before mechanism, exactly as the framework prescribes:

1. **This spec** — the spine. ✅ (you are here)
2. **The What metamodel** ([§7](#7-the-what-metamodel-first-build-target)) — kinds, edges, shapes. The schema everything references.
3. **The resolver spike** ([§9](#9-keep-vs-rebuild)) — URI + `what.lock` + cross-repo reference, the riskiest new piece, proven early.
4. **The How metamodel** ([§8](#8-the-how-metamodel-second-build-target)) — realisation against a pinned What.
5. **Cross-repo verification & delivery** ([§5](#5-delivery-across-the-seam), [§6](#6-verification-across-the-boundary)) — `feature_done` / `release_done` over collected verdicts.
