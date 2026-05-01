# Product

**A knowledge graph for LLM-driven development.**

You give Claude (or Cursor, or Codex) too much code and not enough decisions, and it builds the wrong thing. Product fixes the context problem at the root: it manages your features, architectural decisions, and test criteria as a structured graph of markdown files, then assembles the *exact* context bundle an agent needs — feature plus the ADRs that govern it plus the tests that validate it — in one command.

```
                  ┌──────────────────┐
                  │   Feature        │
                  │   FT-007         │
                  └────────┬─────────┘
                           │
              ┌────────────┼────────────┐
              ▼            ▼            ▼
         ADR-012      ADR-019      TC-031, TC-032
        (governs)    (governs)    (validates)

  $ product context FT-007 --depth 2
  → markdown bundle ready to paste into Claude
```

No database. No service. Just markdown with YAML front-matter, a single Rust binary, and an MCP server so agents can drive the graph themselves.

---

## Why you'd want this

- **Your AI agent keeps forgetting decisions you made three weeks ago.** Product makes those decisions first-class, linked, and queryable.
- **Your PRD has drifted from the code.** `product drift check` catches it; `product gap check` finds the spec holes.
- **You're tired of pasting six files into a chat to give context.** `product context FT-XXX` gives you the right six, and only those.
- **You want agents that can read and write the graph.** `product mcp` exposes the whole tool surface to Claude Code, claude.ai mobile, or any MCP client.

If your project has more than one decision worth remembering and more than one feature in flight, this is for you.

---

## 60-second quickstart

```bash
# 1. install (no Rust toolchain required)
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/Hafeok/product-cli/releases/latest/download/product-installer.sh | sh
# Windows: irm https://github.com/Hafeok/product-cli/releases/latest/download/product-installer.ps1 | iex
# Already have cargo? `cargo install --git https://github.com/Hafeok/product-cli` works too.

# 2. scaffold a project (anywhere)
mkdir my-app && cd my-app
product init -y --name my-app \
  --domain api="HTTP surface" \
  --domain storage="Persistence"

# 3. create a feature + its ADR + a test, all linked, in one atomic write
cat > /tmp/req.yaml <<'EOF'
type: create
reason: "Rate limit the public API"
artifacts:
  - type: feature
    ref: ft-rate-limit
    title: Rate Limiting
    phase: 1
    domains: [api]
    adrs: [ref:adr-token-bucket]
    tests: [ref:tc-100rps]
  - type: adr
    ref: adr-token-bucket
    title: Token bucket for rate limiting
    domains: [api]
    scope: domain
  - type: tc
    ref: tc-100rps
    title: Enforced at 100 req/s
    tc-type: scenario
EOF
product request apply /tmp/req.yaml

# 4. ask the graph what you'd hand to an LLM to implement this
product context FT-001 --depth 2
```

Step 4 prints a single self-contained markdown document with the feature, the ADR that governs it, and the test that validates it — sized for an LLM context window, deterministic, and free of unrelated noise. That bundle is the entire point of the tool.

---

## The core loop

Once you have artifacts in the graph, the daily flow is:

```bash
product status                  # what's in flight, what's blocked, what's done
product feature next            # next feature to pick up (graph-derived)
product context FT-007          # bundle to hand to your agent
product implement FT-007        # or let Product orchestrate the agent itself
product verify FT-007           # run the linked TCs and update status
```

`product implement` runs the full pipeline: gap-checks the spec, assembles the bundle, spawns your configured agent (Claude Code by default), then verifies. `product verify` executes each TC's configured runner (e.g. `cargo test`) and writes results back into front-matter.

---

## How it's structured

```
docs/
  features/   FT-001-*.md     ← one feature per file, YAML front-matter declares links
  adrs/       ADR-001-*.md    ← one decision per file
  tests/      TC-001-*.md     ← one test criterion per file
  deps/       DEP-001-*.md    ← external dependencies (libs, services, hardware)
product.toml                   ← repo config (paths, prefixes, domains)
```

Every artifact has YAML front-matter declaring its identity and edges. The graph is *derived* on every invocation — there is no separate index to keep in sync, and `git diff` shows you exactly what the graph changed.

```yaml
---
id: FT-007
title: Rate Limiting
phase: 1
status: in-progress
domains: [api, security]
adrs: [ADR-012]
tests: [TC-031, TC-032]
---
```

---

## Writing to the graph: the request interface

For anything that touches more than one field or more than one artifact, use a **request** — a YAML document describing an atomic, validated mutation:

```bash
product request create              # opens $EDITOR with a template
product request validate FILE       # dry-run, reports every finding in one pass
product request diff FILE           # show what would change
product request apply FILE          # atomic write; assigns IDs; rewrites refs
product request apply FILE --commit # apply and create a git commit
```

`ref:` values inside a request are forward references — Product topo-sorts the artifacts, assigns the real IDs (`FT-009`, `ADR-031`, `TC-050`), rewrites every reference on write, and materialises bidirectional cross-links automatically. A failed apply leaves zero files changed, verified by SHA-256 checksum.

For trivial single-field tweaks the granular commands are fine and shorter to type:

```bash
product feature new "User Auth" --phase 1
product feature link FT-001 --adr ADR-001 --test TC-001
product adr status ADR-001 --set accepted
```

---

## Plug it into your agent

```bash
product mcp           # stdio MCP server — for Claude Code on the desktop
product mcp --http    # HTTP MCP server — for claude.ai, including mobile
```

`product init` writes `.mcp.json` so Claude Code picks up the server automatically. From inside an agent session you can ask things like *"show me what FT-007 depends on"*, *"create a feature for X with these two ADRs"*, or *"implement FT-007"* and the agent calls Product's tools rather than guessing at your code layout.

---

## Health checks

```bash
product graph check        # broken links, dangling refs, status invariants
product gap check          # specification holes (features without tests, etc.)
product drift check        # spec vs implementation divergence
product preflight FT-007   # domain coverage check before implementing
product impact ADR-012     # what does changing this decision affect?
```

Wire them into pre-commit or CI and your specs stop rotting.

---

## Use it from Dagger

Product ships as a [Dagger](https://dagger.io/) module so you can drop it into any pipeline without installing it on the runner:

```bash
# Drop the binary on disk
dagger -m github.com/Hafeok/product-cli call binary export --path ./product

# One-liner CI gate: fail the pipeline if your graph is broken
dagger -m github.com/Hafeok/product-cli call validate --source=.

# Assemble a context bundle inside a sandbox
dagger -m github.com/Hafeok/product-cli call context --source=. --feature=FT-007
```

The module exposes `binary`, `container`, `validate`, and `context` functions. See [`dagger/main.go`](dagger/main.go) for signatures. Pin a specific release with `--version=v0.1.0`; defaults to `latest`.

---

## Command reference

| Group | What it covers |
|---|---|
| `init` | Scaffold a new Product repository |
| `request *` | Unified atomic write interface — create / change / validate / apply / diff |
| `feature *` | List, show, navigate, link, update features |
| `adr *` | List, show, link, supersede ADRs |
| `test *` | List, show, run test criteria |
| `dep *` | External dependency artifacts |
| `context FT-XXX` | Assemble an LLM context bundle |
| `graph *` | check / rebuild / query / stats / centrality / autolink |
| `impact ADR-XXX` | Change-impact analysis |
| `status` | Project dashboard |
| `gap *`, `drift *`, `preflight` | Specification health |
| `implement FT-XXX` | Full agent-orchestration pipeline |
| `verify [FT-XXX]` | Run TC runners and update status |
| `author *` | Graph-aware authoring sessions |
| `mcp [--http]` | Run as MCP server |
| `metrics *`, `cycle-times`, `forecast` | Architectural fitness + delivery analytics |
| `onboard`, `migrate` | Bring an existing codebase into the graph |

Run `product <group> --help` for the flags on any of them.

---

## Architecture in one paragraph

Single Rust binary, no runtime deps. The graph is rebuilt in memory from front-matter on every invocation (ADR-003), so it can never drift from the files. Oxigraph powers SPARQL queries (ADR-008). Betweenness centrality ranks ADR importance (ADR-012). All file writes go through atomic write + advisory lock (ADR-015). `#![deny(clippy::unwrap_used)]` — zero panics on user input.

---

## Build & test

```bash
cargo build
cargo t                                              # full suite (alias for --no-fail-fast)
cargo clippy -- -D warnings -D clippy::unwrap_used
cargo bench
```

---

## Docs

- [Product PRD](docs/product-prd.md) — the full vision and goals
- [ADRs](docs/product-adrs.md) — every architectural decision behind this tool
- [Request spec](docs/product-request-spec.md) — the unified atomic write interface
- [Feature checklist](CHECKLIST.md) — current implementation status

## License

See [LICENSE](LICENSE).
