## Overview

Authoring sessions give Claude (or another configured agent) full read access to your knowledge graph before it writes any specification content. The `product author` command starts an agent session with a versioned system prompt that enforces a mandatory read phase — Claude must call Product MCP tools to understand existing features, ADRs, and dependencies before proposing new artifacts. This prevents specifications that are internally consistent but externally inconsistent with the rest of the graph. Sessions can run locally via `product author` or remotely via claude.ai with the Product MCP HTTP server.

## Tutorial

### Your first feature authoring session

This walkthrough creates a new feature using `product author feature`, which starts Claude Code with the Product MCP server and a system prompt that guides the authoring process.

1. Start the authoring session:

   ```bash
   product author feature
   ```

2. Claude Code launches with the feature authoring prompt pre-loaded. Before asking you anything, Claude will:
   - Call `product_feature_list` to see what features already exist
   - Call `product_graph_central` to identify foundational ADRs
   - Call `product_context` on related features

3. Claude asks you clarifying questions grounded in what the graph says. Describe the feature you want — for example, "Add a rate limiting feature."

4. Claude scaffolds the feature file, links dependencies, writes ADRs and test criteria.

5. When you signal "done," Claude runs `product_graph_check` and `product_gap_check` on the new artifacts to verify structural integrity.

6. The session ends. Review the changes on disk, then commit:

   ```bash
   git add docs/features/ docs/adrs/ docs/tests/
   git commit -m "feat: add rate limiting feature"
   ```

### Installing the pre-commit hook

The pre-commit hook provides advisory feedback on staged ADRs before each commit.

1. Install the hook:

   ```bash
   product install-hooks
   ```

2. Verify the hook exists and is executable:

   ```bash
   ls -la .git/hooks/pre-commit
   ```

3. Now, whenever you commit staged ADR files, `product adr review --staged` runs automatically and prints any structural findings. The commit always proceeds (exit code 0) — findings are advisory.

## How-to Guide

### Start a feature authoring session

1. Run `product author feature`.
2. Describe the feature to Claude when prompted.
3. Let Claude read the graph, ask questions, and scaffold files.
4. Signal "done" when satisfied. Claude validates the graph before ending.
5. Review and commit the generated files.

### Start an ADR authoring session

1. Run `product author adr`.
2. Claude reads the top-5 ADRs by centrality and checks existing decisions.
3. Claude calls `product_impact` on affected areas to understand blast radius.
4. Describe the decision. Claude drafts the ADR with all five required sections: Context, Decision, Rationale, Rejected alternatives, and Test coverage.
5. Claude runs `product_adr_review` on the draft before finishing.

### Run a specification review session

1. Run `product author review`.
2. Claude fixes structural issues (`product_graph_check`), identifies weak metrics (`product_metrics_stats`), and walks features with low phi scores.
3. Claude proposes formal blocks, feature links for orphaned ADRs, and exit-criteria TCs for uncovered features.
4. Review the summary of improvements and remaining work.

### Set up phone-based authoring via HTTP MCP

1. Start the MCP server on your desktop or server:

   ```bash
   product mcp --http
   ```

2. In claude.ai, create a Project and add the Product MCP server as a connector.

3. Add the authoring system prompt as a Project instruction set:

   ```markdown
   # Product Authoring System Prompt

   You have access to a Product MCP server for the [project-name] repository.

   Before writing any specification content:
   1. Call product_feature_list
   2. Call product_graph_central
   3. Call product_context on related features
   ```

4. Start a conversation in that Project from your phone. Claude has the same graph-aware authoring behaviour as `product author`.

5. Files land in your repo via the HTTP MCP server. Later, at your desktop:

   ```bash
   git pull && product implement FT-XXX
   ```

### Review staged ADRs manually

1. Stage your ADR files:

   ```bash
   git add docs/adrs/ADR-027-*.md
   ```

2. Run the review:

   ```bash
   product adr review --staged
   ```

3. Address any structural or consistency findings printed to stdout.

## Reference

### Commands

#### `product author <session-type>`

Starts an agent session with a versioned system prompt and Product MCP active.

| Argument | Values | Description |
|----------|--------|-------------|
| `session-type` | `feature`, `adr`, `review` | The type of authoring session to start |

Each session type loads a system prompt from `benchmarks/prompts/`:

| Session type | Prompt file |
|--------------|-------------|
| `feature` | `benchmarks/prompts/author-feature-v1.md` |
| `adr` | `benchmarks/prompts/author-adr-v1.md` |
| `review` | `benchmarks/prompts/author-review-v1.md` |

#### `product install-hooks`

Installs a pre-commit hook at `.git/hooks/pre-commit`. The hook runs `product adr review --staged` on any staged ADR files. Always exits 0 (advisory).

#### `product adr review --staged`

Reviews staged ADR files for structural and consistency issues.

**Structural checks (local, instant):**

- All five required sections present (Context, Decision, Rationale, Rejected alternatives, Test coverage)
- `status` field is set and valid
- At least one entry in `features` front-matter
- At least one entry in `validates` (TC linked)
- Evidence blocks present on any `⟦Γ:Invariants⟧` blocks

**LLM review (single call):**

- Internal consistency: does the rationale support the decision?
- Contradiction scan: compare against linked ADRs' decisions
- Missing test suggestion: given the claims, what TCs are absent?

Output format follows ADR-013 rustc-style diagnostics.

### Configuration

In `product.toml`:

```toml
[author]
feature-prompt-version = "1"
adr-prompt-version = "1"
review-prompt-version = "1"
agent = "claude-code"
```

| Key | Default | Description |
|-----|---------|-------------|
| `feature-prompt-version` | `"1"` | Version suffix for the feature session prompt file |
| `adr-prompt-version` | `"1"` | Version suffix for the ADR session prompt file |
| `review-prompt-version` | `"1"` | Version suffix for the review session prompt file |
| `agent` | `"claude-code"` | Agent to invoke for authoring sessions |

### Pre-commit hook

The installed hook script:

```bash
#!/bin/sh
# Installed by: product install-hooks
STAGED_ADRS=$(git diff --cached --name-only | grep "^docs/adrs/")
if [ -n "$STAGED_ADRS" ]; then
    echo "Running product adr review on staged ADRs..."
    product adr review --staged
fi
exit 0
```

The hook only runs `adr review` when ADR files are staged. Non-ADR files (features, tests, source code) do not trigger the review.

## Explanation

### Why a mandatory read phase?

The core design insight behind authoring sessions is the mandatory read phase enforced by each session type's system prompt. Without it, Claude produces specifications in isolation — internally consistent but disconnected from the existing decision landscape. With it, Claude reads foundational ADRs, understands what features exist, and checks for contradictions before writing anything. This is the difference between a documentation tool and a graph-aware authoring tool (ADR-022).

### Why are pre-commit findings advisory?

The pre-commit hook always exits 0 regardless of findings. Blocking commits creates friction that causes developers to bypass the hook entirely (e.g., `git commit --no-verify`). Fast feedback at commit time is more valuable than a hard gate. The CI gap analysis gate (`product gap check`) serves as the hard enforcement point where structural issues must be resolved.

### Phone workflow and the shared-state model

The phone workflow separates authoring from implementation. Authoring happens wherever you are — on your phone via claude.ai with the HTTP MCP server running on your desktop. Implementation happens at your desktop with `product implement`. The repository is the shared state between these environments. This separation works because `product author` sessions only produce specification artifacts (feature files, ADRs, test criteria), never source code.

### System prompt versioning

System prompts are versioned files (`author-feature-v1.md`, etc.) rather than inline strings. This allows prompt evolution to be tracked in git history, prompt versions to be pinned per-project in `product.toml`, and different projects to use different prompt versions. When a prompt improves, bump the version and update the configuration — old prompts remain available for comparison and rollback.

### Why Product never auto-commits

An earlier design considered auto-committing changes when an authoring session ends. This was rejected because the developer must review generated specifications before they enter version control. Product writes files to disk but never commits to git — that boundary is intentional and consistent across all Product commands (ADR-022).
