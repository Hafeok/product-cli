---
id: ADR-022
title: Authoring Sessions — Graph-Aware Specification Writing
status: accepted
features: []
supersedes: []
superseded-by: []
domains: []
scope: domain
---

**Status:** Accepted

**Context:** The current specification authoring flow is context-blind. A developer describes an idea in claude.ai, Claude writes PRD and ADR prose, the developer copies it to the repo. Claude has no awareness of what decisions already exist, which features are already planned, which ADRs are foundational, or whether the new content contradicts something already in the graph. The result is specifications that are internally consistent but externally inconsistent — they do not integrate with the existing artifact graph.

`product author` sessions fix this by giving Claude access to Product's MCP tools from the first message. Claude reads the graph before writing anything. It cannot propose a feature without knowing what features exist. It cannot write an ADR without reading the foundational decisions first.

**Decision:** `product author [feature|adr|review]` starts an agent session (Claude Code by default) with a versioned system prompt that defines the authoring approach and requires specific tool calls before content is produced. The session ends when `product graph check` is clean and `product gap check` returns no high-severity findings for newly created artifacts. Product MCP must be running (either stdio via Claude Code, or HTTP for remote sessions).

---

### System Prompt Design Principles

Each session type's system prompt enforces a mandatory read phase before any write:

**Feature prompt preamble:**
```
Before writing any content:
1. Call product_feature_list to understand what features exist
2. Call product_graph_central to identify the top-5 foundational ADRs
3. Call product_context on the most related existing feature (if any)
4. Ask the user clarifying questions based on what you found

Only after completing these steps should you scaffold any files.
```

**ADR prompt preamble:**
```
Before writing any content:
1. Call product_graph_central — read the top-5 ADRs by centrality
2. Call product_adr_list to see what decisions already exist
3. Call product_impact on the area you're about to decide — understand blast radius
4. Check for potential contradictions with existing linked ADRs

Every ADR must include: Context, Decision, Rationale, Rejected alternatives,
Test coverage. Do not end the session without all five sections present.
```

**Review prompt preamble:**
```
Your goal is to improve specification coverage without adding new features.
Start by:
1. Call product_graph_check — fix structural issues first
2. Call product_metrics_stats — identify weak metrics
3. Walk features by lowest phi score — propose formal blocks
4. Find orphaned ADRs — propose feature links
5. Find features with W003 warnings — propose exit-criteria TCs

Do not create new features or ADRs unless fixing a specific identified gap.
```

---

### Session Lifecycle

```
product author feature
  → loads author-feature-v1.md system prompt
  → starts Claude Code with Product MCP (stdio)
  → Claude reads graph, asks questions, scaffolds
  → on "done" signal from developer:
      product_graph_check → must be clean
      product_gap_check on new artifacts → must be no high-severity gaps
  → session ends, changes on disk ready to commit
```

For phone sessions (HTTP MCP), the lifecycle is identical except Claude Code is not involved — the conversation happens in claude.ai directly. The `product author` command is not needed; the system prompt is loaded as a claude.ai Project instruction set.

**Project instruction setup for phone:**
```markdown
# Product Authoring System Prompt

You have access to a Product MCP server for the [project-name] repository.

Before writing any specification content:
1. Call product_feature_list
2. Call product_graph_central  
3. Call product_context on related features
[... rest of feature prompt ...]
```

This instruction set, stored in the claude.ai Project, turns every conversation in that Project into a graph-aware authoring session. No `product author` command needed — the phone is always in authoring mode.

---

### Pre-Commit Hook

`product install-hooks` writes `.git/hooks/pre-commit`:

```bash
#!/bin/sh
# Installed by: product install-hooks
STAGED_ADRS=$(git diff --cached --name-only | grep "^docs/adrs/")
if [ -n "$STAGED_ADRS" ]; then
    echo "Running product adr review on staged ADRs..."
    product adr review --staged
    # Advisory only — exit 0 regardless of findings
fi
exit 0
```

`product adr review --staged` performs:

**Structural checks (local, instant):**
- All five required sections present (Context, Decision, Rationale, Rejected alternatives, Test coverage)
- `status` field is set and valid
- At least one entry in `features` front-matter
- At least one entry in `validates` (TC linked)
- Evidence blocks present on any `⟦Γ:Invariants⟧` blocks

**LLM review (single call, ~3 seconds):**
- Internal consistency: does the rationale support the decision?
- Contradiction scan: compare against linked ADRs' decisions
- Missing test suggestion: given the claims, what TCs are obviously absent?

Output format matches ADR-013 rustc-style diagnostics. Advisory — the commit proceeds regardless. The developer sees the findings immediately in their terminal.

---

**Rationale:**
- The mandatory read phase in system prompts is the critical design. Without it, Claude produces specifications in isolation. With it, Claude produces specifications that integrate. This is the difference between a documentation tool and a graph-aware authoring tool.
- Phone support via claude.ai Project instructions is simpler and more reliable than the `product author` command for remote sessions. The Project instruction set is persistent — every conversation automatically has the right authoring behaviour without remembering to run `product author`.
- Pre-commit review is advisory because blocking commits creates friction that causes developers to bypass the hook. Fast feedback is more valuable than a hard gate at commit time. The CI gap analysis gate is the hard enforcement point.

**Rejected alternatives:**
- **Require `product graph check` to pass before allowing the session to end** — too prescriptive. Sessions often end mid-thought with the intent to resume. A hard gate on session exit would create pressure to skip proper cleanup. The CI gate enforces cleanliness.
- **Auto-commit on session end** — Product commits the changes after the authoring session completes. Rejected: the developer must review changes before committing. Product never commits to git.