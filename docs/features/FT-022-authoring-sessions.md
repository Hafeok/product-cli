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
  ADR-040: predates ADR-040; feature ships independently of the unified verify / LLM-boundary pipeline
  ADR-042: Pre-dates ADR-042; this feature does not define TC types or validate the type vocabulary. FT-048 owns the mechanics.
  ADR-018: Predates the 2026-04-22 scope promotion of ADR-018 to cross-cutting. Test coverage reflects the property/session/benchmark strategy as it existed when this feature shipped; not retroactively reclassified.
  ADR-043: Predates ADR-043; feature command adapters were written before the slice+adapter pattern was formalised and are not retroactively refactored.
  ADR-041: Pre-dates ADR-041; this feature does not author absence TCs or set removes/deprecates on ADRs. FT-047 owns the mechanics.
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