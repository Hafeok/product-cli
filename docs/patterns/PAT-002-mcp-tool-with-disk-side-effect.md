---
id: PAT-002
title: MCP tool with disk side-effect
status: live
domains:
- api
- storage
adrs:
- ADR-020
- ADR-038
requires:
- PAT-001
examples:
- FT-066
- FT-068
- FT-069
---

## When to use

Any MCP tool whose contract advertises a write (`requires_write:
true`) must produce a corresponding on-disk effect. If the
response envelope reports success but no file moved on disk, the
agent cannot distinguish a real write from a stub — and a TC that
only inspects the response cannot tell either. Use this pattern
whenever you add (or audit) an MCP write tool: feature/TC/ADR
status changes, link writes, pattern scaffolding, request applies.

## Prerequisites

- **PAT-001** — the slice the MCP tool dispatches into. Without a
  shared slice, the MCP handler and the CLI handler will drift
  apart.

## The pattern

The MCP handler is a thin call to the same `slice::plan_*` +
`apply_*` the CLI adapter uses. The slice owns the write through
`fileops::write_file_atomic` (single file) or
`write_batch_atomic` (cascade). Both transports — stdio and HTTP
— go through the same registry call.

```rust
// src/mcp/registry.rs — write tool handler
pub fn handle_feature_status(args: Value, root: &Path) -> Result<Value, ProductError> {
    let id = require_string(&args, "id")?;
    let status: FeatureStatus = require_string(&args, "status")?.parse()?;
    let graph = parser::load_all_full(/* ... */)?;

    // Pure plan from the shared slice — same call the CLI adapter makes.
    let plan = feature::plan_status_change(&graph, &id, status)?;
    // Atomic write through the slice's apply — disk effect lives here.
    feature::apply_status_change(&plan, root)?;

    Ok(json!({
        "id": plan.id,
        "status": plan.new_status.to_string(),
        "previous-status": plan.previous_status.to_string(),
    }))
}
```

The handler never serialises a "looks like success" envelope on a
no-op path. If the slice cannot produce a plan, the handler
returns a `ProductError` and the JSON-RPC envelope carries the
typed error — never a fake success.

## Anti-patterns

- **Returning a success envelope from a no-op stub.** The
  canonical FT-046 → FT-066 case: `handle_status_update` returned
  `{ id, status, note: "Use CLI for status updates..." }` and the
  feature on disk never changed. Every TC that inspected only the
  response passed. The lesson: agents cannot distinguish stubbed
  success from real success without reading the file.
- **Routing two distinct tools through one shared handler that
  discards the type information.** Collapsing
  `product_feature_status` and `product_test_status` into one
  generic handler that switches on a string field loses static
  typing on the inputs — and creates a single point where both
  tools' contracts can quietly diverge from disk.
- **Adding a `note: "Use CLI for ..."` field to a
  supposedly-equivalent MCP write.** Any string that hints "this
  call did not actually persist" is the failure mode this pattern
  exists to prevent. Delete the note and make the call real.

## Worked example

FT-066 is the bug-and-fix case study. The pre-fix
`product_feature_status` returned an envelope and wrote nothing;
the post-fix handler routes through `feature::plan_status_change`
+ `feature::apply_status_change`. The verification shape —
TC-778, TC-779, TC-787 — composes a temp repo, invokes the MCP
tool, and **reads the on-disk feature file** to assert
`status: complete`. The TC fails if the file did not change, even
when the envelope reports success.

See `src/feature/status_change.rs` for the shared slice and
`src/mcp/registry.rs` for the parity-respecting handler.
