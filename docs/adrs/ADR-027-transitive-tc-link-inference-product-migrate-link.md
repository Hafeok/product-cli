---
id: ADR-027
title: Transitive TC Link Inference — `product migrate link-tests` and `product graph infer`
status: accepted
features: []
supersedes: []
superseded-by: []
domains: []
scope: domain
---

**Status:** Accepted

**Context:** After migrating a monolithic PRD and ADR document, TC files have `validates.adrs` populated (the parent ADR they were extracted from) but `validates.features` is empty. Feature files have `adrs` populated (after manual review by the developer) but their linked test criteria are unknown. The graph has the data needed to infer the missing TC→Feature edges: if `FT-001 → ADR-002` and `TC-002 → ADR-002`, then `TC-002 → FT-001` follows mechanically.

This inference is not performed during initial migration (ADR-017) because feature→ADR links require human review — they cannot be reliably heuristic-inferred from prose. Once the developer has confirmed the feature→ADR links, the transitive TC links are fully mechanical and safe to infer automatically.

The same inference also applies after any manual `product feature link FT-XXX --adr ADR-XXX` command: new TC→Feature links may follow from the new edge that were not present before.

**Decision:** Implement two inference commands. `product migrate link-tests` is the primary post-migration step. `product graph infer` is the general-purpose command that runs the same inference at any time. Both use the same algorithm, the same dry-run output format, and the same atomic write safety. Both skip cross-cutting ADRs. Both are idempotent and additive — they never remove existing links.

---

### Algorithm

```
For each ADR A where A.scope ≠ cross-cutting:
    F_set = { F | A ∈ F.adrs }                  // features governed by A
    T_set = { T | A ∈ T.validates.adrs }         // TCs that validate A
    for each T ∈ T_set, F ∈ F_set:
        if F ∉ T.validates.features:
            T.validates.features += [F]           // new transitive link
            emit: "TC-%s → FT-%s via ADR-%s"
```

The cross-cutting exclusion is the critical design decision. ADR-001 (Rust) and ADR-013 (error model) are linked to every feature. If their TCs were auto-linked to every feature, the resulting links would be semantically meaningless — a test that validates every feature validates none of them specifically. Cross-cutting ADRs exist to govern platform-wide concerns; their tests are similarly platform-wide. They do not belong in individual feature validation lists.

The correct graph state after inference: every TC that validates a domain-scoped or feature-specific ADR gains links to every feature that uses that ADR. Cross-cutting TC links remain empty — they are validated by `product graph check` separately as a platform-wide concern.

---

### `product migrate link-tests`

Post-migration entry point. Intended to run once, after `product migrate from-prd` and `product migrate from-adrs`, after the developer has manually confirmed feature→ADR links.

```
product migrate link-tests              # infer and apply all transitive TC links
product migrate link-tests --dry-run    # show what would change, write nothing
product migrate link-tests --adr ADR-002  # scope to one ADR's TCs only
```

Dry-run output:

```
Transitive TC link inference (dry run)
────────────────────────────────────────────────────────────
ADR-002 — openraft for Cluster Consensus  [scope: domain]
  TC-002 Raft Leader Election    → +FT-001, +FT-005   (2 new)
  TC-003 Raft Leader Failover    → +FT-001, +FT-005   (2 new)
  TC-004 Raft Learner Join       → +FT-001             (1 new)

ADR-006 — Oxigraph for RDF Projection  [scope: domain]
  TC-008 SPARQL Basic Query      → +FT-001, +FT-003   (2 new)
  TC-009 Graph Projection        → +FT-003             (1 new)

ADR-001 — Rust as Implementation Language  [scope: cross-cutting]
  → skipped (cross-cutting, would link to all features)

ADR-013 — Error Model  [scope: cross-cutting]
  → skipped (cross-cutting, would link to all features)
────────────────────────────────────────────────────────────
8 new TC→Feature links across 5 TCs and 2 ADRs
2 ADRs skipped (cross-cutting)
0 links already existed (idempotent)

Run without --dry-run to apply.
```

---

### `product graph infer`

General-purpose inference command. Runs the same algorithm as `link-tests` but is not migration-specific. Use after any manual feature→ADR link addition to pick up newly implied TC links.

```
product graph infer                      # infer all missing transitive TC links
product graph infer --dry-run
product graph infer --feature FT-009    # scope to one feature's new links
product graph infer --adr ADR-021       # scope to one ADR's TCs
```

`product graph infer` is idempotent — safe to run on any repository at any time with no risk of incorrect mutations. Existing links are never removed.

**Integration with `product feature link`:** when `product feature link FT-009 --adr ADR-021` is run, Product immediately computes the set of TCs that would be inferred and asks:

```
product feature link FT-009 --adr ADR-021

  Linked: FT-009 → ADR-021

  Transitive TC links inferred:
    TC-041 Rate Limit Under Load    → FT-009  (via ADR-021)
    TC-042 Token Bucket Refill      → FT-009  (via ADR-021)

  Add these TC links automatically? [Y/n]
```

If confirmed, the TC links are applied in the same atomic write batch as the ADR link. If declined, the developer can run `product graph infer --feature FT-009` later.

---

### Reverse Inference: ADR→Feature back-link

When `product migrate link-tests` or `product graph infer` adds `FT-001` to `TC-002.validates.features`, it also adds `TC-002` to `FT-001.tests` if not already present. This keeps the bidirectional front-matter consistent — the feature knows about its tests without requiring a separate step.

```
Before inference:
  FT-001.tests = [TC-001]        # only explicitly linked TC
  TC-002.validates.features = [] # empty after migration

After inference:
  FT-001.tests = [TC-001, TC-002]  # TC-002 added by reverse inference
  TC-002.validates.features = [FT-001, FT-005]
```

Both mutations are written atomically in the same operation. If the write fails mid-batch, neither is applied (ADR-015).

---

### Interaction with Cross-Cutting ADRs

Cross-cutting TCs are not linked to individual features. But they must still be validated. The validation model is: cross-cutting TCs appear in the context bundle for every feature (via ADR-025's cross-cutting bundle inclusion rule) and are run as part of the platform-level test suite, not as part of any individual feature's verify step.

`product verify FT-001` runs `FT-001.tests`. Cross-cutting TCs are not in that list. They are run by `product verify --platform` — a separate command that runs all TCs linked to cross-cutting ADRs regardless of feature association. This keeps feature-level verification fast and focused while ensuring platform-wide invariants are still exercised.

---

**Rationale:**
- Transitive inference is sound for domain-scoped ADRs. The relationship is: feature uses ADR → TC validates that ADR's constraints → TC validates the feature's use of those constraints. No human judgment is required; the relationship is mechanical.
- The cross-cutting exclusion is not optional. Without it, `product migrate link-tests` on a repository where every feature links to ADR-001 (Rust) would add every feature to every TC that validates ADR-001 — tens or hundreds of spurious links. These links would inflate W002 resolution numbers without adding analytical value.
- The interactive confirmation in `product feature link` is the right UX for ongoing development. During migration, the developer runs `link-tests` once as a batch. During active development, they want to know immediately what TC links follow from a new ADR link — not discover it on the next `graph check` run.
- Reverse inference (updating `FT.tests` when `TC.validates.features` is updated) maintains the invariant that the graph is consistent from both directions. Without it, `FT-001.tests` would be out of date until the developer manually ran `product feature link FT-001 --test TC-002`, which they would rarely remember to do.

**Rejected alternatives:**
- **Infer links during `migrate from-adrs` without confirmed feature→ADR links** — feature→ADR links are not reliably inferred by the migration heuristic. Inferring TC→Feature links on top of unconfirmed feature→ADR links propagates the heuristic error into TC front-matter. Two layers of approximation produce unreliable results. Rejected: inference must wait for confirmed links.
- **Include cross-cutting ADR TCs in feature test lists** — meaningful for platform integrity but makes `product verify FT-001` run the entire platform test suite for every feature. Feature verification becomes slow and its output noisy. `product verify --platform` is the right separation.
- **Manual TC linking only** — without `link-tests`, a repository with 30 features and 60 TCs requires 180 manual link operations after migration (each of 60 TCs linked to its 3 average features). This friction is prohibitive and ensures the links are never completed. Automation with dry-run review is the correct balance.
- **Bidirectional sync as a continuous background process** — a daemon watches for file changes and updates links automatically. Rejected: daemon lifecycle complexity (ADR-003 reasoning applies). The interactive confirmation in `product feature link` provides the same real-time benefit without a daemon.