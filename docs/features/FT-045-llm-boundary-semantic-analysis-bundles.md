---
id: FT-045
title: LLM Boundary — Semantic Analysis Bundles
phase: 5
status: complete
depends-on:
- FT-029
- FT-037
adrs:
- ADR-006
- ADR-013
- ADR-019
- ADR-022
- ADR-023
- ADR-036
- ADR-040
tests:
- TC-563
- TC-564
- TC-565
- TC-566
- TC-567
- TC-568
- TC-569
- TC-570
- TC-571
- TC-572
- TC-573
- TC-574
- TC-575
- TC-576
- TC-674
domains:
- api
- error-handling
- observability
domains-acknowledged:
  ADR-043: Predates ADR-043; feature command adapters were written before the slice+adapter pattern was formalised and are not retroactively refactored.
  ADR-041: Pre-dates ADR-041; this feature does not author absence TCs or set removes/deprecates on ADRs. FT-047 owns the mechanics.
  ADR-042: Pre-dates ADR-042; this feature does not define TC types or validate the type vocabulary. FT-048 owns the mechanics.
  ADR-018: Predates the 2026-04-22 scope promotion of ADR-018 to cross-cutting. Test coverage reflects the property/session/benchmark strategy as it existed when this feature shipped; not retroactively reclassified.
  ADR-048: Predates ADR-048; this feature does not author the canonical .product/ folder layout. FT-057 owns the migration command and discovery fallback.
  ADR-047: Predates ADR-047; this feature does not author the functional-spec body convention. FT-055 owns the structural validator and W030 mechanics.
---

## Description

Product is a knowledge tool. It assembles, validates, and presents information. It does not invoke LLMs. This feature delivers that principle by removing every internal LLM call site and replacing it with two commands per concern: a structural-only command (fast, deterministic, no LLM) and a bundle-producing command (LLM-ready input on stdout for the user to direct as they choose).

This feature implements the amendments to ADR-019, ADR-022, and ADR-023 recorded in ADR-040.

---

## Depends on

- **FT-029** — Gap Analysis. Owns `product gap check` and the `gaps.json` suppression model. This feature reshapes the command into structural-only and adds `product gap bundle` as the LLM-ready path.
- **FT-037** — Tag-Based Drift Detection. Owns completion tags and drift file resolution. This feature reshapes `product drift check` into structural-only and adds `product drift diff` as the LLM-ready path.
- **FT-044** — Unified Verify Pipeline. Ensures the pipeline retains the structural `product gap check` and `product drift check` entries (never the LLM-dependent variants) in stage 2 / stage 5.

---

## Scope of this feature

### In

1. **Remove every LLM call inside Product.** `product gap check`, `product drift check`, `product adr review --staged`, and `product adr check-conflicts` no longer invoke any LLM. No API keys, no model config, no network calls from these commands.
2. **New: `product gap bundle`.** Assembles the gap-check input (instructions + depth-2 context bundle for the ADR) and writes it to stdout as a markdown document formatted for LLM consumption. Supports `ADR-XXX` (single), `--all` (every ADR), `--changed` (ADRs modified since the last run), and `--format json`.
3. **New: `product drift diff`.** Assembles the drift-check input (instructions + git diff bounded to implementation files since the completion tag + depth-2 governing ADR context) and writes it to stdout. Supports `FT-XXX`, `--all-complete`, `--changed`, and `--format json`.
4. **New: `product adr conflict-bundle`.** Assembles the conflict-check input (proposed ADR + cross-cutting ADRs + same-domain ADRs + top-N by centrality) and writes it to stdout.
5. **Retain structural `product gap check`.** Checks G002 (invariant block with no TC), G003 (no rejected alternatives section), G008 (DEP with no governing ADR — equivalent to E013). G001 becomes an advisory heuristic keyword scan. G004/G005/G006/G007 are removed from `gap check` (they require semantic understanding; they move to `gap bundle`).
6. **Retain structural `product drift check`.** Confirms the completion tag exists (W020 otherwise), lists implementation files changed since the tag, exits 0 (no changes) or 2 (changes detected). No LLM call, no semantic judgment.
7. **Retain structural `product adr review --staged`.** Checks all five required sections present, `status` valid, ≥1 feature linked, ≥1 TC linked, evidence blocks on any `⟦Γ:Invariants⟧`. No LLM, no consistency scan. Pre-commit hook stays advisory and instant.
8. **Retain structural `product adr check-conflicts`.** Cycle detection on supersedes, symmetry check on `superseded-by`, domain-overlap check against cross-cutting ADRs, scope-consistency check. No LLM.
9. **Prompt files in `benchmarks/prompts/`.** The three previously-internal prompts ship as versioned resources: `gap-analysis-v1.md`, `drift-analysis-v1.md`, `conflict-check-v1.md`. `product prompts list/get/update` exposes them alongside the existing authoring prompts.
10. **Remove LLM config from `product.toml`.** Delete the `[gap-analysis]` section entirely; delete `max-files-per-adr` from `[drift]`; retain `source-roots` and `ignore`.
11. **gaps.json / drift.json compatibility.** Structural-only findings continue to use `gaps.json` and `drift.json`. LLM-detected findings (from `gap bundle` piped to the user's LLM) are outside Product's scope — the user manages them in whatever tool they choose.

### Out

- **User-side LLM orchestration.** Product emits the bundle; the user pipes it to their LLM of choice. No built-in `product gap bundle | run-llm` helper.
- **Caching of bundle output.** Bundles are deterministic functions of graph state and git history; regenerate on every call.
- **Migration of existing `gaps.json` suppressions** created under the old LLM-dependent analysis. Suppressions remain valid (the ID scheme is unchanged); they are tagged with the old prompt-version and emitted as W-class warnings on the first run under the new structural regime.
- **Embedding-based similarity for finding matching.** Rejected in ADR-019 and not reintroduced.

---

## Commands

```bash
# LLM-ready bundles — produce input, no LLM call inside Product
product gap bundle ADR-002                 # one ADR
product gap bundle --all                   # every ADR
product gap bundle --changed               # ADRs changed since last run
product gap bundle ADR-002 --format json   # machine-readable

product drift diff FT-001                  # one feature
product drift diff --all-complete          # every feature with a completion tag
product drift diff --changed               # features touched by recent commits
product drift diff FT-001 --format json    # machine-readable

product adr conflict-bundle ADR-031        # one proposed ADR + related ADRs

# Structural-only — instant, deterministic, no LLM
product gap check                          # G002, G003, G008, optional G001 heuristic
product drift check FT-001                 # tag exists? which files changed?
product adr check-conflicts ADR-031        # cycles, symmetry, overlap, scope
product adr review --staged                # five sections, status, links, evidence
```

---

## Bundle Output Format

Each bundle command writes a self-contained markdown document:

```markdown
# Gap Analysis Input: ADR-002 — openraft for Cluster Consensus

## Instructions

You are performing gap analysis on an architectural decision record.
Check for the following gap types only. For each gap found, output a
JSON object with fields: code, severity, description, location.

Gap types to check:
- G001: Testable claim with no linked TC
- G002: Formal invariant block with no scenario/chaos TC
- G003: No rejected alternatives section
- G004: Rationale references uncaptured external constraint
- G005: Logical inconsistency with a linked ADR
- G006: Feature aspect not addressed by any linked ADR
- G007: Rationale references superseded decisions
- G008: Feature uses dependency with no governing ADR

Output format: one JSON object per line, nothing else.

## Context Bundle

[full depth-2 context bundle for ADR-002]
```

The drift diff bundle follows the same skeleton but replaces the Context Bundle section with:

```
## Implementation Anchor
Feature: FT-001
Completion tag: product/FT-001/complete (2026-04-11T09:14:22Z)
Implementation files: 12 files across src/consensus/, src/storage/

## Changes Since Completion
[git diff output — bounded to implementation files since completion tag]

## Governing ADRs
[depth-2 context bundle — ADRs governing this feature]
```

The conflict-check bundle replaces Context Bundle with the proposed ADR plus the set of existing ADRs to check against (cross-cutting + same-domain + top-N by centrality).

---

## Implementation notes

- **`src/gap/bundle.rs`** — new module. `bundle_for_adr(adr_id, graph, root) -> String` emits the markdown. Reuses existing `context::assemble_bundle` for the context section; prepends the instruction block loaded from `benchmarks/prompts/gap-analysis-v1.md`.
- **`src/gap/check.rs`** — trim to structural checks only. Delete any HTTP / LLM client code paths. The module exports `check_structural(graph)` returning `Vec<Finding>`; nothing else.
- **`src/drift/diff.rs`** — new module. `diff_for_feature(feature_id, graph, root) -> String`. Uses existing `tags::check_drift_since_tag` for the git-diff section; uses `context::assemble_bundle` for the governing-ADRs section.
- **`src/drift/check.rs`** — reduce to tag-existence + changed-files reporting. Exit 0 if no changes, exit 2 if changes detected. Remove any LLM call, remove `drift.json` mutation triggered by LLM findings (the baseline file and its shape stay; only structural findings write to it).
- **`src/commands/adr.rs`** — `AdrCommands::Review { staged }` handler drops the LLM portion. The new `ConflictBundle { id }` subcommand is added to the enum; `CheckConflicts { id }` becomes structural-only.
- **`src/config.rs`** — remove the `GapAnalysisConfig` struct and its `[gap-analysis]` parsing. Remove `max_files_per_adr` from `DriftConfig`. Emit W-class warning on first load if the removed keys are present (W022: deprecated config key).
- **Prompt resources.** `benchmarks/prompts/` gets three new files; `product prompts init` is extended to scaffold them. Versioning of these files follows the same `-v{N}.md` convention as the authoring prompts.
- **Session tests** live under `tests/sessions/` per FT-043 conventions. Each session composes a temp repository, invokes the relevant command, and asserts on stdout / exit code / graph state. Runner config is added at the same time the test is written.

---

## Acceptance criteria

A developer running any of the structural commands on a clean repository can:

1. Observe `product gap bundle ADR-002` emit a markdown document to stdout with an Instructions section listing G001–G008 and a Context Bundle section containing the depth-2 bundle (TC-563).
2. Observe `product gap bundle --changed` scope correctly to ADRs modified in the last commit window (TC-564).
3. Observe `product gap bundle --all` include every ADR in the repository exactly once (TC-565).
4. Observe that `product gap check` (structural) makes zero network calls and completes in under one second on any repository (TC-566).
5. Observe `product gap check` flag G002 on an ADR with an `⟦Γ:Invariants⟧` block but no linked scenario/chaos TC (TC-567) and G003 on an ADR missing a rejected-alternatives section (TC-568).
6. Observe `product drift diff FT-001` emit a markdown document containing the git diff since the completion tag and the governing ADR bundle (TC-569).
7. Observe `product drift diff FT-001` warn W020 when no completion tag exists and still emit a well-formed bundle with an empty Changes section (TC-570).
8. Observe `product drift diff FT-001` emit a bundle whose Changes section is empty when there are no file changes since the tag (TC-571).
9. Observe `product drift check FT-001` report the list of changed files since the tag and exit 2 when changes are detected (TC-572).
10. Observe `product drift check FT-001` exit 0 when there are no file changes since the tag (TC-573).
11. Observe `product adr conflict-bundle ADR-031` emit a bundle whose "Existing ADRs" section contains every cross-cutting ADR plus same-domain ADRs plus top-5 by centrality, and nothing else (TC-574).
12. Observe `product adr check-conflicts ADR-031` run only structural checks and complete in under one second (TC-575).
13. Confirm `product.toml` no longer accepts the `[gap-analysis]` section and that `max-files-per-adr` under `[drift]` triggers a W-class deprecation warning (covered by the exit criteria).
14. `cargo test`, `cargo clippy -- -D warnings -D clippy::unwrap_used`, and `cargo build` all pass.

See TC-576 (exit criteria) for the consolidated check-list.

---

## Follow-on work

- **User-side orchestration recipes** — `scripts/harness/gap-analysis.sh` etc., showing idiomatic `product gap bundle ADR-XXX | claude -p gap-analysis-v1 | jq '.findings[]'` compositions. Not required for this feature to ship.
- **Benchmark refresh.** The existing LLM benchmark (`benchmarks/`) is unchanged by this feature. A follow-on may use the new bundle commands as the input path for the gap and drift dimensions of the benchmark.
- **Prompt version upgrades.** When a prompt evolves from v1 to v2, emit a W-class warning in `product prompts list` indicating suppressions created under v1 should be re-confirmed. The mechanism already exists for authoring prompts; extend to the new ones.

---

## Functional Specification

### Inputs

- `product gap bundle ADR-XXX` — one ADR ID; assembles the gap-analysis input bundle for that ADR.
- `product gap bundle --all` — every ADR in the repository.
- `product gap bundle --changed` — ADRs modified since the last run window.
- `product gap bundle [target] --format json` — machine-readable JSON output.
- `product drift diff FT-XXX` — one feature ID; assembles the drift-check input bundle for that feature.
- `product drift diff --all-complete` — every feature with a completion tag.
- `product drift diff --changed` — features touched by recent commits.
- `product drift diff [target] --format json` — machine-readable output.
- `product adr conflict-bundle ADR-XXX` — one proposed ADR; assembles the conflict-check bundle.
- `product gap check` — structural-only; reads the knowledge graph.
- `product drift check FT-XXX` — structural-only; reads git tags and the implementation file tree.
- `product adr check-conflicts ADR-XXX` — structural-only; reads the graph for supersedes cycles, symmetry, domain overlap, scope consistency.
- `product adr review --staged` — structural-only; reads the staged ADR file.
- Prompt resource files in `benchmarks/prompts/` (`gap-analysis-v1.md`, `drift-analysis-v1.md`, `conflict-check-v1.md`) — read by the bundle commands when constructing the Instructions section.
- `product.toml` — reads `source-roots` and `ignore` under `[drift]`; the `[gap-analysis]` section and `max-files-per-adr` under `[drift]` are removed and emit W022 if still present.

### Outputs

- **`product gap bundle`** — a self-contained markdown document on stdout with an `## Instructions` section (listing G001–G008 gap types with required output format) and a `## Context Bundle` section (depth-2 context bundle for the ADR). Suitable for piping directly to an LLM.
- **`product drift diff`** — a markdown document on stdout with an `## Implementation Anchor` section (feature, tag, file count), a `## Changes Since Completion` section (bounded git diff), and a `## Governing ADRs` section (depth-2 context bundle).
- **`product adr conflict-bundle`** — a markdown document on stdout with the proposed ADR plus the set of related ADRs (cross-cutting + same-domain + top-N by centrality) in the `## Existing ADRs` section.
- **`product gap check`** (structural) — findings G002, G003, G008, and optionally G001 (advisory heuristic keyword scan) in the existing structural gap-check stream. G004/G005/G006/G007 are removed (require semantic understanding; moved to `gap bundle` instructions).
- **`product drift check FT-XXX`** (structural) — lists implementation files changed since the completion tag; exits 0 (no changes) or 2 (changes detected). W020 if the completion tag does not exist.
- **`product adr check-conflicts ADR-XXX`** (structural) — cycle detection, symmetry check, domain-overlap, scope-consistency findings.
- **`product adr review --staged`** (structural) — checks five required sections present, `status` valid, ≥1 feature linked, ≥1 TC linked, evidence blocks on any `⟦Γ:Invariants⟧`.
- **`product prompts list/get/update`** — exposes the three new prompt files alongside existing authoring prompts.
- **W022** — emitted at config-load time when the deprecated `[gap-analysis]` section or `max-files-per-adr` key is present in `product.toml`.

### State

- Stateless. Bundle commands are deterministic functions of graph state and git history; they regenerate on every call with no caching.
- `gaps.json` and `drift.json` — structural-only findings continue to write to these files using the same ID scheme as before. LLM-detected findings (from bundles piped externally) are outside Product's scope.
- Prompt resource files in `benchmarks/prompts/` — versioned files on disk (`-v{N}.md` naming); not mutated by the bundle commands.
- `product.toml` — `[gap-analysis]` section and `max-files-per-adr` are removed from the schema; existing configs with these keys emit W022 on first load.

### Behaviour

1. **Bundle commands write nothing to disk.** `product gap bundle`, `product drift diff`, and `product adr conflict-bundle` write exclusively to stdout. They make zero network calls, zero LLM calls, and zero filesystem writes.
2. **`product gap bundle ADR-XXX`** loads the gap-analysis prompt from `benchmarks/prompts/gap-analysis-v1.md`, prepends it as the `## Instructions` section, then assembles the depth-2 context bundle for the ADR via `context::assemble_bundle` and appends it as `## Context Bundle`.
3. **`product drift diff FT-XXX`** checks for the completion tag via `tags::check_drift_since_tag`, writes a `## Changes Since Completion` section with the bounded git diff (implementation files only since the tag), then appends the governing ADR depth-2 bundle. If the tag does not exist, emits W020 and produces an empty Changes section.
4. **`product adr conflict-bundle ADR-XXX`** assembles the proposed ADR plus every cross-cutting ADR, same-domain ADR, and top-5 ADRs by centrality into the `## Existing ADRs` section.
5. **`product gap check` (structural)** checks G002 (invariant block with no TC), G003 (no rejected-alternatives section), G008 (DEP with no governing ADR). G001 is an advisory heuristic keyword scan. G004/G005/G006/G007 are removed; they require semantic understanding and are captured only in the bundle's Instructions section.
6. **`product drift check FT-XXX` (structural)** confirms the completion tag exists (W020 otherwise), lists implementation files changed since the tag, and exits 0 or 2. No LLM call, no semantic judgment.
7. **`product adr review --staged` (structural)** checks the five required sections, `status`, links, and evidence blocks. No LLM.
8. **`product adr check-conflicts ADR-XXX` (structural)** checks supersedes cycles, symmetry, domain overlap, and scope consistency. No LLM.
9. **W022 on deprecated config.** If `product.toml` contains `[gap-analysis]` or `max-files-per-adr` under `[drift]`, W022 is emitted at config-load time on every invocation. The keys are ignored; the graph builds normally.
10. **Prompt versioning.** Prompt files follow the `-v{N}.md` naming convention; `product prompts list/get/update` manages them alongside authoring prompts. W-class warning on `product prompts list` if suppressions were created under an older prompt version.

### Invariants

- Bundle commands make zero network calls and zero LLM calls. Verified by TC-566 (asserts no network activity and sub-second completion).
- Structural commands (`product gap check`, `product drift check`, `product adr check-conflicts`, `product adr review --staged`) make zero network calls.
- The `product verify` pipeline (FT-044) retains only structural `product gap check` and `product drift check` in its stages, never the LLM-dependent bundle variants.
- `gaps.json` suppression IDs created under the old LLM-dependent analysis regime remain valid; they are emitted as W-class warnings (not errors) on the first structural-only run.

### Error handling

| Code | Condition |
|---|---|
| W020 | Completion tag does not exist for the feature; `product drift diff` still emits a well-formed bundle with an empty Changes section |
| W022 | Deprecated `[gap-analysis]` config key or `max-files-per-adr` under `[drift]` present in `product.toml` |
| G002 | ADR has a formal `⟦Γ:Invariants⟧` block but no linked scenario/chaos TC |
| G003 | ADR is missing a rejected-alternatives section |
| G008 | Feature uses a DEP with no governing ADR (equivalent to E013 in graph check) |
| G001 | Advisory heuristic: testable-sounding language without a linked TC (keyword scan; non-blocking) |
| (exit code) | `product drift check FT-XXX` exits 2 when changes are detected; exits 0 when clean |

### Boundaries

- User-side LLM orchestration is out of scope: Product emits the bundle; the user pipes it to their LLM of choice. No built-in `product gap bundle | run-llm` helper.
- Caching of bundle output is not implemented; bundles are regenerated on every call.
- Embedding-based similarity for gap finding is excluded (rejected in ADR-019).
- Migration of existing `gaps.json` suppressions is not performed; suppressions remain valid and are preserved, but may emit W-class warnings on the first structural run.

## Out of scope

- User-side LLM orchestration: Product emits bundles; piping to an LLM is the user's responsibility. No built-in `product gap bundle | run-llm` helper is provided.
- Caching of bundle output: bundles are deterministic functions of graph state and are regenerated on every call.
- Migration of existing `gaps.json` suppressions created under the old LLM-dependent analysis: suppressions remain valid; they may surface as W-class warnings on first structural run.
- Embedding-based similarity for finding matching: rejected in ADR-019 and not reintroduced.
- G004/G005/G006/G007 gap types in `product gap check`: these require semantic understanding and are only documented in the bundle's Instructions section for the user's LLM to check.
