# ADR-0002: No `.unwrap()` in production code paths

**Status:** Accepted
**Date:** 2026-05-03
**Deciders:** Engineering team
**Convention:** [CTX002](../docs/CTX002.md)

## Context

The product CLI mutates files on disk under advisory locks
(`fileops::atomic_write`), exchanges JSON over stdio with an MCP server,
and runs as a long-lived subprocess in pipelines like Dagger. A panic in
any of these paths corrupts in-flight state and leaves the user with a
stack trace instead of a typed error and a non-zero exit code.

`.unwrap()` is the single most common cause of accidental panics in Rust.
It is also the most reflexive shortcut for an LLM agent generating code:
the agent sees a `Result`, calls `.unwrap()`, and moves on. We need a
deterministic block that catches this at build time rather than relying on
review or prompt instructions.

Clippy's `unwrap_used` lint already exists and ships with the toolchain.
The CI pipeline already passes `-D clippy::unwrap_used` on the command
line. Promoting it to `[workspace.lints.clippy] unwrap_used = "deny"`
moves the rule into version-controlled config (where it travels with PR
diffs) and makes it visible in IDEs that read workspace lints.

## Decision

Set `clippy::unwrap_used = "deny"` in `[workspace.lints.clippy]`. Every
crate in the workspace inherits via `[lints] workspace = true`. The CI
step `cargo clippy -- -D clippy::unwrap_used` is preserved as a redundant
guard but is now belt-and-braces.

## Alternatives considered

- **xtask syn-based check** that walks AST nodes for `MethodCall` with
  `unwrap` ident. Rejected: Clippy already does this with type
  information, while syn would have false positives on
  `MyType::unwrap` (a custom method named `unwrap`). Use Clippy.
- **Forbid `expect_used` as well.** Rejected for now: the codebase uses
  `.expect("constant regex")` for compile-time-known invariants. Banning
  it would force a refactor that adds noise without preventing bugs. Open
  to revisit if the pattern proliferates.
- **Forbid `panic!`/`todo!`/`unimplemented!`.** Out of scope for this
  ADR; tracked separately if needed.
- **Runtime panic hook that converts to `Result`.** Rejected:
  unwinding-after-the-fact still leaves files half-written. The fix is to
  not panic.

## Consequences

- Every new code path in `src/` and `xtask/src/` propagates errors through
  `ProductError` (defined in `src/error.rs`).
- `expect()` remains permitted but should be reserved for invariants
  whose violation indicates a bug, not a recoverable error.
- Test code (`#[cfg(test)] mod tests`, `tests/`, `benches/`) is exempt
  via the `applies_to`/`exclude` fields in CTX002's frontmatter.

## References

- `CLAUDE.md` — "Key Conventions" section: zero unwrap policy.
- `src/error.rs` — `ProductError` enum, the canonical error model.
- Clippy docs: <https://rust-lang.github.io/rust-clippy/master/index.html#unwrap_used>
