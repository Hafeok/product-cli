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
4. **Implementation status tracking** — status is declared in feature and TC front-matter and updated by `product verify`. `product status` renders phase gate state and exit criteria progress. `checklist.md` is an optional generated view for stakeholder sharing, not a data source.
5. **Queryable knowledge graph** — the CLI can answer relational queries: which ADRs apply to this feature, which tests validate this decision, which features are in phase 1 and have no tests.
6. **RDF export** — the derived graph can be exported to `index.ttl` for SPARQL tooling, LLM injection, or external graph queries.
7. **Rust, single binary** — Product ships as a single compiled binary with no runtime dependencies. It can run in CI, on a developer laptop, or inside an agentic pipeline.
8. **Repository-native** — Product operates on a directory of markdown files. No server, no database, no configuration beyond a single `product.toml` at the repo root.
9. **MCP server — stdio and HTTP** — Product exposes its full tool surface as an MCP server. stdio transport supports Claude Code on the local machine. HTTP transport supports remote access from any MCP-capable client, including claude.ai on mobile.
10. **Graph-aware authoring** — `product author` sessions give Claude full read and scaffold access to the graph during spec writing. The authoring agent cannot implement an idea without first understanding what already exists.
11. **Knowledge pipeline** — Product provides every primitive an agent needs to work on a feature: preflight, gap check, drift check, context assembly, and post-implementation verification. The sequence of calls is the pipeline; Product does not invoke agents.
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

**Feature (`FT-XXX`)** — A unit of product capability. Corresponds to a section of a PRD. Declares its phase, status, linked ADRs, linked test criteria, and linked dependencies. A feature is the primary navigation unit of the knowledge graph: everything else is reachable from it.

**Architectural Decision Record (`ADR-XXX`)** — A single architectural decision. Declares context, decision, rationale, rejected alternatives, and the features it applies to. An ADR may apply to multiple features. An ADR may supersede or be superseded by another ADR. An ADR may govern one or more dependencies.

**Test Criterion (`TC-XXX`)** — A single verifiable assertion about system behaviour. A test criterion has a type (scenario, invariant, chaos, exit-criteria, benchmark), is linked to one or more features and one or more ADRs, and belongs to a phase. Test criteria are extracted from ADRs during migration — they are not co-located with the decisions they verify.

**Dependency (`DEP-XXX`)** — A declared external system that one or more features require. Captures the runtime facts about an external dependency: type, version constraint, interface description, and an optional availability check command. Distinct from ADRs, which capture the decision to use the dependency. Six types: `library`, `service`, `api`, `tool`, `hardware`, `runtime`.

### Relationships

```
Feature    ──── implementedBy ──────► ADR
Feature    ──── validatedBy ─────────► TestCriterion
Feature    ──── uses ────────────────► Dependency
ADR        ──── testedBy ───────────► TestCriterion
ADR        ──── supersedes ──────────► ADR
ADR        ──── governs ────────────► Dependency
Feature    ──── depends-on ──────────► Feature
Dependency ──── supersedes ──────────► Dependency
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
    ADR-029-code-quality.md
  /deps
    DEP-001-openraft.md
    DEP-002-oxigraph.md
    DEP-010-raspberry-pi-nvme.md
  /tests
    TC-001-binary-compiles.md
    TC-002-raft-leader-election.md
    TC-003-raft-leader-failover.md
    TC-CQ-001-file-length-hard.md   ← cross-cutting code quality checks
    TC-CQ-002-file-length-warn.md
    TC-CQ-003-function-length.md
    TC-CQ-004-module-structure.md
    TC-CQ-005-single-responsibility.md
  /graph
    index.ttl               ← generated, never hand-edited
  checklist.md              ← generated view (gitignored by default)
                            ← not a data source — front-matter owns status

/scripts
  /checks                   ← enforcement scripts (not part of Product binary)
    file-length.sh
    function-length.sh
    module-structure.sh
    single-responsibility.sh
  /harness                  ← example harness scripts (not part of Product binary)
    implement.sh
    author.sh
    loop.sh

/benchmarks
  /prompts                  ← versioned system prompts
    author-feature-v1.md
    author-adr-v1.md
    author-review-v1.md
    implement-v1.md
  /tasks                    ← LLM benchmark tasks
    task-001-raft-leader-election/

/src                        ← Rust source, structured per ADR-029
  main.rs
  error.rs
  config.rs
  /graph
  /parse
  /context
  /commands
  /verify
  /mcp
  /io
```

Subdirectory names and file prefixes are configurable in `product.toml`. The `docs/` layout above is the default.

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
adrs: [ADR-001, ADR-002, ADR-003, ADR-006]
tests: [TC-001, TC-002, TC-003, TC-004]
uses: [DEP-001, DEP-010]     # external dependencies this feature requires
domains-acknowledged:
  scheduling: >
    No workload scheduling in phase 1. Cluster foundation does not
    place containers — that is phase 2. Intentionally out of scope.
bundle:
  measured-at: 2026-04-11T09:14:22Z
  depth-1-adrs: 6
  depth-2-adrs: 11
  tcs: 8
  domains: 5
  tokens-approx: 7400
---
```

### Dependency

```yaml
---
id: DEP-001
title: openraft
type: library            # library | service | api | tool | hardware | runtime
source: crates.io
version: ">=0.9,<1.0"
status: active           # active | deprecated | evaluating | migrating
features: [FT-001, FT-002, FT-005]
adrs: [ADR-002]          # ADR that governs use of this dependency
availability-check: ~    # null for libraries — no runtime check needed
breaking-change-risk: medium   # low | medium | high
---
```

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
availability-check: "pg_isready -h ${PG_HOST:-localhost} -p ${PG_PORT:-5432}"
breaking-change-risk: low
---
```

The `interface` block is optional and type-specific. For `service` and `api` types it captures the runtime contract an agent needs to write correct integration code. The `availability-check` is a shell command — exit 0 means available. Product never satisfies this check; it only evaluates it. It records the last-measured context bundle dimensions for this feature. `product metrics threshold` reads this block to identify features that breach configured size thresholds. Features with no `bundle` block have not been measured and are reported as W012 by `product graph check`.

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
content-hash: sha256:a3f9...  # optional: computed on acceptance (ADR-032)
amendments:                   # optional: audit trail for post-acceptance edits
  - date: 2026-04-14T09:00:00Z
    reason: "Fix typo in rationale"
    previous-hash: sha256:b4c5...
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
status: unimplemented        # unimplemented | implemented | passing | failing | unrunnable
validates:
  features: [FT-001]
  adrs: [ADR-002]
phase: 1
runner: cargo-test           # cargo-test | bash | pytest | custom
                             # omit if test infrastructure not yet available
runner-args: ["--test", "raft_leader_election", "--", "--nocapture"]
runner-timeout: 60s          # optional, default 30s
requires: [binary-compiled]  # optional — declarative prerequisites
                             # names defined in [verify.prerequisites] in product.toml
                             # Product checks these before running — never satisfies them
content-hash: sha256:a3f9...  # optional: computed by `product hash seal` (ADR-032)
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
deps = "docs/deps"
graph = "docs/graph"
checklist = "docs/checklist.md"    # generated view only — not a data source
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
dep = "DEP"

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
code-quality    = "File size, function length, module structure, naming"

# MCP server settings (product mcp)
[mcp]
write = true                    # enable write tools over MCP
port = 7777                     # HTTP transport port
cors-origins = ["https://claude.ai"]
# token = ""                    # override with PRODUCT_MCP_TOKEN env var

# Checklist — generated view for human consumption, not a data source
# Front-matter owns status. Agents use `product status` and `product feature next`.
[checklist]
in-gitignore = true             # default: do not commit checklist.md
                                # set false to commit for GitHub rendering

# Versioned system prompts for authoring (product prompts get/list/update)
# Product does not invoke agents — these are prompt file version pins only
[author]
feature-prompt-version = "1"
adr-prompt-version = "1"
review-prompt-version = "1"
implement-prompt-version = "1"

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

# TC prerequisite conditions (product verify)
# Each entry is a name and a shell command — exit 0 = satisfied, non-zero = not satisfied.
# Product checks these before running a TC that declares them in `requires:`.
# Product never satisfies prerequisites — it only checks them.
[verify.prerequisites]
binary-compiled      = "test -f target/release/picloud"
two-node-cluster     = "product graph query 'ASK { ?n a picloud:Node } HAVING COUNT(?n) >= 2'"
raft-leader-elected  = "product graph query 'ASK { ?n picloud:hasRole picloud:Leader }'"

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

# Bundle size thresholds — signals features that may need splitting
bundle_depth1_adr_max       = { max = 8,     severity = "warning" }
bundle_tokens_max           = { max = 12000, severity = "warning" }
bundle_domains_max          = { max = 6,     severity = "warning" }
features_over_adr_threshold = { max = 3,     severity = "warning" }
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
product feature next                 # next feature by topological order + phase gate
product feature next --ignore-phase-gate  # skip phase gate (with warning)

product adr list [--status STATUS]
product adr show ADR-002
product adr features ADR-002         # which features reference this ADR
product adr tests ADR-002            # which tests validate this decision

product test list [--phase N] [--type TYPE] [--status STATUS]
product test show TC-002
product test untested                # features with no linked tests

product dep list [--type TYPE] [--status STATUS]
product dep show DEP-001
product dep features DEP-001         # which features use this dependency
product dep check DEP-005            # run availability check manually
product dep check --all              # run all availability checks
product dep bom                      # full dependency bill of materials
product dep bom --format json        # machine-readable for security audits
```

### Context Assembly

```
product context FT-001               # feature + direct ADRs + direct tests (depth 1)
product context FT-001 --depth 2     # transitive context: deps, shared ADRs, their tests
product context FT-001 --measure     # assemble + record bundle dimensions to front-matter
                                     # and metrics.jsonl
product context --phase 1            # all features in phase 1, with full context
product context --phase 1 --adrs-only  # phase 1 features + ADRs, no tests
product context ADR-002              # ADR + all linked features + all linked tests
product context --order id           # override default centrality ordering of ADRs
```

ADRs within a bundle are ordered by betweenness centrality descending by default — the most structurally important decisions appear first. Pass `--order id` for the previous ID-ascending behaviour.

### Status and Checklist

```
product status                       # overview: phases with gate state [OPEN/LOCKED],
                                     # features by status, exit criteria progress
product status --phase 1             # phase detail: all exit-criteria TCs with pass/fail
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
                                     # φ (formal block coverage) across test criteria,
                                     # bundle size summary and features over threshold
product graph stats --feature FT-003 # single-feature detail with split suggestions
product graph central                # top-10 ADRs by betweenness centrality
product graph central --top N        # configurable N
product graph central --all          # full ranked list
product graph coverage               # feature × domain coverage matrix
product graph coverage --domain security   # filter to one domain column
product graph coverage --format json       # machine-readable for CI
product impact ADR-002               # full affected set if this decision changes
product impact FT-001                # what depends on this feature completing
product impact TC-003                # what depends on this test criterion
product impact DEP-001               # features and ADRs affected by a dep change
```

`product graph check` also validates:
- No cycles in the `depends-on` feature DAG (exit code 1)
- Phase label / dependency order disagreements (exit code 2)
- Acknowledgements without reasoning — E011 (exit code 1)
- Domains declared in front-matter not present in `product.toml` vocabulary — E012 (exit code 1)
- Dependencies with no linked ADR — E013 (exit code 1)

### Pre-flight and Domain Coverage

```
product preflight FT-001             # domain coverage check — run before authoring
product preflight FT-001 --format json

product feature acknowledge FT-009 --domain security \
  --reason "no trust boundaries introduced"
product feature acknowledge FT-009 --adr ADR-040 \
  --reason "standard output conventions apply"
```

Pre-flight must be clean before any agent begins implementation work. Pre-flight gaps are resolved by linking an ADR (`product feature link`) or acknowledging a domain/ADR with explicit reasoning. Acknowledgements without reasoning are E011 hard errors.

### Authoring

```
product feature new "Cluster Foundation"   # scaffold FT-XXX file with next ID
product adr new "Use openraft for consensus"
product test new "Raft leader election" --type scenario
product dep new "openraft" --type library        # scaffold DEP-XXX + ADR-XXX stub
product dep new "PostgreSQL" --type service --adr ADR-015  # link to existing ADR

product feature link FT-001 --adr ADR-002  # add edge (mutates front-matter)
product feature link FT-001 --test TC-002
product feature link FT-001 --dep DEP-001  # add uses edge

product adr status ADR-002 accepted        # set ADR status (writes content-hash)
product adr amend ADR-002 --reason "..."   # record amendment, recompute hash (ADR-032)
product adr rehash ADR-002                 # seal accepted ADR that predates content-hash
product adr rehash --all                   # seal all accepted ADRs without content-hash
product test status TC-002 passing         # set test status
product feature status FT-001 complete     # set feature status

product hash seal TC-002                   # compute and write content-hash for a TC
product hash seal --all-unsealed           # seal all TCs without a content-hash
product hash verify                        # verify all content-hashes (fast integrity check)
product hash verify ADR-002                # verify one artifact's content-hash
```

### Migration

```
product migrate from-prd PRD.md           # parse monolithic PRD → feature files
product migrate from-adrs ADRS.md         # parse monolithic ADR file → adr files + test files
product migrate validate                  # report what would be created without writing
product migrate link-tests                # infer TC→Feature links via ADR relationships
product migrate link-tests --dry-run      # preview without writing
product migrate link-tests --adr ADR-002  # scope to one ADR's TCs
```

### Graph Inference

```
product graph infer                       # infer all missing transitive TC→Feature links
product graph infer --dry-run             # preview without writing
product graph infer --feature FT-009      # scope to one feature's new links
product graph infer --adr ADR-021         # scope to one ADR's TCs
```

Both `migrate link-tests` and `graph infer` use the same algorithm: for each domain-scoped ADR, find features that link it and TCs that validate it, add missing TC→Feature links. Cross-cutting ADRs are excluded. Both commands also update the reverse direction — adding the TC to the feature's `tests` list.

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

### Authoring Resources

```
product prompts init                  # scaffold default prompt files in benchmarks/prompts/
product prompts list                  # list available prompts and versions
product prompts get author-feature    # print prompt to stdout (pipe to any agent)
product prompts get implement
product prompts update author-feature # pull latest default version

product adr review ADR-XXX            # review a specific ADR (structural + LLM)
product adr review --staged           # review all staged ADR files (used by pre-commit hook)
product install-hooks                 # install pre-commit hook in .git/hooks/
```

### Implementation Pipeline

```
# Knowledge commands — call these before invoking an agent:
product preflight FT-001
product gap check FT-001 --severity high
product drift check --phase 1
product context FT-001 --depth 2 --measure

# After agent completes:
product verify FT-001                 # run TCs, update status, regenerate checklist
product verify FT-001 --tc TC-002     # run a single TC only
product verify --platform             # run all TCs linked to cross-cutting ADRs
```

Product does not invoke agents. See `scripts/harness/` for example shell scripts that compose these commands into a complete flow.

---

## 8. Context Bundle Format

The context command assembles a deterministic markdown bundle. Order is always: feature → ADRs (by betweenness centrality descending) → dependencies (libraries first, then services, then others) → test criteria (by phase, then type: exit-criteria, scenario, invariant, chaos).

The bundle opens with an AISP-influenced formal header block (see ADR-011) that an agent can parse without reading the full document. It declares the bundle's identity, all linked artifact IDs, and aggregate evidence metrics derived from the test criteria evidence blocks.

```markdown
# Context Bundle: FT-001 — Cluster Foundation

⟦Ω:Bundle⟧{
  feature≜FT-001:Feature
  phase≜1:Phase
  status≜InProgress:FeatureStatus
  generated≜2026-04-11T09:00:00Z
  implementedBy≜⟨ADR-001,ADR-002,ADR-003,ADR-006⟩:Decision+
  uses≜⟨DEP-001,DEP-010⟩:Dependency+
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

## Dependencies

### DEP-001 — openraft [library, >=0.9,<1.0]

[dependency body text, front-matter stripped]

Interface: no runtime interface (build-time library)
Availability: no check required

### DEP-010 — Raspberry Pi 5 NVMe [hardware]

[dependency body text, front-matter stripped]

Interface:
  arch: aarch64 / storage-min-gb: 500
  availability-check: uname -m | grep -q aarch64 && ls /dev/nvme*

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

The bundle evidence block `⟦Ε⟧` at the top is computed as the mean of all linked test criterion `δ` values. The `uses` field in `⟦Ω:Bundle⟧` lists all linked dependency IDs — an agent sees the full runtime dependency surface before reading any content.

---

## 9. Graph Model

Product builds an in-memory directed graph from front-matter on every invocation. The graph is also exportable as RDF Turtle via `product graph rebuild`.

### Edge Types

| Edge | From | To | Description |
|---|---|---|---|
| `implementedBy` | Feature | ADR | Feature is governed by this decision |
| `validatedBy` | Feature | TestCriterion | Feature is verified by this test |
| `uses` | Feature | Dependency | Feature requires this external system |
| `testedBy` | ADR | TestCriterion | Decision is verified by this test |
| `supersedes` | ADR | ADR | This decision replaces another |
| `governs` | ADR | Dependency | Decision that chose this dependency |
| `depends-on` | Feature | Feature | Implementation dependency — must complete before |
| `supersedes` | Dependency | Dependency | Migration — this dependency replaces another |

The reverse of every edge is implicit. Impact analysis (`product impact`) traverses the reverse graph to compute reachability.

### Graph Algorithms

| Algorithm | Applied to | Command | Purpose |
|---|---|---|---|
| Topological sort (Kahn's) | Feature `depends-on` DAG | `product feature next` | Correct implementation ordering with phase gate |
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
@prefix dep: <https://product-meta/dep/> .

ft:FT-001 a pm:Feature ;
    pm:title "Cluster Foundation" ;
    pm:phase 1 ;
    pm:status pm:InProgress ;
    pm:implementedBy adr:ADR-001 ;
    pm:implementedBy adr:ADR-002 ;
    pm:validatedBy tc:TC-001 ;
    pm:validatedBy tc:TC-002 ;
    pm:uses dep:DEP-001 ;
    pm:uses dep:DEP-010 .

ft:FT-003 a pm:Feature ;
    pm:dependsOn ft:FT-001 ;
    pm:dependsOn ft:FT-002 .

adr:ADR-002 a pm:ArchitecturalDecision ;
    pm:title "openraft for Cluster Consensus" ;
    pm:status pm:Accepted ;
    pm:betweennessCentrality 0.731 ;
    pm:appliesTo ft:FT-001 ;
    pm:testedBy tc:TC-002 ;
    pm:governs dep:DEP-001 .

tc:TC-002 a pm:TestCriterion ;
    pm:title "Raft Leader Election" ;
    pm:type pm:Scenario ;
    pm:status pm:Unimplemented ;
    pm:validates ft:FT-001 ;
    pm:validates adr:ADR-002 .

dep:DEP-001 a pm:Dependency ;
    pm:title "openraft" ;
    pm:depType pm:Library ;
    pm:version ">=0.9,<1.0" ;
    pm:status pm:Active ;
    pm:breakingChangeRisk pm:Medium ;
    pm:usedBy ft:FT-001 ;
    pm:usedBy ft:FT-002 ;
    pm:governedBy adr:ADR-002 .
```

Betweenness centrality scores and dependency triples are written into the TTL export on `graph rebuild` so external SPARQL tools can query on them.

---

## 10. Generated Checklist (Optional View)

`checklist.md` is a generated human-readable view of implementation status. It is not a data source, not an agent input, and not a source of truth. Front-matter owns status. Agents use `product status`, `product feature next`, and `product_feature_list` — none of these require checklist.md to exist.

`checklist.md` serves one legitimate purpose: giving stakeholders and code reviewers a readable snapshot of project state without requiring Product to be installed. GitHub renders markdown checkboxes natively, which makes it useful for repository visibility.

**Default behaviour:** `checklist.md` is listed in `.gitignore` by default. It is generated locally when needed and not committed. Teams that want GitHub visibility can opt out of gitignore by setting `checklist-in-gitignore = false` in `product.toml`.

```
product checklist generate        # regenerate from current front-matter
product checklist generate --open  # regenerate and open in browser
```

Generated format:

```markdown
# Implementation Checklist

> Generated by product v0.1 | Last updated: 2026-04-11
> Source of truth: feature and TC front-matter (not this file)
> Do not edit — regenerate with `product checklist generate`
> Phase 1: OPEN (exit criteria 2/4 passing) | Phase 2: LOCKED

## Phase 1 — Cluster Foundation [OPEN]

### FT-001 — Cluster Foundation [in-progress]

- [x] ADR-001: Rust as implementation language (accepted)
- [x] ADR-002: openraft for consensus (accepted)  ← DEP-001 openraft
- [ ] TC-001: Binary compiles (exit-criteria) — unimplemented
- [ ] TC-002: Raft leader election (scenario) — unimplemented
```

The generated file includes the phase gate state in the header so a stakeholder reading it understands why Phase 2 features all show as planned.

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
| E013 | Dependency has no linked ADR — every dependency requires a governing decision |
| E014 | ADR body or title changed after acceptance — content-hash mismatch (ADR-032) |
| E015 | TC body or protected fields changed after sealing — content-hash mismatch (ADR-032) |

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
| W012 | Feature has no `bundle` measurement — run `product context FT-XXX --measure` |
| W013 | Feature uses a deprecated or migrating dependency |
| W015 | Dependency `availability-check` failed during preflight |
| W016 | Accepted ADR has no content-hash — seal with `product adr rehash` (ADR-032) |

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
| G008 | medium | Feature uses a dependency with no ADR governing its use |

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

Domain coverage is part of the pre-implementation knowledge check — pre-flight must be clean before an agent begins work on a feature. See ADR-026.

### Bundle Size and Split Suggestions

`product graph stats --feature FT-XXX` shows the feature's last-measured bundle dimensions and, when thresholds are exceeded, a domain-based split suggestion:

```
product graph stats --feature FT-003

FT-003 — RDF Store  [in-progress, phase 3]
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Bundle dimensions (measured 2026-04-11T09:14:22Z):
  depth-1-adrs:  9   ⚠ exceeds threshold (max 8)
  depth-2-adrs:  14
  tcs:           12
  domains:       4
  tokens-approx: 11,200

Domain breakdown of linked ADRs:
  data-model   4 ADRs  — RDF store, SPARQL, ontology, projections
  networking   2 ADRs  — SPARQL endpoint, mTLS
  iam          2 ADRs  — IAM-gated SPARQL, per-product scoping
  storage      1 ADR   — Oxigraph persistence

⚠ depth-1-adrs (9) exceeds threshold (8). Consider splitting.

Suggested split along domain boundaries:
  FT-003a  RDF Store Core    — data-model (4), storage (1)  → 5 ADRs, ~5,800 tokens
  FT-003b  RDF Store Access  — networking (2), iam (2)      → 4 ADRs, ~4,200 tokens
```

The split suggestion is derived purely from the domain taxonomy — no LLM call, instant. It groups linked ADRs by domain and proposes the smallest number of sub-features that keeps each one under the ADR threshold. The token estimates are recomputed from existing `bundle_measure` entries in `metrics.jsonl`.

`product graph stats` (no `--feature` flag) shows the repository-wide summary:

```
Bundle size summary (from last measurements):
  features measured:  12 / 15
  depth-1-adr p50:    5    p95: 8
  tokens-approx p50:  6,200   p95: 10,800
  features over adr threshold (>8):  2  — FT-003, FT-007
  features over token threshold (>12000):  0
  unmeasured features (W012):  3  — FT-013, FT-014, FT-015
```

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

# Step 2: confirm feature→ADR links manually
product graph check          # shows W001/W002 gaps to fill
product feature link FT-001 --adr ADR-001 --adr ADR-002

# Step 3: infer transitive TC→Feature links
product migrate link-tests --dry-run    # preview what will be linked
product migrate link-tests              # apply all inferred links

# Step 4: final check and checklist
product graph check          # W002 warnings should drop significantly
product checklist generate
```

The migration parser uses heading structure to detect artifact boundaries and extracts phase references, status markers, and test criteria from subsections. It does not infer `depends-on` edges or feature→ADR links — those require human review.

`product migrate link-tests` is the post-migration step that closes the loop: once feature→ADR links are confirmed, it infers all TC→Feature links transitively (through domain-scoped ADRs only — cross-cutting ADRs are excluded to avoid linking every TC to every feature). See ADR-027.

The source document is never modified. Migration can be re-run safely.

---

## 14. Phase Plan

### Phase 1 — Core Graph and Context

**Goal:** A developer can migrate an existing PRD and ADR file, navigate the graph, and assemble a context bundle. The binary is production-safe: no panics on user input, atomic file writes, schema versioning in place.

- [ ] `product.toml` parsing and repository discovery including `schema-version`
- [ ] Schema version validation on startup — E008 on forward incompatibility, W007 on upgrade available
- [ ] Front-matter parser for feature, ADR, test, and dependency files including `depends-on` and `uses`
- [ ] `product dep list/show/features/check/bom`
- [ ] `product dep new` — scaffold DEP-XXX with next ID
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
- [ ] `product migrate link-tests --dry-run / --execute` — transitive TC→Feature inference
- [ ] `product graph infer` — general-purpose inference, post-link confirmation prompt
- [ ] `#![deny(clippy::unwrap_used)]` — zero panics on user input
- [ ] Single binary, ARM64 + x86_64 + Apple Silicon

**Exit criteria:** Migrate the PiCloud PRD and ADRs. Confirm feature→ADR links manually. Run `product migrate link-tests` — ≥ 20 new TC→Feature links inferred. Run `product graph check --format json` — zero errors, W002 count reduced by ≥ 50% vs pre-link-tests. Run `product feature next` with a phase-1 exit-criteria TC marked failing — assert it returns a phase-1 feature, not a phase-2 feature, and names the failing TC. Mark that TC passing and re-run — assert a phase-2 feature is now returned. Assemble `product context FT-001 --depth 2` — bundle opens with `⟦Ω:Bundle⟧` header. Feed 20 malformed YAML files — zero panics (TC-P001 passes). All IT-001–IT-011 integration tests pass. All TC-P001–TC-P004 property tests pass with 1000 cases each.

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

**Goal:** The complete idea-to-implementation loop is supported end-to-end. Product provides every knowledge primitive needed. Agents are invoked by the developer's harness. Phone access works. Domain coverage is enforced at scale.

- [ ] `product mcp` — stdio transport, full tool registry
- [ ] `product mcp --http` — HTTP Streamable transport, bearer token auth
- [ ] `.mcp.json` scaffolding via `product install-hooks`
- [ ] CORS configuration for claude.ai access
- [ ] `product prompts init/list/get/update` — versioned system prompt file management
- [ ] `product adr review --staged` — pre-commit structural and LLM review
- [ ] `product install-hooks` — pre-commit hook installation
- [ ] `[domains]` vocabulary in `product.toml` — controlled concern domain registry
- [ ] `domains` and `scope` fields on ADR front-matter
- [ ] `domains` and `domains-acknowledged` fields on feature front-matter
- [ ] `product preflight FT-XXX` — domain coverage check
- [ ] `product feature acknowledge` — acknowledge domain gaps with reasoning
- [ ] `product graph coverage` — feature × domain coverage matrix
- [ ] `product graph coverage --domain D` and `--format json`
- [ ] `product verify FT-XXX` — TC runner protocol, status update, checklist regeneration
- [ ] `product verify --platform` — cross-cutting TC execution
- [ ] TC front-matter `runner`, `runner-args`, `runner-timeout`, `requires` fields
- [ ] `[verify.prerequisites]` in `product.toml` — named prerequisite conditions
- [ ] `product drift check` — spec-vs-implementation LLM analysis
- [ ] `product drift scan` — reverse: code → governing ADRs
- [ ] `drift.json` baseline, suppression lifecycle
- [ ] `product metrics record` — snapshot to `metrics.jsonl`
- [ ] `product metrics threshold` — CI gate on architectural fitness
- [ ] `product metrics trend` — ASCII sparkline
- [ ] `scripts/harness/implement.sh` — example implementation harness (not part of Product)
- [ ] `scripts/harness/author.sh` — example authoring harness (not part of Product)
- [ ] `benchmarks/prompts/` — default versioned system prompts

**Exit criteria:** From a phone via claude.ai with Product MCP HTTP running: add a new feature with `domains: [security, networking]`, link an ADR, link a TC — all via MCP tool calls. Files appear on disk. Run `product preflight` — security domain flagged. Acknowledge with reasoning. Run `product context FT-XXX --measure` — bundle assembled and measured. Run `scripts/harness/implement.sh FT-XXX` — preflight passes, gap check passes, context file written, agent prompt printed. Run `product verify` manually after agent work — TC executes, status updated, checklist regenerated. Run `product graph coverage` — coverage matrix renders with correct ✓/~ symbols. Run `product metrics record && product metrics threshold` — thresholds pass.

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
| `product_graph_coverage` | `product graph coverage` |
| `product_impact` | `product impact ADR-XXX` |
| `product_preflight` | `product preflight FT-XXX` |
| `product_gap_check` | `product gap check ADR-XXX` |
| `product_adr_review` | `product adr review ADR-XXX` |
| `product_dep_list` | `product dep list` |
| `product_dep_show` | `product dep show DEP-XXX` |
| `product_dep_check` | `product dep check DEP-XXX` |
| `product_dep_bom` | `product dep bom` |
| `product_metrics_stats` | `product metrics stats` |
| `product_prompts_list` | `product prompts list` |
| `product_prompts_get` | `product prompts get PROMPT-NAME` |

**Write tools (require `mcp.write = true` in product.toml):**

| Tool | Equivalent CLI |
|---|---|
| `product_feature_new` | `product feature new "title"` |
| `product_adr_new` | `product adr new "title"` |
| `product_test_new` | `product test new "title" --type TYPE` |
| `product_dep_new` | `product dep new "title" --type TYPE` |
| `product_feature_link` | `product feature link FT-XXX --adr ADR-XXX` |
| `product_feature_acknowledge` | `product feature acknowledge FT-XXX --domain D --reason "..."` |
| `product_adr_status` | `product adr status ADR-XXX accepted` |
| `product_adr_amend` | `product adr amend ADR-XXX --reason "..."` |
| `product_hash_seal` | `product hash seal TC-XXX` |
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

## 16. Authoring Resources

Product does not start agent sessions. It provides the resources agents need to author specifications graph-awarly: versioned system prompt files and fast pre-commit feedback.

### System Prompts

Versioned prompt files in `benchmarks/prompts/`. These are plain markdown files — they work in any LLM interface. Product does not interpret them; it manages their versioning and exposes them via MCP and CLI.

```
benchmarks/prompts/
  author-feature-v1.md     # new feature authoring
  author-adr-v1.md         # new architectural decision authoring
  author-review-v1.md      # spec gardening / coverage improvement
  implement-v1.md          # implementation context template
```

```
product prompts init                  # scaffold default prompt files
product prompts list                  # list available prompts and versions
product prompts get author-feature    # print prompt content to stdout
product prompts update author-feature # pull latest version
```

### How Agents Use Prompts

**Claude Code (local, stdio MCP):**
Product's `.mcp.json` is in the repo. Claude Code connects automatically. Developer loads the prompt as a system prompt or slash command.

**claude.ai Project (remote, HTTP MCP):**
Paste `author-feature-v1.md` contents into the Project's instruction field once. Every conversation in that Project is automatically graph-aware — `product_feature_list`, `product_graph_central`, `product_context` are available as tools. The phone workflow is: open claude.ai, start a conversation, describe what you want to build. Product MCP handles the graph access.

**Any other agent:**
```bash
# Pipe prompt to any agent
product prompts get implement | my-agent --stdin

# Or write to a file for agents that accept file paths
product prompts get author-feature > /tmp/system-prompt.md
my-agent --system-prompt /tmp/system-prompt.md
```

### Pre-Commit Review

`product install-hooks` installs a pre-commit hook that runs `product adr review --staged`. This is fast structural feedback, not an agent session.

**Structural checks (local, instant, no LLM):** required sections, status field, feature links, TC links, evidence blocks.

**LLM check (single call, ~3 seconds):** internal consistency, contradiction scan, missing test suggestion.

Advisory — the commit always proceeds. The CI gap analysis gate is the hard enforcement point.

---

## 17. Implementation Pipeline

Product is a knowledge tool. The implementation pipeline is a sequence of Product knowledge commands. The harness — a shell script, a Makefile, a CI pipeline, an agent invoking tools — is the developer's choice.

### The Knowledge Sequence

What any harness calls before and after agent work:

```bash
# Before invoking an agent:
product preflight FT-001                       # domain coverage clean?
product gap check FT-001 --severity high       # no blocking spec gaps?
product drift check --phase 1                  # codebase matches ADRs?
product context FT-001 --depth 2 --measure    # assemble context bundle

# After the agent completes:
product verify FT-001                          # run TCs, update status
product graph check                            # graph still healthy?
```

Product owns none of the steps between `product context` and `product verify`. That is the agent's domain.

### `product verify FT-XXX`

Reads TC runners from front-matter, executes tests, writes results, regenerates checklist. The one pipeline command Product owns because it writes back to the graph.

```
product verify FT-001
  TC-001 binary-compiles         PASS  (4.1s)
  TC-002 raft-leader-election    PASS  (12.3s)
  TC-003 raft-leader-failover    FAIL  (2.1s)
    "thread 'raft_leader_failover' panicked at..."
  
  1/3 passing. Feature status: in-progress.
  checklist.md regenerated.
```

`product verify --platform` runs all TCs linked to cross-cutting ADRs — the platform-wide invariant check, separate from any feature's verify step.

### Example Harness Scripts

`scripts/harness/` contains reference shell scripts. These are **not part of Product** — they are examples a developer can copy and adapt. They demonstrate how the knowledge commands compose.

**`scripts/harness/implement.sh`:**
```bash
#!/usr/bin/env bash
# Example implementation harness. NOT part of Product. Copy and modify.
set -euo pipefail
FEATURE=${1:?Usage: implement.sh FT-XXX}

product preflight "$FEATURE" || exit 1
product gap check "$FEATURE" --severity high --format json \
  | jq -e '.findings | length == 0' > /dev/null || exit 1
product drift check --phase "$(product feature show "$FEATURE" --field phase)"

BUNDLE=$(mktemp /tmp/product-context-XXXX.md)
product context "$FEATURE" --depth 2 --measure > "$BUNDLE"

# ↓ Replace with your agent of choice
echo "Context ready: $BUNDLE"
echo "Run your agent, then: product verify $FEATURE"
```

**`scripts/harness/author.sh`:**
```bash
#!/usr/bin/env bash
# Example authoring harness. NOT part of Product. Copy and modify.
set -euo pipefail
SESSION=${1:?Usage: author.sh feature|adr|review}

PROMPT=$(product prompts get "author-$SESSION")
echo "Prompt loaded. Start your agent with Product MCP connected."
echo "Paste this prompt as your first message or system instruction:"
echo "---"
echo "$PROMPT"
```

The scripts make the composition explicit. Developers who want a tighter loop write their own. Product's job is to make each command in the sequence reliable and composable.

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
