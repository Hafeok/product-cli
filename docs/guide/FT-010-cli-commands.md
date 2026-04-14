## Overview

Product exposes a single `product` binary with subcommands organised around the knowledge graph's three artifact types — features, ADRs, and test criteria. Every command reads YAML front-matter from Markdown files, rebuilds the graph in memory, and either queries it or mutates the source files. The CLI is designed to work equally well in interactive terminals and CI pipelines: results go to stdout, diagnostics go to stderr, and exit codes carry machine-readable semantics (ADR-009).

## Tutorial

### Exploring the knowledge graph

Start by listing what is in the repository:

```bash
product feature list
product adr list
product test list
```

Pick a feature and drill into it:

```bash
product feature show FT-001
```

See which ADRs inform it and which tests validate it:

```bash
product feature adrs FT-001
product feature tests FT-001
```

Check the full dependency tree to understand what FT-001 builds on:

```bash
product feature deps FT-001
```

### Assembling context for an LLM

Generate a context bundle that includes the feature, its linked ADRs, and its test criteria:

```bash
product context FT-001
```

For a richer bundle that follows transitive dependencies, increase the depth:

```bash
product context FT-001 --depth 2
```

Redirect the bundle to a file for use elsewhere:

```bash
product context FT-001 --depth 2 > bundle.md
```

Warnings (if any) appear on stderr and do not pollute the file.

### Checking graph health

Validate all links, cycles, and phase ordering:

```bash
product graph check
```

A clean graph returns exit code 0. If you see errors, the hint lines tell you exactly what to fix.

### Creating new artifacts

Scaffold a new feature, ADR, or test criterion:

```bash
product feature new "Cluster Foundation"
product adr new "Use openraft for consensus"
product test new "Raft leader election" --type scenario
```

Link them together:

```bash
product feature link FT-001 --adr ADR-002
product feature link FT-001 --test TC-002
```

### Running the implementation pipeline

Once a feature is ready for implementation:

```bash
product implement FT-001
```

This runs pre-flight checks, assembles the context bundle, and invokes the configured agent. After implementation, verify the work:

```bash
product verify FT-001
```

`verify` executes linked TC runners, updates test status in front-matter, and regenerates `CHECKLIST.md`.

## How-to Guide

### List features filtered by phase or status

```bash
product feature list --phase 1
product feature list --status complete
product feature list --phase 2 --status draft
```

### Find the next feature to work on

```bash
product feature next
```

Returns the next incomplete feature in topological order (respecting dependency edges, not phase labels).

### Assemble context for an entire phase

```bash
product context --phase 1
product context --phase 1 --adrs-only
```

The `--adrs-only` flag omits test criteria, producing a smaller bundle focused on decisions.

### Control ADR ordering in context bundles

ADRs are ordered by betweenness centrality (most structurally important first) by default. To sort by ID instead:

```bash
product context FT-001 --order id
```

### Understand the impact of changing a decision

```bash
product impact ADR-002
```

This shows every feature, test, and transitive dependent affected if ADR-002 changes. Works for any artifact type:

```bash
product impact FT-001
product impact TC-003
```

### Run graph health checks in CI

```bash
# Fail on errors only, tolerate warnings:
product graph check || [ $? -eq 2 ] && true

# Fail on any issue (errors or warnings):
product graph check

# Machine-readable output for PR annotations:
product graph check --format json
```

### Check domain coverage before implementation

```bash
product preflight FT-001
product preflight FT-001 --format json
```

Resolve gaps by linking an ADR or acknowledging a domain:

```bash
product feature link FT-009 --adr ADR-040
product feature acknowledge FT-009 --domain security \
  --reason "no trust boundaries introduced"
```

### Run gap analysis

```bash
# All ADRs:
product gap check

# Single ADR:
product gap check ADR-002

# CI mode — only changed ADRs plus 1-hop neighbours:
product gap check --changed

# Filter by severity:
product gap check --severity high

# Suppress a known gap:
product gap suppress GAP-ADR002-G001-a3f9 --reason "deferred to phase 2"
```

### Detect spec-vs-code drift

```bash
product drift check ADR-002
product drift check --changed
product drift scan src/consensus/
product drift report
```

### Record and monitor architectural metrics

```bash
product metrics record
product metrics trend
product metrics trend --metric phi
product metrics threshold
product metrics stats
```

### Start the MCP server

```bash
# stdio transport (for Claude Code):
product mcp

# HTTP transport:
product mcp --http --port 8080

# Remote access with authentication:
product mcp --http --bind 0.0.0.0 --token "$SECRET"

# List available MCP tools:
product mcp tools
```

### Migrate from a monolithic PRD or ADR file

```bash
# Dry run — see what would be created:
product migrate validate

# Extract features from a PRD:
product migrate from-prd PRD.md

# Extract ADRs and test criteria from an ADR file:
product migrate from-adrs ADRS.md
```

## Reference

### Global flags

| Flag | Description |
|---|---|
| `--format json` | Emit all output (results, errors, warnings) as JSON |

### Exit codes

| Code | Meaning | Applies to |
|---|---|---|
| 0 | Success / clean graph | All commands |
| 1 | Error (broken links, cycles, parse failures, gaps found) | All commands |
| 2 | Warnings only (orphans, missing coverage) | `graph check`, `gap check` |
| 3 | Internal error (bug in Product) | All commands |

### Command reference

#### Navigation commands

| Command | Description |
|---|---|
| `product feature list [--phase N] [--status STATUS]` | List features, optionally filtered |
| `product feature show FT-XXX` | Show a single feature |
| `product feature adrs FT-XXX` | ADRs linked to a feature |
| `product feature tests FT-XXX` | Test criteria for a feature |
| `product feature deps FT-XXX` | Transitive dependency tree |
| `product feature next` | Next feature in topological order |
| `product adr list [--status STATUS]` | List ADRs, optionally filtered |
| `product adr show ADR-XXX` | Show a single ADR |
| `product adr features ADR-XXX` | Features referencing an ADR |
| `product adr tests ADR-XXX` | Tests validating an ADR |
| `product test list [--phase N] [--type TYPE] [--status STATUS]` | List test criteria |
| `product test show TC-XXX` | Show a single test criterion |
| `product test untested` | Features with no linked tests |

#### Context assembly

| Command | Description |
|---|---|
| `product context FT-XXX` | Feature bundle at depth 1 |
| `product context FT-XXX --depth N` | Transitive bundle at depth N |
| `product context --phase N` | All features in a phase |
| `product context --phase N --adrs-only` | Phase features + ADRs only |
| `product context ADR-XXX` | ADR + linked features + tests |
| `product context --order id` | Sort ADRs by ID instead of centrality |

#### Graph operations

| Command | Description |
|---|---|
| `product graph check` | Validate links, cycles, phase ordering |
| `product graph rebuild` | Regenerate `index.ttl` from front-matter |
| `product graph query "SELECT ..."` | SPARQL query over the graph |
| `product graph stats` | Artifact counts, link density, centrality, phi coverage |
| `product graph central [--top N] [--all]` | ADRs ranked by betweenness centrality |
| `product graph coverage [--domain X] [--format json]` | Feature x domain coverage matrix |
| `product impact ARTIFACT-ID` | Affected set if an artifact changes |

#### Authoring

| Command | Description |
|---|---|
| `product feature new "TITLE"` | Scaffold a new feature file |
| `product adr new "TITLE"` | Scaffold a new ADR file |
| `product test new "TITLE" --type TYPE` | Scaffold a new test criterion |
| `product feature link FT-XXX --adr ADR-XXX` | Link a feature to an ADR |
| `product feature link FT-XXX --test TC-XXX` | Link a feature to a test |
| `product adr status ADR-XXX STATUS` | Set ADR status |
| `product test status TC-XXX STATUS` | Set test status |
| `product feature status FT-XXX STATUS` | Set feature status |
| `product feature acknowledge FT-XXX --domain D --reason "..."` | Acknowledge a domain gap |
| `product feature acknowledge FT-XXX --adr ADR-XXX --reason "..."` | Acknowledge an ADR gap |

#### Orchestration

| Command | Description |
|---|---|
| `product implement FT-XXX [--agent NAME] [--dry-run]` | Run the implementation pipeline |
| `product verify FT-XXX [--tc TC-XXX]` | Execute TC runners and update status |
| `product preflight FT-XXX [--format json]` | Domain coverage pre-flight check |

### Error codes

| Code | Tier | Description |
|---|---|---|
| E001 | Parse | Malformed YAML front-matter |
| E002 | Graph | Broken link — referenced artifact does not exist |
| E003 | Graph | Dependency cycle in `depends-on` DAG |
| E004 | Graph | Supersession cycle in ADR `supersedes` chain |
| E005 | Parse | Invalid artifact ID format |
| E006 | Parse | Missing required front-matter field |
| E007 | Parse | Unknown artifact type in `type` field |
| E008 | Schema | `schema-version` in `product.toml` exceeds binary support |
| E009 | Orchestration | `product implement` blocked by unsuppressed high-severity gaps |
| E010 | Concurrency | Repository locked by another Product process |
| E011 | Domain | Acknowledgement present with empty or missing reasoning |
| E012 | Domain | Domain not present in `product.toml` vocabulary |
| W001 | Validation | Orphaned artifact — no incoming links |
| W002 | Validation | Feature has no linked test criteria |
| W003 | Validation | Feature has no exit-criteria type test |
| W004 | Validation | Invariant/chaos test missing formal block |
| W005 | Validation | Phase label disagrees with dependency order |
| W006 | Validation | Formal block evidence below threshold |
| W007 | Schema | Schema upgrade available |
| I001 | Internal | Unexpected None in graph traversal |
| I002 | Internal | Assertion failure in topological sort |

### Diagnostic output format

Interactive (default) — rustc-style with file path, line number, offending content, and hint:

```
error[E002]: broken link
  --> docs/features/FT-003-rdf-projection.md
   |
 4 | adrs: [ADR-001, ADR-002, ADR-099]
   |                          ^^^^^^^ ADR-099 does not exist
   |
   = hint: create the file with `product adr new` or remove the reference
```

JSON (`--format json`) — structured object on stderr:

```json
{
  "errors": [{"code": "E002", "tier": "graph", "message": "broken link", ...}],
  "warnings": [...],
  "summary": {"errors": 1, "warnings": 2}
}
```

## Explanation

### Output stream discipline

Product follows a strict Unix convention: results go to stdout, diagnostics go to stderr. This means `product context FT-001 > bundle.md` always produces a clean file, even when the graph has warnings. The `--format json` flag switches both streams to JSON, enabling CI tools to parse diagnostics programmatically without screen-scraping.

### Exit code design (ADR-009)

The three-tier exit code scheme (0/1/2) lets CI pipelines express nuanced policies. A strict pipeline fails on any non-zero exit. A lenient pipeline tolerates warnings (exit 2) but still catches hard errors (exit 1). This follows conventions established by `grep` and `clippy`, reducing the learning curve. Exit code 3 is reserved for internal errors (bugs in Product itself), ensuring they are never confused with user-caused failures.

### Error taxonomy (ADR-013)

Errors are classified into four tiers: parse errors (malformed files), graph errors (structural inconsistencies), validation warnings (incomplete but usable state), and internal errors (bugs). This taxonomy prevents the two most common error model failures: conflating bugs with user errors, and treating all user errors identically. The rustc-style diagnostic format — showing file, line, offending content, and remediation hint — was chosen because developers already know how to read it.

### Graph-first architecture

The CLI has no persistent database. Every invocation rebuilds the graph from YAML front-matter in Markdown files. This means the source files are always the single source of truth, and there is no synchronisation problem between a database and the filesystem. The cost is a full scan on each command, but for repositories with hundreds of artifacts this completes in milliseconds.

### Centrality-ordered context bundles

When assembling a context bundle, ADRs are ordered by betweenness centrality — a graph metric that measures how often an ADR lies on the shortest path between other artifacts. High-centrality ADRs are structurally important decisions that connect many parts of the system. Placing them first in the bundle gives LLMs the most important context early. The `--order id` flag provides a deterministic fallback when centrality ordering is not desired.

### Pre-flight and domain coverage

The `product preflight` command checks whether a feature has adequate ADR coverage across all relevant concern domains (security, performance, observability, etc.). This runs automatically as Step 0 of `product implement`, preventing implementation from starting with unaddressed architectural gaps. Gaps can be resolved by linking a domain-specific ADR or by explicitly acknowledging that a domain does not apply, with mandatory reasoning to prevent empty acknowledgements (E011).
