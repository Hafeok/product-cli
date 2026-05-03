# Conventions

Workspace conventions are enforced by the build, not by prompts or developer
discipline. This directory holds the docs and ADRs that explain *why* each
rule exists. The enforcement itself lives next to the mechanism that runs it
(types, macros, `[workspace.lints]`, or `xtask/src/checks/`).

## Layout

```
conventions/
├── docs/        # CTX###.md — what each rule is, with frontmatter + examples
├── adr/         # ADR-####.md — immutable rationale for accepted rules
└── README.md    # this file
```

## How a violation flows

1. The compiler (`cargo build`/`clippy`) or `cargo xtask check` fails with a
   diagnostic carrying a stable `CTX###` code and a permalink to the matching
   doc in this directory.
2. The doc explains the rule, lists the established ADR, and shows fixes.
3. The ADR explains *why* the decision was made and what was rejected.

## Adding a new rule

1. **Pick the cheapest enforcement tier that works.** See the decision tree
   below.
2. **Author the ADR** under `conventions/adr/ADR-####-<slug>.md`. Capture
   alternatives considered.
3. **Author the doc** under `conventions/docs/CTX###.md` with frontmatter:

   ```yaml
   ---
   id: CTX###
   title: <one-line title; must match Check::title()>
   severity: deny | warn
   tier: 1 | 2 | 3
   mechanism: type | macro | clippy | xtask | cargo-deny | dylint
   adrs: [ADR-####]
   applies_to: ["crates/*/src/**/*.rs"]
   exclude: []
   ---
   ```

4. **Wire up the enforcement:**
   - **Tier 3a** (Clippy or rustc lint): add to `[workspace.lints]` in the
     root `Cargo.toml`.
   - **Tier 3b** (xtask): add an impl of `Check` under
     `xtask/src/checks/ctx###_<slug>.rs` and register it in `Registry::default_set()`.
5. **Run `cargo xtask check --self-test`** locally to confirm the doc and the
   check agree.
6. **Open the PR.** CI runs `cargo xtask check`, `cargo xtask check --self-test`,
   `cargo clippy`, and the standard build.

## Tier selection decision tree

When adding a new rule, consult the tiers in order and stop at the first
match:

1. **Can the wrong code be made unrepresentable in the type system?** If yes,
   do that. Newtypes, sealed traits, phantom types, `#[must_use]`, trait
   bounds. **Tier 1.**
2. **Can the rule be enforced by code generation?** Derive macro, proc macro,
   or template. **Tier 2.**
3. **Can the rule be expressed via existing built-in or Clippy lints?** Set
   it in `[workspace.lints]`. **Tier 3a.**
4. **Can the rule be checked syntactically with `syn`?** Write an xtask
   check. **Tier 3b.** Default for most architectural rules.
5. **Is it a dependency / supply-chain / license rule?** `cargo-deny`.
   **Tier 3c.**
6. **Does the rule genuinely require type or trait resolution?** dylint, with
   nightly cost. **Tier 3d.** Last resort.

## Promotion

A rule may move *up* the tier list as the codebase evolves. When a rule moves
to tier 1 or tier 2 (e.g. by introducing a newtype that makes the violation
unrepresentable), update the doc's `tier` and `mechanism` fields. The
matching xtask check or workspace lint may eventually be removed; the doc
becomes a brief explanation pointing at the type that enforces it. The ADR
is unchanged — its job is rationale, not implementation.

## Example lifecycle

A new convention emerges in code review: *"all error types returned from a
public API must be `#[non_exhaustive]` so we can extend them without a
breaking change."*

1. **ADR drafted.** `conventions/adr/ADR-0042-non-exhaustive-errors.md`
   captures alternatives considered (sealed enums, error trait objects)
   and the decision.
2. **Convention doc added.** `conventions/docs/CTX008.md` carries the
   frontmatter, examples, and a pointer to ADR-0042.
3. **Enforcement wired.** Most rules start at Tier 3b (xtask + syn): scan
   public functions, find `Result<_, E>` returns, walk back to the type
   definition, verify it carries `#[non_exhaustive]`. Implementation lives
   at `xtask/src/checks/ctx008_non_exhaustive_errors.rs` and is registered
   in `Registry::default_set()`.
4. **Drift self-test picks it up.** `cargo xtask check --self-test`
   automatically iterates the new check and validates the doc agrees on
   id, title, adrs, severity, tier, and mechanism.
5. **PR merges.** From the next CI run onward, every crate in the
   workspace enforces the rule. Existing violations surface as build
   breaks; address them in follow-up PRs (or ship the check at
   `severity: warn` first and promote to `deny` once cleaned up).
6. **Promotion.** Months later, someone introduces a `#[derive(WorkspaceError)]`
   macro that emits the type with `#[non_exhaustive]` automatically. The
   rule moves to Tier 2: the doc updates `mechanism: macro`, the xtask
   check is rewritten as "all error types must be defined via
   `#[derive(WorkspaceError)]`", and the ADR is unchanged — its job is
   rationale, not implementation.

## The markdown is not in the trust path

These docs explain *why* a rule fired *after* it fired. The build is the
only authority. If a rule isn't enforced by a check (lint, type-system
encoding, xtask, or proc-macro), it isn't a rule — it's an aspiration.
