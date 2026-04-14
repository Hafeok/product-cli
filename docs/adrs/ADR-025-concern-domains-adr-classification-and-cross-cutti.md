---
id: ADR-025
title: Concern Domains — ADR Classification and Cross-Cutting Scope
status: accepted
features: []
supersedes: []
superseded-by: []
domains: []
scope: domain
content-hash: sha256:6fd182d1ff66a1e1d964ffbfb4d0e496cc34e52dd4c518f94427e0217f5f2c31
---

**Status:** Accepted

**Context:** At scale (100+ ADRs), the graph has a discovery problem. ADRs are nodes with edges, but they carry no information about what kind of concern they govern. An ADR about security and an ADR about storage are structurally identical — the only way to find all security ADRs is to already know which ones they are. A new feature author in a large repository has no systematic way to ask "have I considered all security implications?" because "security" is not a first-class concept in the graph.

Two categories of ADR emerge at scale that are currently invisible:

**Cross-cutting ADRs** apply to every feature regardless of graph links. ADR-013 (error model) governs how every component surfaces errors. ADR-015 (file write safety) governs every mutation. These ADRs are never "done being relevant" — they apply to every new feature, always. Currently they only appear in a feature's context bundle if the author remembers to link them.

**Domain ADRs** govern a concern area (security, storage, IAM) but apply only to features that touch that area. A feature that introduces a new storage mechanism should consider all storage ADRs. Currently there is no way to identify that set without reading every ADR manually.

**Decision:** Add a `domains` field and a `scope` field to ADR front-matter. Domains are a controlled vocabulary declared in `product.toml`. `scope: cross-cutting` marks ADRs that must be acknowledged by every feature. `scope: domain` marks ADRs that must be acknowledged by any feature touching a declared domain. Feature front-matter gains a `domains-acknowledged` block for explicit reasoning when a domain applies but no ADR link is added.

---

### Domain Vocabulary

Domains are declared in `product.toml`. Each domain has a name and a one-sentence description:

```toml
[domains]
security        = "Authentication, authorisation, secrets, trust boundaries"
storage         = "Persistence, durability, volume, block devices, backup"
consensus       = "Raft, leader election, log replication, cluster membership"
networking      = "mDNS, mTLS, DNS, service discovery, port allocation"
error-handling  = "Error model, diagnostics, exit codes, panics, recovery"
observability   = "OTel, metrics, tracing, logging, telemetry"
iam             = "Identity, OIDC, tokens, RBAC, workload identity"
scheduling      = "Workload placement, resource limits, eviction, CPU/memory"
api             = "CLI surface, MCP tools, event schema, resource language"
data-model      = "RDF, SPARQL, ontology, event sourcing, projections"
```

The vocabulary is project-specific and evolves as the project grows. Domains are not a universal taxonomy — they reflect the concern areas that matter for this specific system.

---

### ADR Front-Matter Extension

```yaml
---
id: ADR-013
title: Error Model and User-Facing Error Format
status: accepted
features: [FT-001, FT-002]
domains: [error-handling, developer-experience]
scope: cross-cutting    # cross-cutting | domain | feature-specific
---
```

**Scope values:**

| Value | Meaning | Pre-flight behaviour |
|---|---|---|
| `cross-cutting` | Applies to every feature without exception | Must be linked or acknowledged by every new feature |
| `domain` | Applies to any feature touching the declared domains | Must be linked or acknowledged if the feature declares any matching domain |
| `feature-specific` | Governs a narrow, specific area | No automatic pre-flight requirement |

`feature-specific` is the default when `scope` is absent — preserving backward compatibility with all existing ADRs.

---

### Feature Front-Matter Extension

```yaml
---
id: FT-009
title: Rate Limiting
phase: 2
status: planned
depends-on: [FT-004]
domains: [networking, api]          # domains this feature touches
adrs: [ADR-004, ADR-009, ADR-012]
tests: [TC-041, TC-042]
domains-acknowledged:
  security: >
    Rate limiting operates at the Resource API layer. IAM enforces
    access upstream. No new trust boundaries introduced.
  iam: >
    No new identity primitives. Rate limit state is per-resource,
    not per-identity. Existing RBAC roles are unchanged.
  storage: >
    Token bucket state is in-memory only. No persistence required.
    Intentional — limits reset on restart.
---
```

`domains-acknowledged` entries close domain gaps without requiring a linked ADR. The reasoning is mandatory — an acknowledgement without a reason is a validation error (E011). The reasoning is included in the feature's context bundle so the implementation agent understands the deliberate scope exclusions.

---

### Validation Rules

`product graph check` gains two new checks:

**E011 — Acknowledgement without reasoning:** a `domains-acknowledged` entry exists but the value is empty or whitespace-only.

**W010 — Unacknowledged cross-cutting ADR:** a cross-cutting ADR exists and is neither linked to nor acknowledged by a feature. Reported as a warning per-feature: "FT-009 has not acknowledged ADR-013 (cross-cutting, error-handling)."

**W011 — Domain gap without acknowledgement:** a feature declares a domain (via `domains`) that has domain-scoped ADRs, but the feature neither links those ADRs nor acknowledges the domain.

W010 and W011 are warnings, not errors. During active development phases, a feature author may not have completed domain review. The warnings surface the gaps without blocking CI.

---

### Cross-Cutting ADR Resolution in Context Bundles

When assembling a context bundle for a feature, cross-cutting ADRs are always included regardless of explicit graph links. They are included at a fixed position: after the feature content, before the domain ADRs, before the feature-specific ADRs.

Bundle order:
1. Feature content
2. Cross-cutting ADRs (all, ordered by betweenness centrality)
3. Domain ADRs for the feature's declared domains (top-2 by centrality per domain)
4. Feature-linked ADRs (direct links, by centrality)
5. Test criteria

This ensures the implementation agent sees the governance layer (cross-cutting) before the architectural context (domain and feature-specific).

---

**Rationale:**
- The domain taxonomy is the index that makes large graphs navigable. Without it, finding "all security ADRs" requires reading every ADR. With it, `signal graph check --domain security` returns them instantly.
- `scope: cross-cutting` is the mechanism for ADRs that must never be forgotten. Instead of relying on every feature author to remember to link ADR-013, the system enforces it automatically. The author is free to say "I've considered this and it doesn't apply" — but they cannot silently skip it.
- Mandatory reasoning in `domains-acknowledged` is the critical design. An acknowledgement without reasoning is indistinguishable from a checkbox that was ticked to silence the warning. The reasoning proves intent. It also becomes valuable documentation — future authors reading the feature can see why security was explicitly scoped out.
- Limiting domain ADRs in bundles to top-2 by centrality (not all domain ADRs) is the key to avoiding context explosion. In a domain with 15 ADRs, the top-2 by centrality are the most foundational — the ones that govern the others. Reading them first is sufficient for the agent to understand the domain's constraints.

**Rejected alternatives:**
- **Tag-based classification** — tags with no vocabulary control. No scope distinction (cross-cutting vs domain). No acknowledgement mechanism. Rejected.
- **Mandatory ADR linking for all domains** — requires a linked ADR for every domain the feature touches, even if no existing ADR is relevant. Creates pressure to create unnecessary ADRs. Rejected.
- **All domain ADRs in context bundle** — a feature touching storage would receive all 15 storage ADRs. Context explosion at exactly the scale this ADR is designed to avoid. Rejected.
- **Centrality as the only filter** — use centrality ranking without domain taxonomy. Cannot answer "which ADRs are about security." Centrality tells you what's important, not what topic it's about. Both are needed. Rejected.