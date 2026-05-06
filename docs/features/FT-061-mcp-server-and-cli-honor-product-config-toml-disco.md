---
id: FT-061
title: MCP Server and CLI Honor `.product/config.toml` Discovery
phase: 5
status: planned
depends-on: []
adrs:
- ADR-048
tests: []
domains:
- api
- error-handling
domains-acknowledged: {}
---

## Description

FT-057 / ADR-048 established the canonical `.product/` layout and a
three-step discovery order (`.product/config.toml`,
`.product/product.toml`, `product.toml`) implemented by
`ProductConfig::find_config_in_dir` and `ProductConfig::discover`.

The CLI's command adapters route through `discover()` and so already
honour the canonical path. **The MCP server does not.** Fourteen call
sites in `src/mcp/**` and one in `src/commands/mcp_cmd.rs` build the
config path with the literal string:

```rust
ProductConfig::load(&repo_root.join("product.toml"))
```

That call returns `ConfigError` whenever a repo was created by
`product init` (canonical layout) because the file lives at
`.product/config.toml`, not at `product.toml`. Concretely:

- Every MCP tool that loads the config (`product_responsibility`,
  `product_graph_check`, `product_agent_context`, every write tool,
  `product_drift_check`, `product_preflight`,
  `product_request_validate`, `product_request_apply`, every field
  handler that needs to validate against `[domains]` or
  `[verify.prerequisites]`) returns an error against a canonical-layout
  repo.
- `product mcp --http` / `product mcp` (stdio) read `mcp.write` and
  `mcp.cors-origins` from the wrong path, silently defaulting to
  `false` / empty when those values are present in
  `.product/config.toml`.

## Functional Specification

### Inputs

- An existing repo using the canonical `.product/` layout (config at
  `.product/config.toml`).
- An MCP request (stdio or HTTP) for any tool that ultimately calls
  `ProductConfig::load`.
- `product mcp` invocations with or without `--write`, against either
  layout.

### Outputs

- Every MCP tool listed above succeeds against a canonical-layout repo.
- `product mcp` reads `mcp.write` and `mcp.cors-origins` from the same
  config the CLI sees.

### State

No new persisted state. A single new helper on `ProductConfig` is added
(see Behaviour). No breaking front-matter or config-schema changes.

### Behaviour

1. Add `ProductConfig::load_from_root(root: &Path) -> Result<Self>` that
   resolves the config via `find_config_in_dir(root)` and surfaces a
   structured `ConfigError` if no candidate file exists. The error
   message must enumerate the three searched filenames so callers can
   diagnose layout mistakes without reading source.
2. Replace every `ProductConfig::load(&repo_root.join("product.toml"))`
   call site with `ProductConfig::load_from_root(repo_root)` in:
   - `src/commands/mcp_cmd.rs` — both the `mcp.write` lookup and the
     `cors_origins` lookup. Reuse the config returned by
     `ProductConfig::discover()` when no `--repo` flag is supplied
     instead of re-loading it.
   - `src/mcp/registry.rs` — `load_graph()` and the
     `product_graph_check` responsibility-check branch.
   - `src/mcp/read_handlers.rs` — `handle_responsibility`,
     `handle_agent_context`.
   - `src/mcp/write_handlers.rs` — `handle_feature_new`,
     `handle_adr_new`, `handle_test_new`, `handle_body_update`.
   - `src/mcp/field_handlers.rs` — `handle_feature_domain`,
     `handle_adr_domain`, `handle_test_runner`.
   - `src/mcp/health_handlers/drift_check.rs` — `handle_drift_check`.
   - `src/mcp/health_handlers/preflight.rs` — `handle_preflight`.
   - `src/mcp/request_handlers.rs` — `handle_request_validate`,
     `handle_request_apply`.
3. Update the user-facing error in
   `read_handlers::handle_responsibility` to drop the literal
   "product.toml" reference, replacing with the discovered candidate
   filename or a layout-agnostic phrase ("the product config file").

### Invariants

- The legacy root-`product.toml` layout continues to work unchanged —
  `find_config_in_dir` already searches it as a third fallback.
- Test harnesses that scaffold legacy-layout repos
  (`tests/integration/mod.rs`) need no change: discovery still finds
  `product.toml` at the root.
- No new public CLI surface, no new config keys.

### Error handling

- `load_from_root` returns `ProductError::ConfigError` with a message
  listing all three searched filenames when no candidate exists. This
  matches the existing "No product.toml found in current directory or
  any parent" idiom from `discover()` while remaining accurate for the
  canonical layout.
- All existing E0xx codes flow through unchanged; no new error codes.

### Boundaries

- This feature does not touch `discover()` itself, the FT-057
  walk-up, or the `--root` / `PRODUCT_ROOT` plumbing — those already
  work. The fix is strictly call-site replacement plus one helper.
- This feature does not migrate any repo. Existing
  `product migrate consolidate` continues to be the only way to move a
  legacy repo to the canonical layout.

## Out of scope

- Renaming `product.toml` at the root (legacy alias remains).
- Auto-fixing legacy hardcoded `product.toml` references in user
  scripts or third-party MCP clients — only Product's own surface.
- A general "config discovery wrapper" abstraction beyond
  `load_from_root`. The helper is deliberately small.

## Acceptance criteria

1. In a fresh canonical-layout repo (`product init -y`):
   - `product mcp` (stdio) starts without error and serves
     `product_responsibility`, `product_graph_check`,
     `product_feature_new`, `product_drift_check`, and
     `product_preflight` against that repo.
   - `mcp.write = true` in `.product/config.toml` is honoured by
     `product mcp` without `--write` on the CLI.
2. In a legacy-layout repo (root `product.toml`), every behaviour
   above continues to work — no regression for the current test fixtures.
3. `cargo t`, `cargo clippy -- -D warnings -D clippy::unwrap_used`,
   and `cargo build` all pass.
4. New session test under `tests/sessions/` exercises an MCP read tool
   (e.g. `product_responsibility`) against a `.product/`-layout fixture
   and asserts success.

## Implementation notes

- Keep the new helper next to `find_config_in_dir` in `src/config.rs`
  to satisfy the SRP fitness check (the first `//!` line of
  `config.rs` already covers config parsing + discovery).
- The 14 call-site replacements are each one line; no logic changes.
- `commands/mcp_cmd.rs` should reuse the `ProductConfig` already
  returned by `discover()` rather than load twice. When `--repo` is
  supplied, fall through to `load_from_root` against the user-supplied
  path.
- A grep gate in the code-quality fitness suite is overkill for 14
  call sites, but a single-line property-style assertion in
  `tests/code_quality_tests.rs` (e.g. `repo_root.join("product.toml")`
  must not appear under `src/mcp/`) would prevent recurrence. Optional.
