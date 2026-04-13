# Product CLI

Knowledge graph CLI for managing features, ADRs, and test criteria. Built for LLM-driven development — assembles precise context bundles from a structured artifact graph.

## Install

```bash
cargo install --path .
```

## Quick Start

```bash
# Initialize in your repo
cat > product.toml <<EOF
name = "my-project"
schema-version = "1"
[paths]
features = "docs/features"
adrs = "docs/adrs"
tests = "docs/tests"
graph = "docs/graph"
checklist = "docs/checklist.md"
[prefixes]
feature = "FT"
adr = "ADR"
test = "TC"
EOF

# Create artifacts
product feature new "User Authentication" --phase 1
product adr new "JWT for session tokens"
product test new "Token expiry enforcement" --type scenario

# Link them
product feature link FT-001 --adr ADR-001 --test TC-001

# Navigate
product feature list
product context FT-001 --depth 2    # context bundle for LLM
product graph check                  # validate graph health
product graph central --top 5        # most important ADRs
product impact ADR-001               # what depends on this decision
product status                       # project dashboard
```

## Migrate Existing Docs

```bash
product migrate from-adrs my-adrs.md --execute
product migrate from-prd my-prd.md --execute
product graph autolink                # connect TCs to features via ADRs
```

## Commands

| Command | Purpose |
|---|---|
| `feature list/show/adrs/tests/deps/next/new/link/status/acknowledge` | Feature management |
| `adr list/show/features/tests/new/status/review` | ADR management |
| `test list/show/untested/new/status` | Test criteria management |
| `context FT-XXX [--depth N]` | LLM context bundle assembly |
| `graph check/rebuild/query/stats/central/autolink/coverage` | Graph operations |
| `impact ADR-XXX` | Change impact analysis |
| `status` | Project dashboard |
| `checklist generate` | Generate checklist from graph |
| `gap check/report/suppress/unsuppress/stats` | Specification gap analysis |
| `preflight FT-XXX` | Domain coverage pre-flight check |
| `implement FT-XXX [--dry-run]` | Agent orchestration pipeline |
| `verify FT-XXX` | TC runner and status update |
| `author feature/adr/review` | Graph-aware authoring sessions |
| `drift check/scan/suppress/unsuppress` | Spec vs code drift detection |
| `metrics record/threshold/trend` | Architectural fitness functions |
| `mcp [--http]` | MCP server (stdio + HTTP) |
| `completions bash/zsh/fish` | Shell completions |
| `install-hooks` | Git hooks + .mcp.json scaffolding |
| `migrate from-prd/from-adrs/schema` | Document migration |

## Architecture

- Single Rust binary, no runtime dependencies
- In-memory graph rebuilt from YAML front-matter on every invocation (ADR-003)
- Embedded Oxigraph for SPARQL queries (ADR-008)
- Betweenness centrality for ADR importance ranking (ADR-012)
- Atomic file writes with advisory locking (ADR-015)
- `#![deny(clippy::unwrap_used)]` — zero panics on user input

## Tests

```bash
cargo test                # 112 tests (82 unit + 21 integration + 9 property)
cargo bench               # 4 benchmarks (parse, centrality, impact, BFS)
cargo clippy -- -D warnings -D clippy::unwrap_used
```

## Documentation

- [Product PRD](docs/product-prd.md)
- [Product ADRs](docs/product-adrs.md)
- [Feature Checklist](CHECKLIST.md)
