# ADR-0004: Validate convention identifiers via a `CtxId` newtype

**Status:** Accepted
**Date:** 2026-05-03
**Deciders:** Engineering team
**Convention:** [CTX003](../docs/CTX003.md)

## Context

The xtask convention checker surfaces a stable code (e.g. `CTX001`) on
every diagnostic. That code is the join key between three artifacts: the
`Check` implementation, the matching doc under `conventions/docs/`, and
the ADR referenced from the doc. A typo at any one site silently breaks
the join.

Phase 1 of the convention PRD landed with `Check::id() -> &'static str`.
That worked but required the drift self-test to compare strings against
strings. The drift test catches mismatches between *registered* checks
and their docs — it cannot catch a `Check` impl that returns a malformed
id which then passes through to a diagnostic with no matching doc.

We want to push this validation up the stack: into the type system, where
it costs nothing to enforce and impossible to bypass.

## Decision

Introduce a `CtxId` newtype around `&'static str` with a `const fn new`
constructor that validates the format (regex equivalent: `^CTX[0-9]+$`)
inside a const-evaluated `assert!`. Change `Check::id()` to return
`CtxId`. Lock the rule in with `compile_fail` doctests on `CtxId::new`.

This is the canonical Tier 1 pattern in this codebase: the wrong code
does not compile because the type to express it does not exist. No
runtime diagnostic, no fitness test, no markdown rule to maintain.

## Alternatives considered

- **Keep `&'static str` and add a runtime regex check in `Registry::new`.**
  Rejected: catches violations at runtime instead of compile time. Misuse
  in a code path the registry doesn't exercise (e.g. ad-hoc `Diagnostic`
  construction) still slips through.
- **Generate `CtxId` constants via a `define_check!` macro.** Rejected as
  premature: Tier 2 (proc macro) is more machinery than Tier 1 (newtype)
  for this case. If the validation logic ever needs to be more complex
  than "starts with `CTX`, rest are digits," a macro becomes attractive.
- **Use a `&'static str` const with a `const _: () = assert!(is_valid(ID))`
  block at every call site.** Rejected: identical safety in principle but
  much more boilerplate. Encapsulating the invariant in a type is the
  whole point.
- **Make `CtxId` parameterised over the format with a const-generic
  `prefix`.** Rejected: over-engineered for a single workspace.

## Consequences

- All `Check::id()` impls now declare `const ID: CtxId = CtxId::new("CTX###");`
  at the top of the file. Slightly more verbose than returning a string
  literal, but the const is reusable in tests and diagnostics.
- The drift self-test no longer needs to validate the *format* of
  registered ids — only that they match the frontmatter. The format
  invariant is upheld by the type.
- Doc-only ids that don't have a registered Check (e.g. `CTX-DEPS` for
  cargo-deny) are not constrained to the `CtxId` regex. This is
  intentional: the regex governs what `Check::id()` can return, not what
  filenames are permitted under `conventions/docs/`.

## References

- `xtask/src/check_id.rs` — the type definition and doctests.
- PRD: *In-Workspace Convention Enforcement (Rust)*, "Tier 1 — Eliminate
  via the type system."
