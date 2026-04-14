---
id: ADR-022
title: Authoring Resources — System Prompts and Pre-Commit Review
status: accepted
features: []
supersedes: []
superseded-by: []
domains: []
scope: domain
content-hash: sha256:1e469f937c75e43f66ed41e3e4fcd92c5b2180e91a66a0ad9ba4fcdd766062dd
---

**Status:** Accepted

**Context:** Earlier versions of this ADR described `product author [feature|adr|review]` as CLI commands that started agent sessions. This was identified as a violation of the knowledge boundary established in ADR-021. Product does not invoke agents — it provides the knowledge and resources agents need.

For authoring sessions specifically, three things are needed: a versioned system prompt that tells the agent how to author graph-aware specifications, access to Product's MCP tool surface so the agent can read the graph as it writes, and a fast feedback loop for catching structural issues in draft artifacts before they are committed.

Product owns the system prompts as versioned files in the repository and the pre-commit review command. It does not start agent sessions.

**Decision:** System prompts for authoring sessions are versioned files stored in `benchmarks/prompts/`. A developer or harness loads the appropriate prompt and connects their agent to Product MCP — via stdio (Claude Code) or HTTP (remote clients including claude.ai on mobile). `product adr review --staged` provides fast structural feedback on draft ADRs at pre-commit time. `product install-hooks` installs the pre-commit hook. Both commands are Product's; agent invocation is the harness's.

---

### System Prompt Files

Stored at paths configured in `product.toml` under `[author]`:

```
benchmarks/prompts/
  author-feature-v1.md     # graph-aware feature authoring
  author-adr-v1.md         # graph-aware ADR authoring
  author-review-v1.md      # spec gardening / coverage improvement
  implement-v1.md          # implementation context template
```

Each file is self-contained — it can be pasted into any LLM interface, loaded as a Claude Code system prompt, configured as a claude.ai Project instruction, or fed to any other agent. Product does not parse or interpret these files; it only exposes their paths via the MCP tool `product_prompts_list` and `product_prompts_get`.

**`author-feature-v1.md` preamble (excerpt):**
```markdown
You are authoring a new feature specification for a repository managed by Product.
You have access to Product MCP tools. Before writing any content:

1. Call product_feature_list — understand what features exist
2. Call product_graph_central — identify the top-5 foundational ADRs  
3. Call product_context on the most related existing feature
4. Ask clarifying questions based on what you found

Only scaffold files after completing these steps.
When done: call product_graph_check and product_gap_check on new artifacts.
```

**`author-adr-v1.md` preamble (excerpt):**
```markdown
Before writing any content:
1. Call product_graph_central — read the top-5 ADRs by centrality first
2. Call product_adr_list — see what decisions already exist
3. Call product_impact on the area you're deciding — understand blast radius

Every ADR must have: Context, Decision, Rationale, Rejected alternatives,
Test coverage. Do not end without all five sections present and a linked TC.
```

**`author-review-v1.md` preamble (excerpt):**
```markdown
Your goal is to improve specification coverage without adding new features.
1. Call product_graph_check — fix structural issues first
2. Call product_metrics_stats — identify weak metrics
3. Walk features by lowest phi score — propose formal blocks
4. Find features with W003 warnings — propose exit-criteria TCs
```

---

### How Agents Access Prompts

**Claude Code (stdio MCP, local):**
```bash
# .mcp.json is already in the repo — Claude Code connects automatically
# Developer opens Claude Code and pastes the prompt or uses a custom slash command:
/author-feature   # configured to send author-feature-v1.md as context
```

**claude.ai Project (HTTP MCP, phone or browser):**

The `author-feature-v1.md` content is pasted into the Project's instruction field once. Every conversation in that Project is automatically a graph-aware authoring session. No CLI command needed — the phone is always in authoring mode when that Project is open.

```
Project: PiCloud Development
Instructions: [contents of author-feature-v1.md]
Connected MCP servers: http://your-desktop:7778/mcp
```

**`product prompts init`** — scaffolds `benchmarks/prompts/` with the default prompt files if they don't exist:

```
product prompts init                  # create default prompt files
product prompts list                  # show available prompts and versions
product prompts get author-feature    # print prompt to stdout (for piping)
product prompts update author-feature # bump to latest version
```

These are file management commands, not agent invocation.

---

### Pre-Commit Hook

`product install-hooks` writes `.git/hooks/pre-commit`:

```bash
#!/bin/sh
# Installed by: product install-hooks
# Product is a knowledge tool. This hook runs knowledge checks, not agents.
STAGED_ADRS=$(git diff --cached --name-only | grep "^docs/adrs/")
if [ -n "$STAGED_ADRS" ]; then
    echo "Running product adr review on staged ADRs..."
    product adr review --staged
    # Advisory only — exit 0 regardless of findings
fi
exit 0
```

`product adr review --staged` performs:

**Structural checks (local, instant, no LLM):**
- All five required sections present (Context, Decision, Rationale, Rejected alternatives, Test coverage)
- `status` field set and valid
- At least one entry in `features` front-matter
- At least one TC linked
- Evidence blocks present on any `⟦Γ:Invariants⟧` blocks

**LLM review (single call, ~3 seconds):**
- Internal consistency: does rationale support the decision?
- Contradiction scan: compare against linked ADRs' decisions
- Missing test suggestion: given the claims, what TCs are obviously absent?

Output uses ADR-013 rustc-style diagnostics. Advisory — the commit proceeds regardless. Fast feedback before CI.

---

**Rationale:**
- System prompts as versioned files in the repository means they are version-controlled, reviewable in PRs, and shareable across any agent platform. They are not locked inside Product's binary. A team can fork them, iterate on them, and maintain their own prompt library alongside their specifications.
- `product prompts get author-feature` piping to stdin of any agent is the cleanest composition: `product prompts get author-feature | my-agent`. Product provides the prompt, the harness provides the agent.
- Pre-commit review is advisory and LLM-assisted because the goal is fast authoring-time feedback, not a CI gate. The structural checks (no LLM) complete in milliseconds. The LLM review adds 3 seconds. The developer sees both before the commit lands. The CI gap analysis gate is the hard enforcement point.
- `product prompts init` solves the bootstrap problem: a new repository has no prompt files. The command creates sensible defaults that the team can then evolve.

**Rejected alternatives:**
- **`product author feature` as a CLI command that starts Claude Code** — agent invocation is not Product's responsibility. Rejected (see ADR-021).
- **Prompts embedded in the binary** — not user-modifiable. Teams evolve their authoring approaches; baking prompts into the binary forces a Product upgrade to change a prompt. Rejected.
- **Pre-commit hook that starts an agent session** — a blocking agent session in a pre-commit hook makes commits slow and non-deterministic. The hook runs `product adr review --staged`, which is fast and deterministic. Rejected.