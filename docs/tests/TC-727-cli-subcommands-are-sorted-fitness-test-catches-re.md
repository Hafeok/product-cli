---
id: TC-727
title: cli subcommands are sorted fitness test catches regressions
type: invariant
status: unimplemented
validates:
  features: []
  adrs: []
phase: 1
runner: cargo-test
runner-args: cli_subcommands_are_sorted
---

## Invariant

For every `Subcommand`-deriving enum declared under `src/commands/`,
the sequence of variant identifiers (translated to their clap-rendered
kebab-case names) is sorted under `str::cmp`.

## Formal blocks

⟦Σ:Types⟧{
  EnumName ≜ Identifier
  VariantName ≜ Identifier
  KebabName ≜ String
  SubcommandEnum ≜ ⟨name:EnumName, variants:VariantName+⟩
  CommandsTree ≜ SubcommandEnum+
}

⟦Γ:Invariants⟧{
  ∀ e:SubcommandEnum ∈ CommandsTree:
    LET names ≜ [variant_to_kebab(v) FOR v ∈ e.variants]
    IN names = sort(names)
}

⟦Ε⟧⟨δ≜0.9;φ≜90;τ≜◊⁺⟩

## Procedure

Implement as `cli_subcommands_are_sorted` in
`tests/code_quality_tests.rs`, consistent with the existing
file-length and SRP checks (string-based parsing, no `syn` dep).

`parse_subcommand_enums` finds every `pub enum <Name> {` block whose
preceding lines contain `#[derive(...Subcommand...)]`. Variant
identifiers are extracted between the enum's `{` and `}` tokens; each
identifier is the first PascalCase word on a line that does not begin
with `//`, `#[`, `}`, or whitespace-only content.

`variant_to_kebab` follows clap's default rule: insert a `-` before
every uppercase letter (except at position 0), then lowercase. If a
variant carries `#[command(name = "X")]`, use `X` as the rendered
name. Aliases are ignored.

On failure, the panic message must include:
- the file path,
- the enum name,
- the first out-of-order pair (`expected <X> before <Y> but got
  <Y> before <X>`).

## Expected

The test passes after FT-060's reordering work; introducing a
deliberately out-of-order variant on any branch must produce a
failing build with a clear message.
