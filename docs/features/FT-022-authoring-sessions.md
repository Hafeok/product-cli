---
id: FT-022
title: Authoring Sessions
phase: 5
status: complete
depends-on: []
adrs:
- ADR-020
- ADR-022
tests:
- TC-116
- TC-117
- TC-118
- TC-119
- TC-120
- TC-166
- TC-315
- TC-316
- TC-317
- TC-321
- TC-322
- TC-323
- TC-324
domains:
- api
domains-acknowledged:
  ADR-041: Pre-dates ADR-041; this feature does not author absence TCs or set removes/deprecates on ADRs. FT-047 owns the mechanics.
  ADR-042: Pre-dates ADR-042; this feature does not define TC types or validate the type vocabulary. FT-048 owns the mechanics.
  ADR-043: Predates ADR-043; feature command adapters were written before the slice+adapter pattern was formalised and are not retroactively refactored.
  ADR-040: predates ADR-040; feature ships independently of the unified verify / LLM-boundary pipeline
  ADR-018: Predates the 2026-04-22 scope promotion of ADR-018 to cross-cutting. Test coverage reflects the property/session/benchmark strategy as it existed when this feature shipped; not retroactively reclassified.
  ADR-048: Predates ADR-048; this feature does not author the canonical .product/ folder layout. FT-057 owns the migration command and discovery fallback.
  ADR-047: Predates ADR-047; this feature does not author the functional-spec body convention. FT-055 owns the structural validator and W030 mechanics.
---

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

---

## Description

Authoring sessions are structured interactions between a developer and an LLM agent (typically Claude Code or claude.ai) in which the agent has access to the Product MCP tool surface and is guided by a versioned system prompt. Product owns the system prompts (stored in `benchmarks/prompts/`) and the pre-commit review command (`product adr review --staged`). Product does not invoke agents — it provides the knowledge resources agents need (ADR-022). Three session types are supported: `feature` (adding new capability), `adr` (adding an architectural decision), and `review` (spec gardening). The phone workflow — authoring via claude.ai connected to a running `product mcp --http` server — is a first-class supported path.

## Functional Specification

### Inputs

- **Session type**: one of `feature`, `adr`, `review` — determines which versioned system prompt is loaded
- **System prompt files**: stored at `benchmarks/prompts/author-{type}-v{N}.md`; version configured per type in `product.toml` under `[author]`
- **`product prompts init`**: scaffolds default prompt files if absent
- **`product prompts get TYPE`**: prints prompt content to stdout for piping to any agent
- **`product install-hooks`**: writes `.git/hooks/pre-commit` to run `product adr review --staged` before each commit
- **Staged ADR files**: read by `product adr review --staged` via `git diff --cached --name-only`
- **`product.toml` `[author]` section**: configures prompt versions and agent name

### Outputs

- **`benchmarks/prompts/` files**: versioned markdown prompt files readable by any agent platform
- **`.git/hooks/pre-commit`**: executable hook script installed by `product install-hooks`
- **`product adr review --staged` output**: rustc-style diagnostic messages (ADR-013) on stdout for structural findings and LLM-assisted consistency findings; advisory only (exits 0 regardless of findings)
- **`product prompts list`**: table of available prompts and their versions
- **`product prompts get TYPE`**: prompt content to stdout (for piping)

### State

- Versioned prompt files in `benchmarks/prompts/` are the persistent state for authoring sessions. They are version-controlled alongside the graph.
- The pre-commit hook is a file in `.git/hooks/pre-commit`. Its presence is checked by `product install-hooks` before overwriting.
- No session state is maintained between invocations — each `product adr review --staged` call reads staged files fresh.

### Behaviour

1. `product prompts init` creates default prompt files if they do not exist. It does not overwrite existing files.
2. `product prompts get TYPE` prints the configured version of the prompt to stdout. Piping to an agent (`product prompts get author-feature | my-agent`) is the intended composition.
3. `product install-hooks` writes `.git/hooks/pre-commit`. The hook runs `product adr review --staged` for any staged ADR files. It always exits 0 — the hook is advisory.
4. `product adr review --staged` performs two passes: (a) structural checks (local, instant, no LLM) — required sections, status field, at least one linked feature and TC, evidence blocks on invariant blocks; (b) LLM review (single call, ~3s) — internal consistency, contradiction with linked ADRs, missing test suggestions.
5. In a claude.ai Project configured with the `author-feature-v1.md` content as project instructions and the Product HTTP MCP server as a connector, every new conversation is automatically a graph-aware authoring session — no CLI command required.

### Invariants

- Product does not invoke agents. It provides prompts and review commands; agent invocation is the harness's responsibility (ADR-022).
- System prompts are user-modifiable files, not embedded in the binary. A Product upgrade never silently changes an authoring session's behaviour.
- `product adr review --staged` always exits 0. It never blocks a commit — the CI gap analysis gate is the enforcement point.
- Prompt versions are independent per session type. Incrementing `feature-prompt-version` does not affect `adr-prompt-version`.

### Error handling

- If a prompt file referenced by `product.toml` does not exist, `product prompts get TYPE` exits 1 with an E-class message suggesting `product prompts init`.
- If the LLM call in `product adr review --staged` fails (network error, timeout), structural findings are still printed and the hook exits 0. The LLM failure is noted on stderr as advisory.
- `product prompts update TYPE` bumps to the latest built-in version; if the file has local modifications, the command warns and requires `--force` to overwrite.

### Boundaries

- Product owns the prompt files and the pre-commit review command. It does not own agent lifecycle, agent invocation, or what happens between prompt delivery and the agent producing output.
- The pre-commit hook is advisory. Enforcement of specification quality is via CI gap analysis (`product gap check --changed`), not the hook.
- `product adr review --staged` reviews only staged ADR files. It does not review feature files, TC files, or unstaged changes.

## Out of scope

- Agent invocation (`product author feature` as a CLI command that starts Claude Code — rejected, see ADR-022)
- Prompts embedded in the binary (user-modifiable files in the repo are the correct design)
- Blocking pre-commit hooks that prevent commits on spec quality grounds
- Session state or conversational history between `product adr review` invocations
- LLM-driven auto-fix of structural ADR issues (review is read-only and advisory)
