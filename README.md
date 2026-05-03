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

## Install

Pick whichever fits your environment. None require a Rust toolchain except option 2.

**1. Prebuilt binary (recommended)** — works on macOS, Linux, Windows:

```bash
# macOS / Linux
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/Hafeok/product-cli/releases/latest/download/product-installer.sh | sh

# Windows (PowerShell)
irm https://github.com/Hafeok/product-cli/releases/latest/download/product-installer.ps1 | iex
```

The script drops `product` into `~/.cargo/bin` (or `%CARGO_HOME%\bin` on Windows). Add that to your `PATH` if it isn't already.

**2. From source (if you have Rust)**:

```bash
cargo install --git https://github.com/Hafeok/product-cli
```

**3. Via Dagger** — for hermetic CI use, no install on the runner:

```bash
dagger -m github.com/Hafeok/product-cli call binary export --path ./product
```

Verify any of the above with:

```bash
product --version    # → product 0.1.0
```

---

## 60-second quickstart

```bash
# 1. scaffold a project (anywhere)
mkdir my-app && cd my-app
product init -y --name my-app \
  --domain api="HTTP surface" \
  --domain storage="Persistence"

# 2. create a feature + its ADR + a test, all linked, in one atomic write
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

# 3. ask the graph what you'd hand to an LLM to implement this
product context FT-001 --depth 2
```

Step 3 prints a single self-contained markdown document with the feature, the ADR that governs it, and the test that validates it — sized for an LLM context window, deterministic, and free of unrelated noise. That bundle is the entire point of the tool.

> Don't want to write the YAML yourself? Skip to **Author with Claude** below — `product author feature` does the same thing through a guided conversation.

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

## Author with Claude

Writing well-formed features by hand is tedious — you have to remember which ADRs are relevant, link the right tests, pick the right phase, and not duplicate something that already exists. `product author feature` makes that someone else's problem.

```bash
product author feature
```

What this does:

1. Spawns Claude Code (or whatever agent you configured in `[author]` of `product.toml`) with a versioned authoring system prompt pre-loaded.
2. Connects the Product MCP server so Claude has **full read access to your graph from the first message**.
3. Before proposing anything, Claude calls `product_feature_list`, `product_graph_central`, and `product_context` on related features — so its proposal is grounded in what already exists, not invented.
4. You describe what you want in plain English. Claude asks clarifying questions tied to existing decisions ("ADR-012 already governs rate limiting via token bucket — should this feature inherit that or override?").
5. When you agree on the shape, Claude scaffolds the feature file, drafts any new ADRs and TCs, and links everything bidirectionally — typically as a single `product request apply` so the write is atomic.
6. Before exiting, it runs `product graph check` and `product gap check` so you don't end up with broken links or untested features.

```bash
product author feature                    # open-ended: "I want to add X"
product author feature --feature FT-007   # extend an existing feature; gates on preflight
product author adr                        # for a pure decision (no new capability)
product author review                     # spec gardening — fix orphans, weak metrics, missing TCs
```

Three useful properties:

- **It cannot hallucinate IDs.** Claude scaffolds via the request interface, which assigns real IDs only on apply. No ghost references to features that don't exist.
- **It reads before it writes.** The system prompt forces graph reads before scaffolding, so the proposal isn't a duplicate of something you wrote three weeks ago.
- **It works from your phone.** If `product mcp --http` is running on your dev box or a server, the same authoring flow runs in any claude.ai conversation that has the Product MCP server configured. Author a feature on the train; verify it on a laptop.

If you don't have Claude Code installed, point `[author].cli` at `copilot` in `product.toml`, or stick with the `product request` flow shown in the quickstart.

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

Product ships as a [Dagger](https://dagger.io/) module. If you already use Dagger for CI, this gives you a hermetic, no-install path to running graph checks, assembling context bundles, or shipping the binary into a downstream container — without ever putting `product` on the runner image.

### Why through Dagger

- **Zero-install CI.** Your runner doesn't need a Rust toolchain or even `curl`. Dagger pulls the prebuilt binary from the GitHub Release and caches it.
- **Pinned and reproducible.** `--version=v0.1.0` locks the binary; the pipeline gives the same result on a laptop and in CI.
- **Composable.** `dag.Product().Container()` returns a container with `product` on PATH that you can chain into your own pipelines.

### Functions

```bash
dagger -m github.com/Hafeok/product-cli functions
```

| Function | What it does |
|---|---|
| `binary --version --platform` | Returns the `product` binary as a `*File`. Default version is `latest`, default platform is `linux/amd64`. |
| `container --version --platform` | Debian slim with `product` on PATH and as the entrypoint. |
| `validate --source --version` | Runs `product graph check` against a directory containing `product.toml`. Fails the pipeline on any graph error — perfect CI gate. |
| `context --source --feature --depth --version` | Assembles an LLM context bundle for a feature inside a sandbox; returns the markdown as a string. |

### Common one-liners

```bash
# Drop the binary on disk
dagger -m github.com/Hafeok/product-cli call binary export --path ./product

# Specific version + platform
dagger -m github.com/Hafeok/product-cli call binary \
  --version=v0.1.0 --platform=darwin/arm64 \
  export --path ./product

# Fail CI if the graph is broken
dagger -m github.com/Hafeok/product-cli call validate --source=.

# Pipe a context bundle into a downstream tool
dagger -m github.com/Hafeok/product-cli call context \
  --source=. --feature=FT-007 --depth=2 > bundle.md
```

### GitHub Actions example

```yaml
- uses: dagger/dagger-for-github@v6
  with:
    verb: call
    module: github.com/Hafeok/product-cli
    args: validate --source=.
```

That's the entire CI gate. No setup-rust, no cargo install, no version drift between local and CI.

### Local development of the module

If you're iterating on the module itself (in `dagger/`):

```bash
cd dagger && dagger develop          # regenerate the SDK after editing main.go
dagger -m . functions                # list functions
dagger -m . call binary export ...   # test against the latest GitHub Release
```

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

## Path scoping

`product` locates the graph it operates on by, in priority order:

1. **`--root <path>`** — top-level flag, accepted before or after the
   subcommand. Highest priority; use for one-off scripting.
2. **`PRODUCT_ROOT` env var** — session-level override. Use to scope an
   entire shell or container at a single graph. Empty values are ignored.
3. **Walk-up from the current directory** — the default, unchanged. Picks
   the nearest ancestor that contains a `.product/` directory or
   `product.toml`.

When `--root` and `PRODUCT_ROOT` are both set, `--root` wins.

```bash
product --root crates/verify-cli feature list   # one-off
PRODUCT_ROOT=/workspace/typo-cli product mcp    # whole-session scope
```

Explicit paths are tilde-expanded, resolved against the current directory
when relative, and canonicalized (symlinks followed). Pointing at the
`.product/` directory itself (`--root foo/.product`) is treated as the
parent. The path must exist, be a directory, and contain a `.product/`
subdirectory; otherwise the binary exits with code 24 and an
`error[E024]: graph root not found` diagnostic naming the supplied value
and the source (`flag` or `env`).

The MCP server reads the same resolution at startup and is fixed to the
resolved root for its lifetime — restart the server to point at a
different graph.

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
