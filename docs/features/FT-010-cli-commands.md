---
id: FT-010
title: CLI Commands
phase: 1
status: complete
depends-on: []
adrs:
- ADR-009
- ADR-013
tests:
- TC-027
- TC-028
- TC-029
- TC-030
- TC-055
- TC-056
- TC-057
- TC-058
- TC-059
domains:
- api
- error-handling
domains-acknowledged:
  ADR-040: predates ADR-040; feature ships independently of the unified verify / LLM-boundary pipeline
  ADR-042: Pre-dates ADR-042; this feature does not define TC types or validate the type vocabulary. FT-048 owns the mechanics.
  ADR-043: Predates ADR-043; feature command adapters were written before the slice+adapter pattern was formalised and are not retroactively refactored.
  ADR-041: Pre-dates ADR-041; this feature does not author absence TCs or set removes/deprecates on ADRs. FT-047 owns the mechanics.
  ADR-018: Predates the 2026-04-22 scope promotion of ADR-018 to cross-cutting. Test coverage reflects the property/session/benchmark strategy as it existed when this feature shipped; not retroactively reclassified.
---

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