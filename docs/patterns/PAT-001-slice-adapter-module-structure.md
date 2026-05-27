---
id: PAT-001
title: Slice + Adapter module structure
status: live
domains:
- api
- data-model
adrs:
- ADR-043
requires: []
examples:
- FT-066
- FT-068
- FT-069
---

## When to use

Any new CLI command (or MCP tool) that does non-trivial work needs
unit-testable business logic. Mixing graph loading, validation,
computation, file I/O, and stdout rendering inside a single handler
makes the rule under test reachable only through `cargo run` plus
stdout parsing. Splitting into a slice (pure) + adapter (thin)
turns the rule into a function with typed inputs and a typed plan,
testable in milliseconds against an in-memory graph.

## Prerequisites

- Familiarity with `ProductError` (every slice function returns
  `Result<_, ProductError>`).
- Familiarity with the atomic-write helpers in `fileops` —
  `write_file_atomic` for single-file writes and
  `write_batch_atomic` for multi-file cascades.

## The pattern

A slice lives at `src/<slice>/` and exposes three function
families. An adapter lives at `src/commands/<cmd>.rs` and is a
thin wrapper.

```rust
// src/<slice>/<op>.rs — pure plan
pub struct Plan { /* typed description of the intended change */ }

pub fn plan_op(graph: &KnowledgeGraph, input: Input) -> Result<Plan, ProductError> {
    // Pure: no println!, no fs::write, no std::process::exit.
    // Compute the plan or return ProductError.
    Ok(Plan { /* ... */ })
}

// src/<slice>/<op>.rs — minimal I/O
pub fn apply_op(plan: &Plan, root: &Path) -> Result<(), ProductError> {
    fileops::write_file_atomic(&path, &rendered)?;
    Ok(())
}

// src/commands/<cmd>.rs — thin adapter
pub fn handle(args: Args) -> CmdResult {
    let graph = shared::load_graph_typed()?;
    let _lock = shared::acquire_write_lock_typed()?;
    let plan = slice::plan_op(&graph, args.into())?;
    slice::apply_op(&plan, repo_root())?;
    Ok(Output::text(slice::render_op_text(&plan)))
}
```

JSON parity is one line of `serde::Serialize` on the plan type —
no second branch in the handler. See `src/feature/link.rs`,
`src/feature/status_change.rs`, and `src/pattern/create.rs` for
real implementations.

## Anti-patterns

- **Doing the work in `commands/foo.rs` directly so the slice has
  nothing to test.** The rule then lives in a function that
  reaches for `std::fs`, `println!`, and the global loader; every
  change costs a 200 ms integration test instead of a 1 ms unit
  test.
- **Returning `BoxResult` from a new handler when no exception in
  `CLAUDE.md` applies.** `BoxResult` collapses every typed error
  to exit code 1, losing the `ProductError::exit_code` mapping.
- **Calling `println!` (or `eprintln!`) from inside the slice.**
  Output belongs to the adapter; the slice returns structured
  data and an `Output::Text` / `Output::Both` wrapper handles
  rendering at the boundary.

## Worked example

`src/feature/link.rs` (FT-066 era) is the cleanest current
example. `plan_link` takes the loaded graph and a `LinkSpec`, and
returns a `LinkPlan` describing every file that must change
(feature, TC, ADR, pattern) and every reciprocation edge.
`apply_link` calls `fileops::write_batch_atomic` exactly once.
The adapter `commands/feature_link.rs` (and the MCP handler in
`mcp/registry.rs`) call the same `plan_link` + `apply_link` —
the slice is the single source of truth for the linking rule.
