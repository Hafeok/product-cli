---
id: FT-060
title: Alphabetically Sorted CLI Help Output
phase: 1
status: planned
depends-on: []
adrs:
- ADR-013
- ADR-029
- ADR-018
tests:
- TC-725
- TC-726
- TC-727
- TC-728
domains: []
domains-acknowledged: {}
---

## Description

`product --help` currently lists subcommands in the order they happen
to appear in the `Commands` enum in `src/commands/mod.rs`. That order
is historical, not alphabetical: `feature`, `adr`, `test`, `dep`,
`context`, `graph`, `impact`, `status`, `checklist`, `completions`,
`migrate`, `gap`, `mcp`, `implement`, `verify`, `author`,
`install-hooks`, `prompts`, `drift`, `tags`, `metrics`, `preflight`,
`onboard`, `init`, `hash`, `schema`, `agent-init`, `request`,
`cycle-times`, `forecast`.

Users scanning `--help` cannot quickly find a command because there is
no predictable lookup order. The same problem applies to nested
subcommand groups (`product feature --help`,
`product adr --help`, etc.), where variant order in the per-group
enums similarly leaks into the rendered help.

This feature makes the rendered help output stable and alphabetical
across the entire CLI surface:

1. Every top-level subcommand is listed in alphabetical order in
   `product --help`.
2. Every nested subcommand (the second-level `--help` for `feature`,
   `adr`, `test`, `dep`, `graph`, `checklist`, `migrate`, `gap`,
   `author`, `prompts`, `drift`, `tags`, `metrics`, `onboard`, `hash`,
   `request`) is also listed alphabetically.
3. The reordering is purely cosmetic — it touches the variant order
   of the relevant clap `Subcommand` enums and (where needed) the
   match arms in `dispatch`. No flags, exit codes, or behaviour
   change.

The simplest implementation is to physically reorder enum variants
alphabetically by their derived clap command name (kebab-case of the
variant identifier). Clap renders subcommands in declaration order,
so the source order is the only knob. We choose a physical reorder
over `Command::next_help_heading` / sort-at-render hooks because:

- It keeps the source self-documenting — anyone reading the enum
  sees the same order users see in help.
- It survives clap upgrades and `derive(Subcommand)` regenerations
  without any custom sort plumbing.
- A code-quality fitness test (added under
  `tests/code_quality_tests.rs`) can enforce the property going
  forward, so future PRs that add a new subcommand cannot regress
  the order.

---

## Functional Specification

### Inputs

- The current `Commands` enum in `src/commands/mod.rs` and every
  nested `*Commands` enum re-exported from `src/commands/`
  (`AdrCommands`, `AuthorCommands`, `ChecklistCommands`,
  `DepCommands`, `DriftCommands`, `FeatureCommands`, `GapCommands`,
  `GraphCommands`, `HashCommands`, `MetricsCommands`,
  `MigrateCommands`, `OnboardCommands`, `PromptsCommands`,
  `TestCommands`, `RequestCommands`, plus any others discovered
  during implementation).
- The clap-rendered command name for each variant — by default the
  kebab-case of the variant identifier, or the explicit name if a
  variant uses `#[command(name = "...")]` or `#[command(alias =
  "...")]`. The sort key is the *primary rendered name* (not aliases).

### Outputs

- `product --help` lists subcommands alphabetically by primary name.
- `product <group> --help` lists nested subcommands alphabetically
  for every group above.
- No change to `--version`, no change to any command's behaviour or
  exit codes.

### State

- No runtime state. The change is entirely a source-order change in
  enum declarations.

### Behaviour

1. Reorder variants in `Commands` (and every nested `*Commands`
   enum) to match the alphabetical order of their rendered command
   names. Variants are moved as whole blocks (doc comment + attrs +
   variant body) so doc comments stay attached.
2. Reorder match arms in the corresponding `dispatch` functions to
   match the new variant order. This is mechanical, not behavioural —
   `match` order in Rust does not affect semantics for non-overlapping
   patterns, but matching the enum order keeps diffs reviewable.
3. Add a code-quality fitness test
   (`tests/code_quality_tests.rs::cli_subcommands_are_sorted`) that
   parses each `Subcommand` enum in `src/commands/` via a regex over
   the source file (consistent with the existing `code_quality_tests`
   approach) and asserts variant order is alphabetical by rendered
   name.
4. Update integration tests that snapshot `--help` output (if any)
   to reflect the new order.

### Invariants

- For every `Subcommand` enum in `src/commands/`, the sequence of
  variant rendered names is sorted with the standard `str::cmp`
  ordering (case-sensitive ASCII, which matches clap's
  default rendering — clap lowercases identifiers when deriving
  command names).
- No two variants in the same enum render to the same primary
  command name (clap already enforces this; the test should not
  introduce a duplicate-detection false positive).
- The fitness test fails the build if a future PR adds an
  out-of-order variant.

### Error handling

- The fitness test failure renders as a standard `assert_eq!` /
  `panic!` from `cargo test --test code_quality_tests`, naming the
  enum file and the first out-of-order variant pair.
- No new `ProductError` variant is introduced — this is a
  build-time check, not a runtime error.

### Boundaries

- **In scope:** reorder enum variants for every `Subcommand` enum
  under `src/commands/`; reorder corresponding match arms; add the
  fitness test; update any help-output snapshot tests.
- **Out of scope:** changing argument order *within* a subcommand,
  changing flag names or aliases, changing the rendered help text
  itself, changing default values, adding new subcommands.
- **Out of scope:** sorting top-level argument flags
  (`--format`, `--root`) — those are global args, already few in
  number, and clap has its own conventions for them.
- **Out of scope:** sorting variants in non-`Subcommand` enums (e.g.
  internal state machines, error enums, `Output` enum). The
  alphabetical contract applies only to user-facing `--help`.

---

## Out of scope

- **Custom rendering hook** — using
  `clap::builder::Command::mut_subcommand` or a runtime sort would
  decouple source order from rendered order. We deliberately keep
  them coupled so the source is self-documenting.
- **Localised sort orders** — no need for locale-aware collation.
  ASCII case-sensitive ordering (the default) is what every
  contributor and reader uses, and it matches clap's own behaviour.
- **Reordering hidden / debug-only subcommands** — if any subcommand
  is `#[command(hide = true)]` it is still part of the enum and
  still subject to the fitness test for consistency, even though
  users do not see it in help.
- **Backports to legacy snapshot fixtures** — if a session test
  records a literal `--help` snapshot, the snapshot is updated in
  this feature's commit; we do not maintain compatibility with the
  pre-sort output.

---

## Acceptance criteria

A developer can:

1. Run `product --help` and see top-level subcommands in
   alphabetical order: `adr`, `agent-init`, `author`, `checklist`,
   `completions`, `context`, `cycle-times`, `dep`, `drift`,
   `feature`, `forecast`, `gap`, `graph`, `hash`, `impact`,
   `implement`, `init`, `install-hooks`, `mcp`, `metrics`,
   `migrate`, `onboard`, `preflight`, `prompts`, `request`,
   `schema`, `status`, `tags`, `test`, `verify`.
2. Run `product feature --help`, `product adr --help`, `product
   test --help` (etc.) and see the same alphabetical contract for
   every nested group.
3. Run `cargo t` and observe the new
   `cli_subcommands_are_sorted` fitness test passing.
4. Add a deliberately out-of-order variant to any `Subcommand` enum
   and observe `cargo t` failing with a clear message naming the
   offending enum and pair.
5. Run `cargo build`, `cargo t`, and `cargo clippy -- -D warnings -D
   clippy::unwrap_used` and observe all three pass.
6. Confirm no behavioural change: every previously passing
   integration test continues to pass with no edits beyond any
   help-snapshot updates the new order requires.

---

## Implementation notes

- Touch points (non-exhaustive — confirm during implementation):
  - `src/commands/mod.rs` — `Commands` enum + `dispatch` match.
  - `src/commands/feature.rs` — `FeatureCommands`.
  - `src/commands/adr.rs` — `AdrCommands`.
  - `src/commands/test_cmd.rs` — `TestCommands`.
  - `src/commands/dep.rs` — `DepCommands`.
  - `src/commands/graph_cmd.rs` — `GraphCommands`.
  - `src/commands/checklist.rs` — `ChecklistCommands`.
  - `src/commands/migrate.rs` — `MigrateCommands`.
  - `src/commands/gap.rs` — `GapCommands`.
  - `src/commands/author.rs` — `AuthorCommands`.
  - `src/commands/prompts_cmd.rs` — `PromptsCommands`.
  - `src/commands/drift.rs` — `DriftCommands`.
  - `src/commands/tags.rs` — `TagsCommands`.
  - `src/commands/metrics_cmd.rs` — `MetricsCommands`.
  - `src/commands/onboard.rs` — `OnboardCommands`.
  - `src/commands/hash.rs` — `HashCommands`.
  - `src/commands/request_cmd.rs` — `RequestCommands`.
- Sort key: kebab-case of the variant identifier, with explicit
  `#[command(name = "...")]` overrides honoured. For the existing
  enums this matches Rust's default identifier-to-kebab conversion
  (e.g. `AgentInit` → `agent-init`, `CycleTimes` → `cycle-times`,
  `InstallHooks` → `install-hooks`).
- The fitness test reads the source file as a string (no syn
  dependency — consistent with the existing 400-line / SRP fitness
  tests which use plain `std::fs::read_to_string` + line scanning).
  A small parser extracts variant identifiers between the enum's
  `{` and `}` and asserts the kebab-cased sequence is sorted.
- Files to grep for snapshot fixtures: anything under `tests/` that
  contains the literal substring `Usage:` or that pipes
  `--help` output to a comparison.
- Diff size: roughly one block-move per variant per enum. No
  behavioural changes; review burden is "do the variants line up
  alphabetically and does the match arm order still cover every
  variant".
