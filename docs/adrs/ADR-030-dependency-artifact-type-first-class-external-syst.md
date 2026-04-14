---
id: ADR-030
title: Dependency Artifact Type — First-Class External System Declarations
status: accepted
features: []
supersedes: []
superseded-by: []
domains: []
scope: domain
---

**Status:** Accepted

**Context:** ADRs capture architectural decisions — why a dependency was chosen, what was rejected, and the rationale. They do not capture the runtime facts about a dependency: what version is required, what interface it exposes, whether it is currently available, and which features depend on it. These facts are different in kind from decision rationale and serve different consumers.

Four problems exist without a dependency artifact type:

1. **Preflight gaps** — `product preflight FT-007` checks domain coverage and spec gaps, but cannot check whether PostgreSQL 14 is running on port 5432. There is no structured place to declare that requirement.

2. **Missing graph edges** — `product impact DEP-001` cannot exist because "openraft" is not a graph node. When openraft releases a breaking change, there is no query that returns every affected feature. That impact is discoverable only by reading every ADR.

3. **Context bundle blind spots** — an agent implementing a feature that calls an external API needs the interface contract: auth mechanism, rate limits, error model. This information is not in decision rationale and is not currently in context bundles.

4. **No dependency bill-of-materials** — there is no command that returns every external dependency the product has, across all features, with versions and availability checks. This is valuable for security audits, upgrade planning, and onboarding.

ADRs remain the right home for the *decision* to use a dependency. `DEP-XXX` artifacts are the right home for the *runtime facts* about that dependency.

**Decision:** Add `Dependency` (`DEP-XXX`) as a first-class artifact type. Dependencies declare their type, version constraint, interface description, and an optional availability check command. They are linked to features via a `uses` edge and to ADRs via a `governs` edge. Preflight, TC prerequisites, context bundles, impact analysis, and gap analysis all integrate with the new type.

---

### Dependency Types

| Type | Meaning | Availability check pattern |
|---|---|---|
| `library` | Build-time code dependency (crate, npm, NuGet, Maven) | Usually none — version managed by package manifest |
| `service` | Runtime service that must be running (database, queue, cache, message broker) | TCP check, health endpoint, CLI ping |
| `api` | External HTTP or gRPC API | HTTP health check, auth validation |
| `tool` | CLI tool required at runtime or in CI | `which tool && tool --version` |
| `hardware` | Physical hardware requirement | `uname`, device node presence check |
| `runtime` | Execution environment (OS version, SDK, JVM) | Version check command |

---

### Front-Matter Schema

**Library dependency:**

```yaml
---
id: DEP-001
title: openraft
type: library
source: crates.io
version: ">=0.9,<1.0"
status: active           # active | deprecated | evaluating | migrating
features: [FT-001, FT-002, FT-005]
adrs: [ADR-002]          # decision that governs use of this dependency
availability-check: ~    # null — library, no runtime check
breaking-change-risk: medium   # low | medium | high
---

This crate provides Raft consensus with pluggable storage and network layers.
It is used for leader election, log replication, and cluster membership.
```

**Service dependency:**

```yaml
---
id: DEP-005
title: PostgreSQL Event Store
type: service
version: ">=14"
status: active
features: [FT-007, FT-012]
adrs: [ADR-015]
interface:
  protocol: tcp
  port: 5432
  auth: md5
  connection-string-env: DATABASE_URL
  health-endpoint: ~
availability-check: "pg_isready -h ${PG_HOST:-localhost} -p ${PG_PORT:-5432}"
breaking-change-risk: low
---

PostgreSQL is used as the backing store for the event log in development and
test environments. Production uses the embedded storage layer (DEP-002).
```

**API dependency:**

```yaml
---
id: DEP-007
title: GitHub Container Registry
type: api
version: "v2"
status: active
features: [FT-018]
adrs: [ADR-022]
interface:
  base-url: https://ghcr.io
  auth: bearer-token
  auth-env: GHCR_TOKEN
  rate-limit: 5000/hour
  error-model: OCI Distribution Spec v1.1
availability-check: >
  curl -sf -H "Authorization: Bearer ${GHCR_TOKEN}"
  https://ghcr.io/v2/ > /dev/null
breaking-change-risk: low
---
```

**Hardware dependency:**

```yaml
---
id: DEP-010
title: Raspberry Pi 5 — NVMe Storage
type: hardware
version: ~
status: active
features: [FT-001, FT-004]
adrs: [ADR-001]
interface:
  arch: aarch64
  storage-min-gb: 500
  storage-device-pattern: /dev/nvme*
availability-check: >
  uname -m | grep -q aarch64
  && ls /dev/nvme* 2>/dev/null | head -1 | grep -q nvme
breaking-change-risk: low
---
```

---

### Dependency Statuses

| Status | Meaning |
|---|---|
| `active` | In use, maintained, current version in use |
| `evaluating` | Under consideration — not yet committed |
| `deprecated` | Scheduled for removal, features migrating away |
| `migrating` | Active migration in progress to a successor dependency |

When `status: deprecated` or `migrating`, `product graph check` emits W013 ("feature uses a deprecated dependency") for every feature still linked to it.

---

### New Graph Edges

| Edge | From | To | Description |
|---|---|---|---|
| `uses` | Feature | Dependency | Feature requires this dependency at runtime |
| `governs` | ADR | Dependency | Decision that chose this dependency |
| `supersedes` | Dependency | Dependency | This dependency replaces another (migration) |

The reverse of every edge is traversable. `product impact DEP-001` uses reverse-graph BFS to find every feature and ADR that would be affected by a breaking change in openraft.

---

### Integration: Preflight

`product preflight FT-XXX` is extended to check dependency availability. For each DEP linked to the feature where `availability-check` is non-null, Product executes the check command. The same semantics as `[verify.prerequisites]` — exit 0 = satisfied, non-zero = not satisfied. Product never installs or starts dependencies; it only checks them.

```
product preflight FT-007

━━━ Dependency Availability ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  DEP-001  openraft        [library — no check]    ✓
  DEP-005  PostgreSQL 14+  [pg_isready ...]         ✗ not running
  DEP-002  embedded store  [library — no check]    ✓

━━━ Cross-Cutting ADRs ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  ...
```

Dependency availability failures are warnings (exit 2), not errors (exit 1) — the agent can still implement the feature without the dependency running; it just cannot run tests that require it. The TC `requires` mechanism (ADR-021) handles the runtime gate separately.

---

### Integration: TC `requires` Field

TCs can reference DEP IDs directly in `requires`:

```yaml
---
id: TC-042
title: Event Store Persistence
type: scenario
requires: [DEP-005]          # resolves to DEP-005.availability-check automatically
runner: bash
runner-args: ["scripts/test-harness/event-store.sh"]
---
```

Product resolves `DEP-005` to its `availability-check` command. No need to duplicate the check command in `[verify.prerequisites]` — the dependency declaration is the single source of truth. Named prerequisite strings in `[verify.prerequisites]` still work for non-dependency checks.

---

### Integration: Context Bundles

A "Dependencies" section is inserted into context bundles after ADRs and before test criteria:

```markdown
## Dependencies

### DEP-001 — openraft [library, >=0.9,<1.0]

[dependency body text, front-matter stripped]

Interface: no runtime interface (build-time library)
Availability: no check required

### DEP-005 — PostgreSQL Event Store [service, >=14]

[dependency body text, front-matter stripped]

Interface:
  protocol: tcp / port: 5432
  auth: md5 / env: DATABASE_URL
  availability-check: pg_isready -h ${PG_HOST:-localhost} -p 5432
```

An agent implementing a feature receives the complete interface contract for every external dependency. It knows what environment variables to read, what ports to connect to, what auth mechanism to use, and what check to run to verify the dependency is present.

---

### Integration: Impact Analysis

`product impact DEP-001` performs reverse-graph BFS from the dependency node:

```
product impact DEP-001

Impact analysis: DEP-001 — openraft

Direct dependents:
  Features:  FT-001 (in-progress), FT-002 (complete), FT-005 (planned)
  ADRs:      ADR-002 (governs)

Transitive dependents (via feature dependencies):
  Features:  FT-007 (planned) — depends-on FT-001

Breaking change risk: medium
Summary: 4 features, 1 ADR. 1 feature already complete may need revisiting.
```

---

### Integration: Gap Analysis — G008

New gap code for gap analysis (ADR-019):

| Code | Severity | Description |
|---|---|---|
| G008 | medium | Feature uses a dependency (`uses` edge to DEP) with no ADR governing its use (`governs` edge from any ADR to that DEP) |

This enforces the principle that every external dependency choice is documented as an architectural decision. A feature that adds a new `uses` edge to a DEP without a corresponding ADR is a specification gap.

---

### New Commands

```
product dep list                      # all dependencies with status
product dep list --type service       # filter by type
product dep list --status deprecated  # find deprecated deps
product dep show DEP-001              # full dependency detail
product dep features DEP-001          # which features use this dependency
product dep check DEP-005             # run availability check manually
product dep check --all               # run all availability checks
product dep bom                       # full dependency bill of materials
product dep bom --format json         # machine-readable for security audits
```

`product dep bom` produces a structured bill of materials across all features:

```
Dependency Bill of Materials — product v0.1

Libraries (build-time):
  DEP-001  openraft          >=0.9,<1.0    crates.io   active
  DEP-003  oxigraph          >=0.4         crates.io   active
  DEP-004  clap              >=4.0         crates.io   active

Services (runtime):
  DEP-005  PostgreSQL        >=14          —           active (dev/test only)

Hardware:
  DEP-010  Raspberry Pi 5    —             —           active

Total: 5 dependencies across 3 types
Breaking change risk: 1 medium (DEP-001), 4 low
```

---

### New Validation Codes

| Code | Tier | Description |
|---|---|---|
| E013 | Dependency | Dependency has no linked ADR — every dependency requires a governing decision |
| W013 | Validation | Feature uses a deprecated or migrating dependency |
| W015 | Validation | Dependency `availability-check` failed during preflight |

E013 is a hard error (exit code 1). Every external dependency is an architectural choice — why this library over alternatives, what version constraint, what the tradeoffs are. That choice belongs in an ADR. A dependency without an ADR is an undocumented decision, which is the same class of problem as a broken link: the graph is structurally incomplete.

`product dep new "openraft" --type library` scaffolds both the `DEP-XXX` file and an `ADR-XXX` stub linked to it. The author is prompted to complete the ADR. Creating a DEP without creating or linking an ADR is a deliberate friction point — the tool makes the easy path the correct path.

G008 (LLM-detected undocumented dependency decisions) remains in gap analysis but is now a backstop for the rare case where a DEP has an ADR link that doesn't actually document the decision clearly. E013 handles the structural absence; G008 handles the semantic absence.

---

**Rationale:**
- Separating dependency facts from decision rationale keeps each artifact focused. An ADR that also contains version constraints, interface specs, and availability check commands becomes a kitchen-sink document. `DEP-XXX` is the right unit for runtime facts.
- The `uses` edge from Feature to Dependency is the missing link in the graph. Without it, `product impact DEP-001` cannot exist. With it, a single command returns the full blast radius of any dependency change.
- Availability checks co-located with the dependency declaration are the single source of truth. The same check is used by preflight, by TC `requires` resolution, and by `product dep check`. No duplication, no drift.
- The six dependency types cover the actual range of external dependencies in real projects without over-engineering. `library` and `service` handle 80% of cases. `api`, `tool`, `hardware`, and `runtime` handle the rest.
- `breaking-change-risk` is a human-declared field, not computed. It communicates intent: "this dependency is stable and unlikely to break" vs. "this is a pre-1.0 library and we expect breaking changes." It informs prioritisation of upgrade work.

**Rejected alternatives:**
- **Dependencies modelled only as ADRs** — the current state. ADRs capture decisions, not runtime facts. A developer reading ADR-002 knows why openraft was chosen; they do not know what environment variable the connection string lives in, what port the service binds to, or what command to run to check it. The information requirements are different.
- **Dependencies as front-matter fields on features** — `external-deps: [openraft>=0.9, postgres>=14]`. Duplicates across features sharing the same dependency. No graph node to query. No `product impact` possible. Rejected.
- **Using `[verify.prerequisites]` for all dependency checks** — `[verify.prerequisites]` is a project-level dictionary of named shell commands. It is not typed, versioned, or linked to features. It cannot be queried. It doesn't produce a bill of materials. Rejected as insufficient at scale.
- **Separate dependency management tool** — a `product-deps` companion binary. Rejected: the dependency graph and the artifact graph are the same graph. Separating them into different tools creates the synchronisation problem that the unified graph is designed to avoid.