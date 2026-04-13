# Product — Product Requirements Document

> **Status:** Draft
> **Version:** 0.1
> **Companion:** See `product-adrs.md` for all architectural decisions

---

## 1. Vision

Product is a command-line tool and MCP server that manages the full lifecycle of software development from idea to implementation. It imposes a structured, file-based knowledge graph over a project's features, architectural decisions, and test criteria — and uses that graph to give LLM agents precisely the context they need, nothing more.

The core insight is that LLM-driven development fails not because the model is incapable, but because context is wrong. Too much of it, in the wrong shape, forces the model to reason about irrelevant material and miss the connections that matter. Product solves this by making relationships between artifacts machine-readable and queryable, so the right context bundle can be assembled for any task in a single command.

Product bridges the full loop: ideas become structured specifications through an LLM authoring session with full graph awareness; specifications become implementations through an LLM agent that receives exactly the right context bundle; implementations are verified against the test criteria defined in the spec. The filesystem is the persistent state shared between every agent in this loop. Git is the audit log.

Product runs as a CLI for local use, as a stdio MCP server for Claude Code on the desktop, and as an HTTP MCP server for remote access — including from a phone via claude.ai. The same tool surface, the same graph, the same authoring and implementation flows, regardless of where you are.

---

## 2. Goals

1. **Structured artifact decomposition** — features, architectural decisions, and test criteria each live in their own files. No monolithic documents.
2. **Machine-readable relationships** — YAML front-matter in every file declares its identity and its edges to other artifacts. The graph is derived from these declarations, not maintained separately.
3. **Precise LLM context assembly** — the `context` command bundles a feature with all linked ADRs and test criteria into a single, clean markdown document ready for an LLM agent.
4. **Implementation status tracking** — `checklist.md` is generated from feature front-matter. Status is declared in files, not maintained by hand in a separate document.
5. **Queryable knowledge graph** — the CLI can answer relational queries: which ADRs apply to this feature, which tests validate this decision, which features are in phase 1 and have no tests.
6. **RDF export** — the derived graph can be exported to `index.ttl` for SPARQL tooling, LLM injection, or external graph queries.
7. **Rust, single binary** — Product ships as a single compiled binary with no runtime dependencies. It can run in CI, on a developer laptop, or inside an agentic pipeline.
8. **Repository-native** — Product operates on a directory of markdown files. No server, no database, no configuration beyond a single `product.toml` at the repo root.
9. **MCP server — stdio and HTTP** — Product exposes its full tool surface as an MCP server. stdio transport supports Claude Code on the local machine. HTTP transport supports remote access from any MCP-capable client, including claude.ai on mobile.
10. **Graph-aware authoring** — `product author` sessions give Claude full read and scaffold access to the graph during spec writing. The authoring agent cannot implement an idea without first understanding what already exists.
11. **Agent orchestration** — `product implement` assembles context, invokes the implementation agent, and hands back control to `product verify`. The full loop from feature to passing tests is a single command.
12. **Continuous specification health** — drift detection, fitness functions, pre-commit review, and gap analysis run continuously to catch specification degradation before it reaches the implementation agent.

---

## 3. Non-Goals

1. **Management web UI** — Product has no browser-based management interface. The CLI and MCP tool surface are the only interfaces. A web portal may be built on top of Product at a later stage.
2. **Remote state** — Product does not sync to any external service. The filesystem is the only state store. The HTTP MCP server exposes the local filesystem to remote clients; it does not replicate or store state externally.
3. **Issue tracker integration** — Product does not create GitHub Issues, Jira tickets, or Linear items. It is a knowledge management tool, not a project management platform.
4. **Code generation** — Product does not write implementation code. It assembles context for agents that do.
5. **Multi-user collaboration** — Product is designed for a single owner per repository. Concurrent access via the HTTP MCP server is serialised by advisory lock. Conflict resolution and multi-author workflows are handled by git, not by Product.
6. **Schema enforcement** — Product validates front-matter structure but does not enforce ontological constraints on the knowledge graph. It reports broken links; it does not prevent them.
7. **Plugin system** — Product has a fixed set of artifact types. Extensibility is out of scope for v1.
8. **Multi-repo workspaces** — Product operates on a single repository. A `product.toml` spans exactly one repo. Cross-repository knowledge graphs are not planned.

---

## 4. Core Concepts

### Artifact Types

**Feature (`FT-XXX`)** — A unit of product capability. Corresponds to a section of a PRD. Declares its phase, status, linked ADRs, and linked test criteria. A feature is the primary navigation unit of the knowledge graph: everything else is reachable from it.

**Architectural Decision Record (`ADR-XXX`)** — A single architectural decision. Declares context, decision, rationale, rejected alternatives, and the features it applies to. An ADR may apply to multiple features. An ADR may supersede or be superseded by another ADR.

**Test Criterion (`TC-XXX`)** — A single verifiable assertion about system behaviour. A test criterion has a type (scenario, invariant, chaos, exit-criteria), is linked to one or more features and one or more ADRs, and belongs to a phase. Test criteria are extracted from ADRs during migration — they are not co-located with the decisions they verify.

### Relationships

```
Feature ──── implementedBy ────► ADR
Feature ──── validatedBy ───────► TestCriterion
ADR     ──── testedBy ──────────► TestCriterion
ADR     ──── supersedes ────────► ADR
```

Edges are declared in the *source* artifact's front-matter. The derived graph is bidirectional — every edge is traversable in both directions by the CLI.

### The Derived Graph

Product reads all front-matter declarations on every command invocation and builds an in-memory graph. There is no persistent graph store. The graph is always consistent with the files. `product graph rebuild` writes `index.ttl` as a snapshot for external tooling, but this file is never read by Product itself.

### The Context Bundle

A context bundle is a single markdown document containing a feature, all its linked ADRs, and all its linked test criteria — assembled in a deterministic order and formatted for direct injection into an LLM context window. This is the primary output of Product. Everything else in the tool exists to make context bundles accurate and complete.

---

## 5. Repository Layout

```
/docs
  product.toml              ← repository config (name, prefix, phases)
  /features
    FT-001-cluster-foundation.md
    FT-002-products-iam.md
    FT-003-rdf-event-store.md
  /adrs
    ADR-001-rust-language.md
    ADR-002-openraft-consensus.md
  /tests
    TC-001-binary-compiles.md
    TC-002-raft-leader-election.md
    TC-003-raft-leader-failover.md
  /graph
    index.ttl               ← generated, never hand-edited
  checklist.md              ← generated, never hand-edited
```

Subdirectory names and file prefixes are configurable in `product.toml`. The layout above is the default.

---

## 6. Front-Matter Schema

### Feature

```yaml
---
id: FT-001
title: Cluster Foundation
phase: 1
status: in-progress          # planned | in-progress | complete | abandoned
depends-on: []               # feature IDs that must be complete before this one
domains: [consensus, networking, storage, iam, observability]
                             # concern domains this feature touches
adrs: [ADR-001, ADR-002, ADR-003, ADR-006]
tests: [TC-001, TC-002, TC-003, TC-004]
domains-acknowledged:        # explicit reasoning for domains with no linked ADR
  scheduling: >
    No workload scheduling in phase 1. Cluster foundation does not
    place containers — that is phase 2. Intentionally out of scope.
---
```

The `depends-on` field declares implementation dependencies between features. Product validates that these edges form a DAG — cycles are a hard error. `product feature next` uses topological sort over this DAG to determine the correct implementation order, replacing the previous phase-label ordering.

### ADR

```yaml
---
id: ADR-002
title: openraft for Cluster Consensus
status: accepted             # proposed | accepted | superseded | abandoned
features: [FT-001]
supersedes: []
superseded-by: []
domains: [consensus, networking]   # concern domains this ADR governs
scope: domain               # cross-cutting | domain | feature-specific (default)
source-files:                # optional: source files that implement this decision
  - src/consensus/raft.rs    # used by `product drift check` for precise analysis
  - src/consensus/leader.rs  # if absent, Product uses pattern-based discovery
---
```

### Test Criterion

Test criterion files use a hybrid format. The YAML front-matter carries graph metadata. The file body contains a prose description followed by optional AISP-influenced formal blocks (see ADR-011).

**Types and formal block requirements:**

| Type | Description | Formal blocks |
|---|---|---|
| `scenario` | Given/when/then integration test | Optional (`⟦Λ:Scenario⟧`) |
| `invariant` | Property that must hold for all valid inputs | Mandatory (`⟦Γ:Invariants⟧`) |
| `chaos` | System behaviour under fault injection | Mandatory (`⟦Γ:Invariants⟧`) |
| `exit-criteria` | Measurable threshold for phase completion | Optional (`⟦Λ:ExitCriteria⟧`) |
| `benchmark` | Quality measurement producing a score over time | Mandatory (`⟦Λ:Benchmark⟧`) |

The `benchmark` type is distinct from the others: it does not produce a binary pass/fail result. It produces a score in [0.0, 1.0] tracked over releases. A benchmark test criterion references an external task directory and rubric file rather than expressing an inline assertion.

**Scenario example:**
```markdown
---
id: TC-002
title: Raft Leader Election
type: scenario
status: unimplemented        # unimplemented | implemented | passing | failing
validates:
  features: [FT-001]
  adrs: [ADR-002]
phase: 1
runner: cargo-test           # cargo-test | bash | pytest | custom
                             # omit if test infrastructure not yet available
runner-args: ["--test", "raft_leader_election", "--", "--nocapture"]
runner-timeout: 60s          # optional, default 30s
---

## Description

Bootstrap a two-node cluster. Assert that exactly one node is elected leader
within 10 seconds, and that the leader identity is reflected in the RDF graph.

## Formal Specification

⟦Σ:Types⟧{
  Node≜IRI
  Role≜Leader|Follower|Learner
  ClusterState≜⟨nodes:Node+, roles:Node→Role⟩
}

⟦Λ:Scenario⟧{
  given≜cluster_init(nodes:2)
  when≜elapsed(10s)
  then≜∃n∈nodes: roles(n)=Leader
       ∧ graph_contains(n, picloud:hasRole, picloud:Leader)
}

⟦Ε⟧⟨δ≜0.95;φ≜100;τ≜◊⁺⟩
```

**Invariant example:**
```markdown
---
id: TC-020
title: Betweenness Centrality Always In Range
type: invariant
status: unimplemented
validates:
  features: [FT-001]
  adrs: [ADR-012]
phase: 3
---

## Description

For any valid graph input, betweenness centrality scores must be in [0.0, 1.0].
Verified by proptest over generated connected graphs.

## Formal Specification

⟦Σ:Types⟧{
  Graph≜⟨nodes:Node+, edges:Edge*⟩
  CentralityScore≜Float
}

⟦Γ:Invariants⟧{
  ∀g:Graph, ∀n∈g.nodes: betweenness(g,n) ≥ 0.0 ∧ betweenness(g,n) ≤ 1.0
}

⟦Ε⟧⟨δ≜0.99;φ≜100;τ≜◊⁺⟩
```

**Benchmark example:**
```markdown
---
id: TC-030
title: LLM Context Quality — Raft Leader Election
type: benchmark
status: unimplemented
validates:
  features: [FT-001]
  adrs: [ADR-006, ADR-012]
phase: 3
benchmark:
  task: benchmarks/tasks/task-001-raft-leader-election
  rubric: benchmarks/tasks/task-001-raft-leader-election/rubric.md
  conditions: [none, naive, product]
  runs-per-condition: 5
  pass-threshold:
    product: 0.80
    delta-vs-naive: 0.15
---

## Description

Measures the improvement in LLM implementation quality when using a Product
context bundle vs. no context and vs. naive full-document context.

## Formal Specification

⟦Λ:Benchmark⟧{
  baseline≜condition(none)
  target≜condition(product)
  scorer≜rubric_llm(temperature:0)
  pass≜score(product) ≥ 0.80 ∧ score(product) - score(naive) ≥ 0.15
}

⟦Ε⟧⟨δ≜0.85;φ≜80;τ≜◊?⟩
```

The evidence block fields are:
- `δ` — specification confidence (0.0–1.0)
- `φ` — coverage completeness (0–100%)
- `τ` — stability signal: `◊⁺` stable, `◊⁻` unstable, `◊?` unknown

### Repository Config (`product.toml`)

The complete canonical `product.toml`. All sections except `[paths]`, `[phases]`, and `[prefixes]` are optional and shown with their defaults.

```toml
name = "picloud"
schema-version = "1"

[paths]
features = "docs/features"
adrs = "docs/adrs"
tests = "docs/tests"
graph = "docs/graph"
checklist = "docs/checklist.md"
metrics = "metrics.jsonl"
gaps = "gaps.json"
drift = "drift.json"
prompts = "benchmarks/prompts"

[phases]
1 = "Cluster Foundation"
2 = "Products and IAM"
3 = "RDF and Event Store"
4 = "Operational Maturity"

[prefixes]
feature = "FT"
adr = "ADR"
test = "TC"

# Concern domain vocabulary — controlled by the project, not by Product
# Any domain declared in ADR or feature front-matter must appear here
[domains]
security        = "Authentication, authorisation, secrets, trust boundaries"
storage         = "Persistence, durability, volume, block devices, backup"
consensus       = "Raft, leader election, log replication, cluster membership"
networking      = "mDNS, mTLS, DNS, service discovery, port allocation"
error-handling  = "Error model, diagnostics, exit codes, panics, recovery"
observability   = "OTel, metrics, tracing, logging, telemetry"
iam             = "Identity, OIDC, tokens, RBAC, workload identity"
scheduling      = "Workload placement, resource limits, eviction"
api             = "CLI surface, MCP tools, event schema, resource language"
data-model      = "RDF, SPARQL, ontology, event sourcing, projections"

# MCP server settings (product mcp)
[mcp]
write = true                    # enable write tools over MCP
port = 7777                     # HTTP transport port
cors-origins = ["https://claude.ai"]
# token = ""                    # override with PRODUCT_MCP_TOKEN env var

# Agent invocation (product implement)
[agent]
default = "claude-code"         # claude-code | cursor | custom
auto-verify = true
gap-gate = "high"               # refuse to implement if gaps at this severity

[agent.claude-code]
flags = []

[agent.custom]
command = "./scripts/agent.sh {context_file} {feature_id}"

# Versioned system prompts for authoring sessions
[author]
feature-prompt-version = "1"
adr-prompt-version = "1"
review-prompt-version = "1"
agent = "claude-code"

# Versioned implementation prompt
[implementation-prompt]
version = "1"

# LLM gap analysis settings (product gap check)
[gap-analysis]
prompt-version = "1"
model = "claude-sonnet-4-6"
max-findings-per-adr = 10
severity-threshold = "medium"   # findings below this are informational only

# Drift detection settings (product drift check)
[drift]
source-roots = ["src/", "lib/"]
ignore = ["tests/", "benches/", "target/"]
max-files-per-adr = 20

# Architectural fitness thresholds (product metrics threshold)
[metrics]
record-on-merge = true          # automatically append to metrics.jsonl in CI

[metrics.thresholds]
spec_coverage           = { min = 0.90, severity = "error" }
test_coverage           = { min = 0.80, severity = "error" }
exit_criteria_coverage  = { min = 0.60, severity = "warning" }
phi                     = { min = 0.70, severity = "warning" }
gap_resolution_rate     = { min = 0.50, severity = "warning" }
drift_density           = { max = 0.20, severity = "warning" }
```

---

## 7. CLI Commands

### Navigation

```
product feature list [--phase N] [--status STATUS]
product feature show FT-001
product feature adrs FT-001          # all ADRs linked to this feature
product feature tests FT-001         # all test criteria for this feature
product feature deps FT-001          # full transitive dependency tree
product feature next                 # next feature by topological order (not phase label)

product adr list [--status STATUS]
product adr show ADR-002
product adr features ADR-002         # which features reference this ADR
product adr tests ADR-002            # which tests validate this decision

product test list [--phase N] [--type TYPE] [--status STATUS]
product test show TC-002
product test untested                # features with no linked tests
```

### Context Assembly

```
product context FT-001               # feature + direct ADRs + direct tests (depth 1)
product context FT-001 --depth 2     # transitive context: deps, shared ADRs, their tests
product context --phase 1            # all features in phase 1, with full context
product context --phase 1 --adrs-only  # phase 1 features + ADRs, no tests
product context ADR-002              # ADR + all linked features + all linked tests
product context --order id           # override default centrality ordering of ADRs
```

ADRs within a bundle are ordered by betweenness centrality descending by default — the most structurally important decisions appear first. Pass `--order id` for the previous ID-ascending behaviour.

### Status and Checklist

```
product status                       # summary: features by phase and status
product status --phase 1             # phase 1 detail with test coverage
product status --untested            # features with no linked test criteria
product status --failing             # features with one or more failing tests
product checklist generate           # regenerate checklist.md from feature files
```

### Graph Operations

```
product graph check                  # validate all links, DAG cycles, phase/dep mismatches
product graph rebuild                # regenerate index.ttl from all front-matter
product graph query "SELECT ..."     # SPARQL over the generated graph
product graph stats                  # artifact counts, link density, centrality summary,
                                     # φ (formal block coverage) across test criteria
product graph central                # top-10 ADRs by betweenness centrality
product graph central --top N        # configurable N
product graph central --all          # full ranked list
product graph coverage               # feature × domain coverage matrix
product graph coverage --domain security   # filter to one domain column
product graph coverage --format json       # machine-readable for CI
product impact ADR-002               # full affected set if this decision changes
product impact FT-001                # what depends on this feature completing
product impact TC-003                # what depends on this test criterion
```

`product graph check` also validates:
- No cycles in the `depends-on` feature DAG (exit code 1)
- Phase label / dependency order disagreements (exit code 2)
- Acknowledgements without reasoning — E011 (exit code 1)
- Domains declared in front-matter not present in `product.toml` vocabulary — E012 (exit code 1)

### Pre-flight and Domain Coverage

```
product preflight FT-001             # domain coverage check — run before authoring
product preflight FT-001 --format json

product feature acknowledge FT-009 --domain security \
  --reason "no trust boundaries introduced"
product feature acknowledge FT-009 --adr ADR-040 \
  --reason "standard output conventions apply"
```

Pre-flight must be clean before `product implement` proceeds (Step 0 in the pipeline). Pre-flight gaps are resolved by linking an ADR (`product feature link`) or acknowledging a domain/ADR with explicit reasoning. Acknowledgements without reasoning are E011 hard errors.

### Authoring

```
product feature new "Cluster Foundation"   # scaffold FT-XXX file with next ID
product adr new "Use openraft for consensus"
product test new "Raft leader election" --type scenario

product feature link FT-001 --adr ADR-002  # add edge (mutates front-matter)
product feature link FT-001 --test TC-002

product adr status ADR-002 accepted        # set ADR status
product test status TC-002 passing         # set test status
product feature status FT-001 complete     # set feature status
```

### Migration

```
product migrate from-prd PRD.md           # parse monolithic PRD → feature files
product migrate from-adrs ADRS.md         # parse monolithic ADR file → adr files + test files
product migrate validate                  # report what would be created without writing
```

### Gap Analysis

```
product gap check                         # analyse all ADRs for specification gaps
product gap check ADR-002                 # analyse a single ADR
product gap check --changed               # CI mode: only ADRs changed since HEAD~1
                                          # plus 1-hop graph neighbours
product gap check --format json           # structured JSON to stdout for CI annotation
product gap check --severity high         # filter to high-severity findings only

product gap report                        # human-readable gap summary across all ADRs
product gap stats                         # gap density by ADR, resolution rate over time

product gap suppress GAP-ADR002-G001-a3f9 --reason "deferred to phase 2"
product gap unsuppress GAP-ADR002-G001-a3f9
```

Gap findings go to stdout (they are results). Analysis errors (network failure, model error) go to stderr. Exit code 0 = no new gaps. Exit code 1 = new unsuppressed gaps found. Exit code 2 = analysis warnings (partial results, model errors on some ADRs).

### Drift Detection

```
product drift check ADR-002               # check one ADR against codebase
product drift check --changed             # only ADRs changed in current PR
product drift check --phase 1             # all phase-1 ADRs
product drift scan src/consensus/         # what ADRs govern this code?
product drift report                      # full drift report across all ADRs
product drift suppress DRIFT-ADR002-D001-a3f9 --reason "..."
```

### Metrics and Fitness Functions

```
product metrics record                    # snapshot current metrics to metrics.jsonl
product metrics trend                     # graph over last N snapshots
product metrics trend --metric phi        # single metric over time
product metrics threshold                 # check metrics against declared thresholds
product metrics stats                     # current values for all tracked metrics
```

### MCP Server

```
product mcp                               # stdio transport (default, for Claude Code)
product mcp --http                        # HTTP transport on default port 7777
product mcp --http --port 8080            # HTTP transport on custom port
product mcp --http --bind 0.0.0.0        # bind to all interfaces (remote access)
product mcp --http --token $SECRET        # bearer token auth (required for remote)
product mcp tools                         # list all available MCP tools
```

### Authoring Sessions

```
product author feature                    # graph-aware feature authoring session
product author adr                        # graph-aware ADR authoring session
product author review                     # spec gardening — find gaps and improve coverage

product install-hooks                     # install pre-commit hook in .git/hooks/
product adr review --staged               # review staged ADR files (used by pre-commit hook)
product adr review ADR-XXX                # review a specific ADR
```

### Agent Orchestration

```
product implement FT-001                  # gap-check → assemble context → invoke agent
product implement FT-001 --agent cursor   # override configured agent
product implement FT-001 --dry-run        # show what would be sent to agent, don't invoke
product verify FT-001                     # run linked TCs, update status, regenerate checklist
product verify FT-001 --tc TC-002         # run a single TC only
```

---

## 8. Context Bundle Format

The context command assembles a deterministic markdown bundle. Order is always: feature → ADRs (by ID ascending) → test criteria (by phase, then type: exit-criteria, scenario, invariant, chaos).

The bundle opens with an AISP-influenced formal header block (see ADR-011) that an agent can parse without reading the full document. It declares the bundle's identity, all linked artifact IDs, and aggregate evidence metrics derived from the test criteria evidence blocks.

```markdown
# Context Bundle: FT-001 — Cluster Foundation

⟦Ω:Bundle⟧{
  feature≜FT-001:Feature
  phase≜1:Phase
  status≜InProgress:FeatureStatus
  generated≜2026-04-11T09:00:00Z
  implementedBy≜⟨ADR-001,ADR-002,ADR-003,ADR-006⟩:Decision+
  validatedBy≜⟨TC-001,TC-002,TC-003,TC-004⟩:TestCriterion+
}
⟦Ε⟧⟨δ≜0.92;φ≜75;τ≜◊⁺⟩

---

## Feature: FT-001 — Cluster Foundation

[full content of FT-001-cluster-foundation.md, front-matter stripped]

---

## ADR-001 — Rust as Implementation Language

[full content of ADR-001-rust-language.md, front-matter stripped]

---

## ADR-002 — openraft for Cluster Consensus

[full content, front-matter stripped]

---

## Test Criteria

### TC-001 — Binary Compiles (exit-criteria)

[prose description]

⟦Λ:ExitCriteria⟧{
  binary_size < 20MB
  compile_time(rpi5, cold) < 5min
  ldd(binary) = {libc}
}
⟦Ε⟧⟨δ≜0.98;φ≜100;τ≜◊⁺⟩

### TC-002 — Raft Leader Election (scenario)

[prose description]

⟦Σ:Types⟧{ Node≜IRI; Role≜Leader|Follower|Learner }
⟦Γ:Invariants⟧{ ∀s:ClusterState: |{n | roles(n)=Leader}| = 1 }
⟦Λ:Scenario⟧{
  given≜cluster_init(nodes:2)
  when≜elapsed(10s)
  then≜∃n∈nodes: roles(n)=Leader ∧ graph_contains(n, picloud:hasRole, picloud:Leader)
}
⟦Ε⟧⟨δ≜0.95;φ≜100;τ≜◊⁺⟩
```

The bundle evidence block `⟦Ε⟧` at the top is computed as the mean of all linked test criterion `δ` values (confidence), and the percentage of criteria with formal blocks present (`φ`). An agent receiving this bundle can assess the specification quality before reading the full content.

YAML front-matter is stripped from all sections. Formal blocks in test criteria are preserved verbatim — they are the specification, not metadata.

---

## 9. Graph Model

Product builds an in-memory directed graph from front-matter on every invocation. The graph is also exportable as RDF Turtle via `product graph rebuild`.

### Edge Types

| Edge | From | To | Description |
|---|---|---|---|
| `implementedBy` | Feature | ADR | Feature is governed by this decision |
| `validatedBy` | Feature | TestCriterion | Feature is verified by this test |
| `testedBy` | ADR | TestCriterion | Decision is verified by this test |
| `supersedes` | ADR | ADR | This decision replaces another |
| `depends-on` | Feature | Feature | Implementation dependency — must complete before |

The reverse of every edge is implicit. Impact analysis (`product impact`) traverses the reverse graph to compute reachability.

### Graph Algorithms

| Algorithm | Applied to | Command | Purpose |
|---|---|---|---|
| Topological sort (Kahn's) | Feature `depends-on` DAG | `product feature next` | Correct implementation ordering |
| BFS to depth N | All edges | `product context --depth N` | Transitive context assembly |
| Betweenness centrality (Brandes') | ADR nodes | `product graph central` | Structural importance ranking |
| Reverse-graph BFS | All edges reversed | `product impact` | Change impact analysis |

### RDF Export

Product exports the knowledge graph as RDF Turtle. The ontology prefix is `pm:` (product-meta).

```turtle
@prefix pm: <https://product-meta/ontology#> .
@prefix ft: <https://product-meta/feature/> .
@prefix adr: <https://product-meta/adr/> .
@prefix tc: <https://product-meta/test/> .

ft:FT-001 a pm:Feature ;
    pm:title "Cluster Foundation" ;
    pm:phase 1 ;
    pm:status pm:InProgress ;
    pm:implementedBy adr:ADR-001 ;
    pm:implementedBy adr:ADR-002 ;
    pm:validatedBy tc:TC-001 ;
    pm:validatedBy tc:TC-002 .

ft:FT-003 a pm:Feature ;
    pm:dependsOn ft:FT-001 ;
    pm:dependsOn ft:FT-002 .

adr:ADR-002 a pm:ArchitecturalDecision ;
    pm:title "openraft for Cluster Consensus" ;
    pm:status pm:Accepted ;
    pm:betweennessCentrality 0.731 ;
    pm:appliesTo ft:FT-001 ;
    pm:testedBy tc:TC-002 .

tc:TC-002 a pm:TestCriterion ;
    pm:title "Raft Leader Election" ;
    pm:type pm:Scenario ;
    pm:status pm:Unimplemented ;
    pm:validates ft:FT-001 ;
    pm:validates adr:ADR-002 .
```

Betweenness centrality scores are written into the TTL export on `graph rebuild` so external SPARQL tools can query on them.

---

## 10. Generated Checklist

`checklist.md` is regenerated by `product checklist generate`. It is never hand-edited. Status is owned by feature front-matter.

```markdown
# Implementation Checklist

> Generated by product v0.1 | Last updated: 2026-04-11
> Do not edit directly — update status in feature/test front-matter and run `product checklist generate`

## Phase 1 — Cluster Foundation

### FT-001 — Cluster Foundation [in-progress]

- [x] ADR-001: Rust as implementation language (accepted)
- [x] ADR-002: openraft for consensus (accepted)
- [ ] TC-001: Binary compiles (exit-criteria) — unimplemented
- [ ] TC-002: Raft leader election (scenario) — unimplemented
- [ ] TC-003: Raft leader failover (chaos) — unimplemented
```

---

## 11. Validation and Graph Health

`product graph check` is the primary consistency tool. All output goes to stderr. Exit codes follow the three-tier scheme from ADR-009 and ADR-013.

Errors (exit code 1):

| Code | Condition |
|---|---|
| E002 | Broken link — referenced artifact does not exist |
| E003 | Dependency cycle in `depends-on` DAG |
| E004 | Supersession cycle in ADR `supersedes` chain |
| E001 | Malformed front-matter in any artifact file |
| E011 | `domains-acknowledged` entry present with empty reasoning |
| E012 | Domain declared in front-matter not present in `product.toml` vocabulary |

Warnings (exit code 2 when no errors):

| Code | Condition |
|---|---|
| W001 | Orphaned artifact — ADR or test with no incoming feature links |
| W002 | Feature has no linked test criteria |
| W003 | Feature has no test of type `exit-criteria` |
| W004 | Invariant or chaos test missing formal specification blocks |
| W005 | Phase label disagrees with topological dependency order |
| W006 | Evidence block `δ` below 0.7 (low-confidence specification) |
| W007 | Schema upgrade available |
| W008 | Migration: ADR status field not found, defaulted to `proposed` |
| W009 | Migration: no test subsection found in ADR, no TC files extracted |
| W010 | Cross-cutting ADR not linked or acknowledged by a feature |
| W011 | Feature declares a domain with existing domain-scoped ADRs but no coverage |

Schema errors (exit code 1):

| Code | Condition |
|---|---|
| E008 | `schema-version` in `product.toml` exceeds this binary's supported version |

Gap analysis codes (stdout, separate from `graph check`):

| Code | Severity | Condition |
|---|---|---|
| G001 | high | Testable claim in ADR body with no linked TC |
| G002 | high | Formal invariant block with no scenario or chaos TC |
| G003 | medium | ADR has no rejected alternatives section |
| G004 | medium | Rationale references undocumented external constraint |
| G005 | high | Logical contradiction between this ADR and a linked ADR |
| G006 | medium | Feature aspect not addressed by any linked ADR |
| G007 | low | Rationale references decisions superseded by a newer ADR |

All errors use the rustc-style diagnostic format (file path, line number, offending content, remediation hint). `--format json` outputs structured JSON to stderr for CI consumption. See ADR-013 for the full error model.

---

## 12. Domain Coverage Matrix

`product graph coverage` produces the feature × domain coverage matrix — the portfolio-level view of architectural completeness at scale.

```
product graph coverage

                    sec  stor  cons  net  obs  err  iam  sched  api  data
FT-001 Cluster       ✓    ✓     ✓    ✓    ✓    ✓    ✓    ✓     ✓    ✓
FT-002 Products      ✓    ✓     ·    ✓    ✓    ✓    ✓    ·     ✓    ·
FT-003 RDF Store     ~    ✓     ·    ·    ✓    ✓    ~    ·     ✓    ✓
FT-009 Rate Limit    ✗    ✗     ·    ✓    ✗    ✗    ✗    ·     ✓    ·

Legend:
  ✓  covered      — feature has a linked ADR in this domain
  ~  acknowledged — domain acknowledged with explicit reasoning, no linked ADR
  ·  not declared — feature does not declare this domain (may still apply)
  ✗  gap          — feature declares domain but has no coverage
```

`product preflight FT-XXX` produces the single-feature view of the same data, with specific ADRs named and resolution commands printed:

```
product preflight FT-009

━━━ Cross-Cutting ADRs (must acknowledge all) ━━━━━━━━━━━━━━

  ✓  ADR-001  Rust as implementation language          [linked]
  ✓  ADR-013  Error model and diagnostics              [linked]
  ✗  ADR-038  Observability requirements               [not acknowledged]

━━━ Domain Coverage ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  networking  ✓  ADR-004 (linked), ADR-006 (linked)
  security    ✗  no coverage — top-2 by centrality: ADR-011, ADR-019

━━━ To resolve ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  product feature link FT-009 --adr ADR-038
  product feature acknowledge FT-009 --domain security --reason "..."
```

Domain coverage is integrated into the `product implement` pipeline as Step 0 — pre-flight must be clean before context assembly or agent invocation. See ADR-026.

---

## 13. Migration Path

Migration is a two-phase extract-then-confirm process. See ADR-017 for full heuristic specification.

```bash
# Dry run — see what would be created
product migrate from-adrs picloud-adrs.md --validate
product migrate from-prd picloud-prd.md --validate

# Execute — write files, skip existing
product migrate from-adrs picloud-adrs.md --execute
product migrate from-prd picloud-prd.md --execute

# Interactive — review each artifact before writing
product migrate from-prd picloud-prd.md --interactive

# Post-migration: fill in link gaps and generate checklist
product graph check
product checklist generate
```

The migration parser uses heading structure to detect artifact boundaries and extracts phase references, status markers, and test criteria from subsections. It does not infer `depends-on` edges or feature→ADR links — those require human review and are filled in via `product feature link` commands after migration.

The source document is never modified. Migration can be re-run safely.

---

## 14. Phase Plan

### Phase 1 — Core Graph and Context

**Goal:** A developer can migrate an existing PRD and ADR file, navigate the graph, and assemble a context bundle. The binary is production-safe: no panics on user input, atomic file writes, schema versioning in place.

- [ ] `product.toml` parsing and repository discovery including `schema-version`
- [ ] Schema version validation on startup — E008 on forward incompatibility, W007 on upgrade available
- [ ] Front-matter parser for feature, ADR, and test files including `depends-on`
- [ ] Formal block parser (ADR-016) — `⟦Σ⟧`, `⟦Γ⟧`, `⟦Λ⟧`, `⟦Ε⟧` blocks with line-level error reporting
- [ ] In-memory graph construction from front-matter (all five edge types)
- [ ] DAG cycle detection on `depends-on` edges — E003 hard error
- [ ] Topological sort (Kahn's algorithm) for `product feature next`
- [ ] `product feature list/show/adrs/tests/deps`
- [ ] `product adr list/show/features/tests`
- [ ] `product test list/show`
- [ ] `product context FT-XXX` — context bundle with AISP `⟦Ω:Bundle⟧` header
- [ ] `product context --depth N` — BFS to depth N with deduplication
- [ ] `product graph check` — all error and warning codes, rustc-style diagnostics
- [ ] `product graph check --format json` — structured stderr for CI
- [ ] `product graph rebuild` — TTL export including `pm:dependsOn`, centrality placeholder
- [ ] Atomic file writes (temp + rename + fsync) for all mutations
- [ ] Advisory lock on `.product.lock` with stale lock detection
- [ ] Tmp file cleanup on startup
- [ ] `product migrate from-prd --validate/--execute/--interactive`
- [ ] `product migrate from-adrs --validate/--execute/--interactive`
- [ ] `#![deny(clippy::unwrap_used)]` — zero panics on user input
- [ ] Single binary, ARM64 + x86_64 + Apple Silicon

**Exit criteria:** Migrate the PiCloud PRD and ADRs. Run `product graph check --format json` — zero errors, JSON output is valid. Run `product feature next` — returns topologically correct next feature. Assemble `product context FT-001 --depth 2` — bundle opens with `⟦Ω:Bundle⟧` header. Feed 20 malformed YAML files — zero panics (TC-P001 passes). All IT-001–IT-11 integration tests pass. All TC-P001–TC-P004 property tests pass with 1000 cases each.

---

### Phase 2 — Authoring, Status and Impact

**Goal:** Product is the primary interface for creating and updating artifacts. Change impact is visible before mutations. Schema migration works end-to-end.

- [ ] `product feature/adr/test new` — scaffold with auto-incremented ID
- [ ] `product feature/adr/test link` — add edges, validates no cycles introduced
- [ ] `product feature/adr/test status` — update status; ADR supersession triggers impact report
- [ ] `product impact ADR-XXX / FT-XXX / TC-XXX` — reverse-graph reachability
- [ ] `product migrate schema --dry-run / --execute` — in-place schema upgrades
- [ ] `product checklist generate` — ordered by topological sort
- [ ] `product status` with phase, coverage, and dependency summary
- [ ] `product test untested` and `--failing` filters
- [ ] Front-matter validation on write — type checking, ID format, unknown fields preserved
- [ ] Git-aware: warn if modified files are uncommitted when regenerating checklist
- [ ] `schema-version = "1"` migration function registered and tested

**Exit criteria:** Supersede ADR-002 — confirm impact report prints before commit. Run `product migrate schema` on a v0 repository — all files updated, `schema-version` bumped. Run two concurrent `product feature status` commands — one succeeds, one exits E010. No data corruption. All IT-12–IT-19 integration tests pass. All TC-P005–TC-P011 property tests pass.

---

### Phase 3 — Graph Intelligence and CI Integration

**Goal:** Structural graph analysis works. SPARQL queries work. LLM benchmark validates the core value proposition. Product runs cleanly in CI with benchmark-validated performance.

- [ ] Betweenness centrality (Brandes' algorithm) — `product graph central`
- [ ] ADR ordering by centrality in context bundles (default, `--order id` override)
- [ ] `product graph stats` — centrality summary, φ formal coverage, link density, timing
- [ ] `product graph query` — embedded Oxigraph, SPARQL 1.1
- [ ] Centrality scores in TTL export on `graph rebuild`
- [ ] Benchmark runner binary (`benchmarks/runner/`)
- [ ] Three benchmark tasks: TC-030 (Raft election), TC-031 (front-matter parser), TC-032 (context bundle assembly)
- [ ] Benchmark rubric files and golden result baseline
- [ ] Benchmark suite — parse 200 files, centrality on 200 nodes, BFS depth 2 on 500 edges
- [ ] All timing invariants validated: parse < 200ms, centrality < 100ms, impact < 50ms
- [ ] `--format json` output on all list and navigation commands
- [ ] Shell completions (bash, zsh, fish)
- [ ] GitHub Actions example workflow

**Exit criteria:** `product graph central` returns ADR-001 as rank 1 on the PiCloud graph. Benchmark suite passes all timing invariants on a Raspberry Pi 5. TC-030, TC-031, TC-032 each pass: `score(product) ≥ 0.80` and `delta_vs_naive ≥ 0.15`. `product graph check` CI gate fails on a PR with a broken link.

---

### Phase 4 — Continuous Gap Analysis

**Goal:** Every ADR in the repository is continuously analysed for specification gaps by an LLM in CI. New gaps fail the build. Known gaps are baselined. Resolved gaps are tracked.

- [ ] `product gap check` — run gap analysis on all ADRs using full context bundles
- [ ] `product gap check ADR-XXX` — single ADR analysis
- [ ] `product gap check --changed` — CI mode: only ADRs changed in current commit plus 1-hop graph neighbours
- [ ] `product gap check --format json` — structured output for CI annotation
- [ ] `product gap report` — human-readable gap report across all ADRs
- [ ] `product gap suppress GAP-ID --reason "..."` — baseline a known gap
- [ ] `product gap unsuppress GAP-ID` — re-enable a suppressed gap
- [ ] `gaps.json` baseline file — tracks known, new, and resolved gaps
- [ ] Gap codes G001–G007 (see section 19)
- [ ] CI exit codes: 0 (no new gaps), 1 (new gaps found), 2 (analysis warnings)
- [ ] GitHub Actions workflow with gap annotation on PR diff
- [ ] `product gap stats` — gap density by ADR, gap resolution rate over time

**Exit criteria:** Run `product gap check --changed` on a PR that introduces ADR-019. Assert at least one gap is found (ADR-019 has no test criteria at time of writing). Suppress the gap. Assert subsequent run exits 0. Resolve the gap by adding a TC. Assert gap appears as resolved in `gaps.json`.

---

### Phase 5 — Agent Orchestration and Full Loop

**Goal:** The complete idea-to-implementation loop runs end-to-end. Authoring sessions are graph-aware. Implementation is a single command. Phone access works. Domain coverage is enforced at scale.

- [ ] `product mcp` — stdio transport, full tool registry
- [ ] `product mcp --http` — HTTP Streamable transport, bearer token auth
- [ ] `.mcp.json` scaffolding via `product install-hooks`
- [ ] CORS configuration for claude.ai access
- [ ] `product author feature/adr/review` — versioned system prompts, Claude Code integration
- [ ] `product adr review --staged` — pre-commit structural and LLM review
- [ ] `product install-hooks` — pre-commit hook installation
- [ ] `[domains]` vocabulary in `product.toml` — controlled concern domain registry
- [ ] `domains` and `scope` fields on ADR front-matter
- [ ] `domains` and `domains-acknowledged` fields on feature front-matter
- [ ] `product preflight FT-XXX` — domain coverage check before authoring or implementing
- [ ] `product feature acknowledge` — link or acknowledge domain gaps with reasoning
- [ ] `product graph coverage` — feature × domain coverage matrix
- [ ] `product graph coverage --domain D` and `--format json`
- [ ] `product implement FT-XXX` — preflight (Step 0), gap gate, drift check, context assembly, agent invocation
- [ ] `product implement --dry-run` — inspect assembled context without invoking agent
- [ ] `product verify FT-XXX` — TC runner protocol, status update, checklist regeneration
- [ ] TC front-matter `runner` and `runner-args` fields
- [ ] `product drift check` — spec-vs-implementation LLM analysis
- [ ] `product drift scan` — reverse: code → governing ADRs
- [ ] `drift.json` baseline, suppression lifecycle
- [ ] `product metrics record` — snapshot to `metrics.jsonl`
- [ ] `product metrics threshold` — CI gate on architectural fitness
- [ ] `product metrics trend` — ASCII sparkline
- [ ] Threshold configuration in `product.toml`
- [ ] `benchmarks/prompts/` — versioned system prompts for all session types and implement

**Exit criteria:** From a phone via claude.ai with Product MCP HTTP running: add a new feature with `domains: [security, networking]`, link an ADR, link a TC — all via MCP tool calls. Confirm files appear on disk. Run `product preflight` on the new feature — security domain flagged. Acknowledge with reasoning. Re-run — clean. Run `product implement` — preflight passes, gap gate passes, context assembled, agent invoked. Run `product verify` — TC executes, status updated. Run `product graph coverage` — coverage matrix renders with correct ✓/~ symbols. Run `product metrics record` and `product metrics threshold` — thresholds pass.

---

## 15. MCP Server

Product exposes its full tool surface as an MCP server. The same binary serves both transports. The transport is a startup flag, not a separate binary.

### Transports

**stdio** — spawned as a subprocess by Claude Code. Standard MCP transport. Local only. No authentication required — the parent process controls access.

```bash
# .mcp.json at repo root — committed, picked up automatically by Claude Code
{
  "mcpServers": {
    "product": {
      "command": "product",
      "args": ["mcp"],
      "cwd": "/path/to/repo"
    }
  }
}
```

**HTTP (Streamable HTTP)** — Product runs as an HTTP server. Any MCP-capable client can connect, including claude.ai configured with a remote MCP server URL. This is the transport for phone access.

```bash
# On your desktop or Pi:
product mcp --http --port 7777 --bind 0.0.0.0 --token $PRODUCT_TOKEN

# In claude.ai Settings → Connectors → Add MCP Server:
# URL:   http://your-machine.local:7777/mcp
# Header: Authorization: Bearer $PRODUCT_TOKEN
```

The HTTP transport implements the MCP Streamable HTTP spec — HTTP POST to `/mcp` for client→server, server-sent events on the same endpoint for streaming responses.

### Tool Surface

MCP tools are a curated subset of the CLI. All tools are read-safe by default. Write tools (scaffold, link, status update) require the `write` capability to be enabled in `product.toml`.

**Read tools (always enabled):**

| Tool | Equivalent CLI |
|---|---|
| `product_context` | `product context FT-XXX --depth N` |
| `product_feature_list` | `product feature list` |
| `product_feature_show` | `product feature show FT-XXX` |
| `product_feature_deps` | `product feature deps FT-XXX` |
| `product_adr_show` | `product adr show ADR-XXX` |
| `product_adr_list` | `product adr list` |
| `product_test_show` | `product test show TC-XXX` |
| `product_graph_check` | `product graph check` |
| `product_graph_central` | `product graph central` |
| `product_impact` | `product impact ADR-XXX` |
| `product_gap_check` | `product gap check ADR-XXX` |
| `product_adr_review` | `product adr review ADR-XXX` |
| `product_metrics_stats` | `product metrics stats` |

**Write tools (require `mcp.write = true` in product.toml):**

| Tool | Equivalent CLI |
|---|---|
| `product_feature_new` | `product feature new "title"` |
| `product_adr_new` | `product adr new "title"` |
| `product_test_new` | `product test new "title" --type TYPE` |
| `product_feature_link` | `product feature link FT-XXX --adr ADR-XXX` |
| `product_adr_status` | `product adr status ADR-XXX accepted` |
| `product_test_status` | `product test status TC-XXX passing` |
| `product_feature_status` | `product feature status FT-XXX complete` |

### Configuration

```toml
# product.toml
[mcp]
write = true              # enable write tools
token = ""                # bearer token for HTTP transport
                          # override with PRODUCT_MCP_TOKEN env var
port = 7777               # default HTTP port
cors-origins = []         # allowed CORS origins for HTTP transport
                          # ["https://claude.ai"] for claude.ai access
```

### Security Model

stdio transport has no authentication — the invoking process owns the repo. HTTP transport requires a bearer token when `--token` is set. Requests without a valid token receive 401. The token is never logged. For remote access from claude.ai, the token is set as a request header in the claude.ai connector configuration.

TLS is not handled by Product. For HTTPS, terminate TLS upstream (nginx, Caddy, Cloudflare Tunnel). Product binds HTTP; the proxy provides TLS.

---

## 16. Authoring Sessions

An authoring session is a `product author` command that starts Claude Code (or another configured agent) with a versioned system prompt pre-loaded and Product MCP active. Claude has full read access to the graph from the first message. It reads existing decisions before proposing new ones.

### Session Types

**`product author feature`** — for adding new product capability.

Claude's approach in this session:
1. Call `product_feature_list` — understand what exists
2. Call `product_graph_central` — identify foundational ADRs to read first
3. Call `product_context` on related features — understand the decision landscape
4. Ask clarifying questions grounded in what the graph already says
5. Scaffold the feature file, link dependencies, write ADRs and TCs
6. Call `product_graph_check` and `product_gap_check` before ending the session

**`product author adr`** — for adding a new architectural decision.

Claude's approach:
1. Call `product_graph_central` — read the top-5 ADRs before writing anything
2. Call `product_impact` on affected areas — understand blast radius
3. Draft the ADR with rejected alternatives and test criteria
4. Call `product_adr_review` on the draft — address findings before finishing
5. Link to affected features

**`product author review`** — spec gardening. No implementation intent.

Claude's approach:
1. Call `product_graph_check` — fix any structural issues first
2. Call `product_metrics_stats` — identify which metrics are weak
3. Walk through features with low `phi` scores — propose formal blocks
4. Find orphaned ADRs — propose feature links
5. Find features with no exit-criteria TC — propose them
6. End with a summary of what was improved and what remains

### System Prompts

Each session type has a versioned system prompt stored at:
```
benchmarks/prompts/author-feature-v1.md
benchmarks/prompts/author-adr-v1.md
benchmarks/prompts/author-review-v1.md
```

The prompt version is configured in `product.toml`:

```toml
[author]
feature-prompt-version = "1"
adr-prompt-version = "1"
review-prompt-version = "1"
agent = "claude-code"           # agent to invoke
```

### Phone Workflow

When `product mcp --http` is running on your desktop or server, authoring sessions are not limited to `product author` invocations from the command line. The same tool surface is available in any claude.ai conversation configured with the Product MCP server:

1. Open claude.ai on your phone
2. Start a new conversation — Product tools are available as connectors
3. "Add a rate limiting feature to PiCloud" — Claude calls `product_feature_list`, `product_graph_central`, reads context, asks questions, scaffolds files
4. Files land in your repo (via the HTTP MCP server writing to the filesystem)
5. Later, at your desktop: `git pull && product implement FT-009`

The phone conversation is the authoring session. The desktop is the implementation environment. The repo is the shared state between them.

---

## 17. Agent Orchestration

### `product implement FT-XXX`

The implementation command runs a five-step pipeline:

**Step 1 — Gap gate.** Runs `product gap check FT-XXX`. If any high-severity gaps (G001, G002, G005) are found and unsuppressed, the command exits with an explanation. You cannot implement a specification with known high-severity gaps — the agent would be working from an incomplete contract.

**Step 2 — Drift check.** Runs `product drift check --phase N` for the feature's phase. If the codebase has already drifted from a related ADR, the agent needs to know before it writes more code.

**Step 3 — Context assembly.** Runs `product context FT-XXX --depth 2`. Wraps it in the versioned implementation prompt from `benchmarks/prompts/implement-v1.md`.

**Step 4 — Agent invocation.** Invokes the configured agent with the assembled context. For Claude Code: pipes the context bundle to `claude --print` or writes it to a temp file and passes the file path.

**Step 5 — Auto-verify.** On agent completion, runs `product verify FT-XXX` automatically unless `--no-verify` is passed.

```
product implement FT-001
  ✓ Gap check: no high-severity gaps
  ✓ Drift check: no drift detected
  → Assembling context bundle (FT-001, 4 ADRs, 6 TCs, depth 2)
  → Invoking claude-code...
  [agent output streams here]
  → Running product verify FT-001...
  TC-001 binary-compiles         PASS
  TC-002 raft-leader-election    PASS
  TC-003 raft-leader-failover    FAIL
  ✗ 1 test failing. Feature status: in-progress
```

### `product verify FT-XXX`

Verify reads each linked TC file and derives how to run it from the TC metadata:

```yaml
---
id: TC-002
type: scenario
runner: cargo-test
runner-args: ["--test", "raft_leader_election", "--", "--nocapture"]
---
```

The `runner` and `runner-args` fields in TC front-matter tell verify how to execute the criterion. Supported runners: `cargo-test`, `bash`, `pytest`, `custom`.

On completion:
- TC statuses updated (`passing`, `failing`)
- Feature status updated if all TCs pass → `complete`
- `checklist.md` regenerated
- Results written to stdout in the error model format (ADR-013)

### Implementation Prompt

The implementation prompt wraps the context bundle with explicit constraints:

```markdown
# Implementation Task: {FEATURE_ID} — {FEATURE_TITLE}

## Your role
Implement this feature according to the architectural decisions in the
context bundle. The test criteria define done — your implementation is
complete when all linked TCs pass.

## Current test status
{TC_STATUS_TABLE}

## Hard constraints
- Language: determined by ADR-001
- No new dependencies without a linked ADR
- Run the test suite before reporting complete
- When done: `product verify {FEATURE_ID}`

## Context Bundle
{BUNDLE}
```

The test status table is generated fresh at invocation time — the agent sees which TCs are currently passing and which are not.

---

## 18. Engineering Workflows

### Drift Detection

`product drift` checks whether the codebase matches what the ADRs decided. The LLM receives the ADR's context bundle plus the source files most likely to implement it (resolved via configurable path patterns in `product.toml`).

```toml
[drift]
source-roots = ["src/", "lib/"]
ignore = ["tests/", "benches/"]
```

Drift codes:

| Code | Severity | Description |
|---|---|---|
| D001 | high | Decision not implemented — ADR says X, no code implements X |
| D002 | high | Decision overridden — code does Y, ADR says do X |
| D003 | medium | Partial implementation — some aspects of the decision implemented |
| D004 | low | Implementation ahead of spec — code does X but no ADR documents why |

Drift findings follow the same baseline/suppression model as gap findings (`drift.json`). `product drift scan src/consensus/` is the reverse direction — given source code, identify which ADRs govern it. Useful for onboarding and code review.

### Fitness Functions

`product metrics record` snapshots the current repository health into `metrics.jsonl` (one JSON line per run, committed to the repo):

```json
{"date":"2026-04-11","spec_coverage":0.87,"test_coverage":0.72,"exit_criteria_coverage":0.61,"phi":0.68,"gap_density":0.4,"gap_resolution_rate":0.75,"centrality_stability":0.02}
```

Thresholds declared in `product.toml` are checked by `product metrics threshold` in CI — this is the architectural fitness function gate. A declining `phi` below 0.70 fails CI just as a broken link does.

`product metrics trend` renders an ASCII chart to terminal for quick visual inspection.

### Pre-Commit Review

`product install-hooks` installs a pre-commit hook that runs `product adr review --staged` before every commit. The hook is advisory — it prints findings but does not block the commit. The CI gap analysis gate is the enforcement point; pre-commit is the fast-feedback loop.

The review checks locally (no LLM, instant):
- Required sections present
- At least one linked feature and one linked TC
- Status field is set
- Evidence blocks present on formal blocks

Then a single LLM call checks:
- Internal consistency of rationale
- Contradiction with linked ADRs
- Obvious missing tests given the claims made

---

## 19. Gap Analysis

Gap analysis is the continuous LLM-driven process of identifying specification incompleteness, inconsistency, and missing coverage in the repository's ADRs. It runs in CI against changed ADRs and produces structured findings that are tracked over time.

### What Gap Analysis Checks

Gap analysis checks seven classes of gap, each with a code and a severity:

| Code | Severity | Description |
|---|---|---|
| G001 | high | **Missing test coverage** — ADR makes a testable claim with no linked TC |
| G002 | high | **Untested formal invariant** — `⟦Γ:Invariants⟧` block exists but no scenario or chaos TC exercises it |
| G003 | medium | **Missing rejected alternatives** — ADR has no documented rejected alternatives |
| G004 | medium | **Undocumented constraint** — ADR rationale references an external constraint not captured in any linked artifact |
| G005 | high | **Architectural contradiction** — this ADR makes a claim logically inconsistent with a linked ADR |
| G006 | medium | **Feature coverage gap** — a feature aspect is not addressed by any linked ADR |
| G007 | low | **Stale rationale** — ADR rationale references something contradicted by a more recent superseding ADR |

### Context Used for Analysis

Each ADR is analysed with its full depth-2 context bundle — the ADR, all linked features, all linked test criteria, and all related ADRs reachable within 2 hops. This is the same bundle an implementation agent would receive, which means gap analysis validates the context bundle's completeness from the same perspective.

### Output Format

Gap findings are structured JSON, written to stdout (not stderr — they are results, not errors):

```json
{
  "adr": "ADR-002",
  "run_date": "2026-04-11T09:00:00Z",
  "product_version": "0.1.0",
  "findings": [
    {
      "id": "GAP-ADR002-001",
      "code": "G001",
      "severity": "high",
      "description": "The invariant 'exactly one leader at all times' stated in the rationale has no linked chaos test exercising a split-brain scenario.",
      "affected_artifacts": ["ADR-002"],
      "suggested_action": "Add a chaos TC validating leader uniqueness under network partition.",
      "suppressed": false
    }
  ],
  "summary": { "high": 1, "medium": 0, "low": 0, "suppressed": 0 }
}
```

### The Baseline File

`gaps.json` at the repository root tracks gap state across runs:

```json
{
  "schema-version": "1",
  "suppressions": [
    {
      "id": "GAP-ADR002-001",
      "reason": "Split-brain chaos test deferred to phase 2",
      "suppressed_by": "git:abc123",
      "suppressed_at": "2026-04-11T09:00:00Z"
    }
  ],
  "resolved": [
    {
      "id": "GAP-ADR001-003",
      "resolved_at": "2026-04-12T14:30:00Z",
      "resolving_commit": "git:def456"
    }
  ]
}
```

A gap is **new** if its ID does not appear in `gaps.json`. A gap is **suppressed** if it appears in `suppressions`. A gap is **resolved** if it was previously suppressed or known and is no longer detected. Only new unsuppressed gaps cause CI to exit 1.

### Gap IDs

Gap IDs are deterministic and stable. They are derived from: the ADR ID, the gap code, and a hash of the affected artifact IDs and gap description. The same logical gap detected on two different runs produces the same ID. This is critical for suppression to work correctly — a suppressed gap must remain suppressed across runs.

```
GAP-{ADR_ID}-{GAP_CODE}-{SHORT_HASH}
e.g. GAP-ADR002-G001-a3f9
```

### CI Integration

The `--changed` flag is the primary CI mode. It uses `git diff --name-only HEAD~1` to identify changed ADR files, then expands to include 1-hop graph neighbours (ADRs that share a feature with any changed ADR). This scoping strategy ensures that:

- A PR that modifies ADR-002 also analyses ADR-005 if they share a feature (because the change may create new inconsistencies between them)
- The analysis set is bounded — at most `changed_adrs × avg_neighbour_count` ADRs are analysed per run
- Unrelated ADRs are never analysed, keeping CI cost proportional to change scope

### LLM Prompt Design

The gap analysis prompt is fixed and versioned. It does not change between runs unless a new version is explicitly released. This ensures that the same repository state produces comparable findings across runs.

The prompt instructs the model to:
1. Read the context bundle
2. Check only for the seven defined gap types — not for general quality issues
3. Respond only in the specified JSON schema — no prose preamble
4. Assign the deterministic gap ID format
5. For G005 (contradiction), cite the specific claims from both ADRs that conflict

The prompt is stored at `benchmarks/prompts/gap-analysis-v1.md` and referenced by version in `product.toml`:

```toml
[gap-analysis]
prompt-version = "1"
model = "claude-sonnet-4-6"
max-findings-per-adr = 10
severity-threshold = "medium"   # gaps below this severity are informational only
```

### Determinism Strategy

LLM output is non-deterministic. Three measures stabilise gap analysis for CI:

1. **Temperature=0** for all gap analysis calls
2. **Structured JSON output only** — the model is instructed to produce only JSON matching the schema. Findings that cannot be parsed into the schema are discarded with a warning, not propagated as failures
3. **Run twice, intersect** — for high-severity findings (G001, G002, G005), the analysis is run twice. Only findings present in both runs are reported. This eliminates hallucinated gaps that appear in one run but not another

The cost of running twice is justified for high-severity findings — a false G005 (architectural contradiction) that fails CI is highly disruptive. Medium and low severity findings are single-run only.

---

## 20. Resolved Decisions

The following questions were raised during drafting and resolved before implementation began.

1. **Context bundle token budget** — resolved: token budget management is the agent's responsibility. Product assembles complete, accurate bundles. Truncation, summarisation, or chunking is a concern for the agent or the pipeline invoking Product, not for Product itself.

2. **ADR supersession in context** — resolved: when a context bundle includes a superseded ADR, it is replaced by its successor. The superseded ADR does not appear in the bundle. This keeps the bundle actionable — an agent receiving it sees only current decisions.

3. **Test criterion ownership on feature abandonment** — resolved: when a feature is marked `abandoned`, its linked test criteria are automatically orphaned (their `validates.features` reference is removed and they appear in `product test untested`). No explicit action is required from the developer. `product graph check` reports orphaned tests as warnings (exit code 2), not errors.

4. **Multi-repo support** — resolved: Product operates on a single repository. Multi-repo workspace mode is not planned. The `product.toml` is always at the root of one repository.
