---
id: TC-723
title: AGENTS.md key mcp tools table matches registry
type: invariant
status: unimplemented
validates:
  features: []
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_723_agents_md_key_mcp_tools_table_matches_registry
---

## Given

The current repository on disk, after FT-059 ships. The generated `AGENTS.md` content includes a "Key MCP Tools" markdown table that lists tool names in backticks (`` `product_xxx` ``).

## When

The fitness test:
1. Calls `crate::agent_context::generate_agent_md(...)` with the repo's config and graph.
2. Parses every backticked token under the "Key MCP Tools" section that matches `/^product_[a-z_]+$/`.
3. Calls `crate::mcp::tools::build_tool_list()` and collects `t.name` for every entry.

## Then

- For every advertised tool name in the AGENTS.md table, `build_tool_list()` contains a `ToolDef` with `name == that_name`.
- The set difference (advertised − registered) is empty. If non-empty, the test failure message lists the missing names, e.g. `"AGENTS.md advertises product_feature_next, product_dep_bom but the registry does not"`.

## Invariant

This is an `invariant`-type TC. It should run on every commit; failing it means either the agent context generator overpromises or the registry under-delivers. The fix is always one of: register the tool, remove the advertisement, or open a new feature to add it.

## Formal specification

⟦Σ:Types⟧{
  ToolName ≜ String matching ^product_[a-z_]+$
  Advertised ≜ {t:ToolName | t appears in AGENTS.md "Key MCP Tools" table}
  Registered ≜ {t:ToolName | t = d.name for some d ∈ build_tool_list()}
}

⟦Γ:Invariants⟧{
  ∀ t ∈ Advertised : t ∈ Registered
  ⇔  Advertised ⊆ Registered
  ⇔  (Advertised \ Registered) = ∅
}

⟦Ε⟧⟨δ≜0.9;φ≜100;τ≜◊⁺⟩
