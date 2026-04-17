# Product CLI

Knowledge graph CLI for managing features, ADRs, and test criteria. Built for LLM-driven development — assembles precise context bundles from a structured artifact graph.

## Install

```bash
cargo install --path .
```

## Getting Started with `product init`

`product init` scaffolds a repository end-to-end: it writes `product.toml`, creates the artifact directories (`docs/features/`, `docs/adrs/`, `docs/tests/`, `docs/deps/`), drops in starter authoring prompts under `benchmarks/prompts/`, configures `.mcp.json` for Claude Code, and appends the necessary entries to `.gitignore`.

### Interactive

```bash
cd my-project
product init
```

You'll be prompted for project name, domain vocabulary, and MCP port. Domains are the taxonomy your features and ADRs will be classified under — pick the ones that actually divide your codebase (e.g. `api`, `storage`, `security`, `observability`).

### Non-interactive

```bash
product init -y --name my-project \
  --domain api="HTTP surface, request handling" \
  --domain storage="Persistence layer, migrations" \
  --domain security="Auth, secrets, trust boundaries" \
  --write-tools
```

Flags:
- `-y, --yes` — accept defaults, no prompts
- `--name NAME` — project name (defaults to directory name)
- `--domain K=V` — repeatable; key is the domain identifier, value is a short description
- `--port PORT` — MCP HTTP port (default `7777`)
- `--write-tools` — enable MCP write tools so agents can mutate the graph via `product_request_apply`
- `--force` — overwrite an existing `product.toml`
- `--path DIR` — target a directory other than CWD

### After init

```bash
product status           # dashboard — empty at first
product graph check      # should be clean
product prompts list     # authoring prompts Claude Code will use
```

## Creating artifacts — the request interface (FT-041)

The canonical way to create and link artifacts is a **request**: a YAML document describing a multi-artifact, atomic mutation to the graph. One request = one validation pass = one atomic write = one audit record.

### Scaffold and apply your first request

```bash
product request create              # opens $EDITOR with a template in .product/requests/
product request validate FILE       # dry-run validation, reports every finding at once
product request apply FILE          # atomically writes every file, assigns IDs
product request apply FILE --commit # apply then create a git commit with reason as message
product request diff FILE           # show what would change without writing
product request draft               # list saved drafts
```

### Example: create a feature with its ADR and a test, all linked, in one step

```yaml
type: create
reason: "Add rate limiting to the resource API"
artifacts:
  - type: feature
    ref: ft-rate-limiting
    title: Rate Limiting
    phase: 2
    domains: [api, security]
    adrs: [ref:adr-token-bucket]
    tests: [ref:tc-rate-limit]

  - type: adr
    ref: adr-token-bucket
    title: Token bucket algorithm for rate limiting
    domains: [api]
    scope: domain
    features: [ref:ft-rate-limiting]

  - type: tc
    ref: tc-rate-limit
    title: Rate limit enforced at 100 req/s
    tc-type: scenario
    validates:
      features: [ref:ft-rate-limiting]
      adrs: [ref:adr-token-bucket]
```

`ref:` values are forward references scoped to the request — Product topologically sorts the artifacts, assigns real IDs (`FT-009`, `ADR-031`, `TC-050`), rewrites every `ref:` occurrence on write, and materialises bidirectional cross-links automatically.

### The three request types

| Type | Use |
|---|---|
| `create` | New artifacts that don't exist yet |
| `change` | Mutations to existing artifacts (`set`/`append`/`remove`/`delete` on any field, dot-notation for nested) |
| `create-and-change` | Both in one atomic operation |

### Invariants

- Validation reports **every** finding in one pass, not just the first
- `reason:` is mandatory — missing or whitespace-only is `E011`
- A failed apply leaves **zero** files changed (verified by pre/post SHA-256 checksums)
- A successful apply never produces `graph check` exit 1
- Each apply appends one line to `.product/request-log.jsonl` with timestamp, reason, request hash, and assigned IDs

Full spec: [`docs/product-request-spec.md`](docs/product-request-spec.md). Pinned decisions: [ADR-038](docs/adrs/ADR-038-product-request-unified-atomic-write-interface.md).

### When to use granular commands instead

For trivial single-field edits, the granular commands remain available and are cheaper to type:

```bash
product feature new "User Auth" --phase 1
product feature link FT-001 --adr ADR-001 --test TC-001
product feature domain FT-001 --add security
product adr status ADR-001 --set accepted
```

Requests are the right interface whenever intent spans more than one artifact or more than one field.

## Navigating the graph

```bash
product feature list
product context FT-001 --depth 2     # context bundle for LLM
product graph check                   # validate graph health
product graph central --top 5         # most important ADRs by betweenness
product impact ADR-001                # what depends on this decision
product preflight FT-001              # domain coverage check before implementing
product status                        # project dashboard
```

## Commands

| Command | Purpose |
|---|---|
| `init` | Scaffold a new Product repository |
| `request create/change/validate/apply/diff/draft` | Unified atomic write interface (FT-041) |
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

## Architecture

- Single Rust binary, no runtime dependencies
- In-memory graph rebuilt from YAML front-matter on every invocation (ADR-003)
- Embedded Oxigraph for SPARQL queries (ADR-008)
- Betweenness centrality for ADR importance ranking (ADR-012)
- Atomic file writes with advisory locking (ADR-015)
- `#![deny(clippy::unwrap_used)]` — zero panics on user input

## Tests

```bash
cargo test                # 539 tests (147 unit + 369 integration + 10 property + 13 code quality)
cargo bench               # 4 benchmarks (parse, centrality, impact, BFS)
cargo clippy -- -D warnings -D clippy::unwrap_used
```

## Documentation

- [Product PRD](docs/product-prd.md)
- [Product ADRs](docs/product-adrs.md)
- [Feature Checklist](CHECKLIST.md)
