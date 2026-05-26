---
id: FT-067
title: 'Platform-scoped ADRs: separate enforced-by-platform decisions from per-feature gaps'
phase: 1
status: planned
depends-on: []
adrs:
- ADR-025
- ADR-026
- ADR-040
- ADR-018
- ADR-041
- ADR-042
- ADR-047
- ADR-043
tests: []
domains:
- api
- data-model
domains-acknowledged:
  ADR-048: No file-layout changes. This feature edits enum variants and call sites under `src/`; the canonical `.product/` layout is orthogonal.
  ADR-049: Context bundle templates are unaffected. The widening in `context/mod.rs` includes platform-scoped ADRs in the same bundle slot cross-cutting already occupies — no template change.
---

## Description

Add a fourth value to the `AdrScope` vocabulary — `platform` — for decisions that are **enforced once by the platform itself** (a fitness function TC, a chokepoint validator, a build-time check) rather than re-considered on every feature. Preflight stops flagging these ADRs as per-feature gaps; verify keeps running their TCs as project-wide platform checks.

The driving signal: `product preflight` on a fresh feature in `decision-cli` (a downstream repo) reports **30 cross-cutting gaps** even when the feature has nothing to do with most of them. Examination of the catalog shows 32 of 66 ADRs (≈48%) marked `scope: cross-cutting`. The current vocabulary collapses two distinct meanings into one value:

1. **"Every feature should pause and decide"** — true cross-cutting (e.g. an error-model ADR — every feature emits errors and must conform).
2. **"Enforced once by the substrate; per-feature linking is noise"** — fitness functions (e.g. code-quality enforcement, SHACL chokepoint validators, capability-tag binding done at dispatch time, system-wide vocabularies).

`src/verify/pipeline/stage_platform.rs:93-114` already encodes meaning (2) under the name *platform* — it sweeps cross-cutting ADRs into a single project-wide TC sweep at `verify --platform`. But `src/domains/preflight.rs:69-78` reads cross-cutting under meaning (1), demanding per-feature link-or-acknowledge for the same ADRs. The two callsites disagree about what cross-cutting means, and the per-feature gate wins on every preflight run.

This slice splits the concept. `platform` is the new value for meaning (2). `cross-cutting` keeps meaning (1). Authors re-tag the catalog repo-by-repo; the existing per-feature behaviour for genuine cross-cutting ADRs is unchanged.

One subcommand → one slice — this slice introduces no new CLI verbs. It extends an enum, threads the variant through three callsites (preflight, verify, adr list), and ships a migration helper for downstream repos.

## Functional Specification

### Inputs

- ADR front-matter `scope` field — accepts a new value `platform` alongside the existing `cross-cutting`, `domain`, `feature-specific`.
- Existing flags: `product adr list --scope <value>` accepts `platform` as a filter.
- Existing flags: `product verify --platform` already exists; the predicate that selects "platform TCs" in `stage_platform.rs:96` widens to include both `cross-cutting` AND `platform` scopes.

### Outputs

- `product preflight FT-XXX` — **no longer lists `platform`-scoped ADRs** in the *Cross-Cutting ADRs* section, and they cannot become gaps. They optionally appear in a new informational `Platform Invariants` section (one line per ADR, no symbols, no failure semantics) so authors can still see what platform invariants exist.
- `product adr show ADR-XXX` — displays `scope: platform` verbatim. No other output changes.
- `product adr list --scope platform` — filters the ADR list to platform-scoped ADRs.
- JSON output: any structure that surfaces `scope` gains `"platform"` as a possible string value; consumers using `serde` deserialization see the new variant directly.
- Exit codes: unchanged. Preflight's *cross-cutting gap count* drops for downstream repos that re-tag ADRs, which may flip the exit code from `1` to `0` — that is the intended behaviour.

### State

- `src/types.rs` — `AdrScope` enum gains a `Platform` variant; `Display`/`FromStr`/`Serialize`/`Deserialize` accept `"platform"`. Existing serde aliases are preserved (no breaking change to `cross-cutting` / `domain` / `feature-specific`).
- `src/domains/preflight.rs` — the loop at L68 keeps its existing predicate (`scope == CrossCutting`); a second loop (or extended structure) collects `Platform`-scoped ADRs for the informational section but never treats them as gaps.
- `src/domains/preflight.rs` `PreflightResult` — gains a `platform_invariants: Vec<PlatformInvariant>` field for the new section (mirrors `CrossCuttingGap` minus the `status` enum, since the status is always informational).
- `src/verify/pipeline/stage_platform.rs:96` — the predicate widens from `scope == CrossCutting` to `scope == CrossCutting || scope == Platform`. TCs validating ADRs of either scope are platform TCs.
- `src/gap/conflict.rs:74`, `src/context/mod.rs:87,103`, `src/graph/inference.rs:84`, `src/feature/link.rs`, `src/adr/conflicts.rs:135,142,167`, `src/domains/validation.rs:51,83`, `src/domains/coverage.rs:77`, `src/commands/feature_write.rs:59` — every existing callsite that compares against `AdrScope::CrossCutting` is audited and updated explicitly, per call:
  - Gap conflict (`gap/conflict.rs:74`): widens to include `Platform` (platform-scoped ADRs are still architectural facts that constrain new proposals).
  - Context bundle inclusion (`context/mod.rs:87,103`): widens to include `Platform` (LLMs should see platform invariants when implementing any feature).
  - Graph inference `skip_cross_cutting` flag (`graph/inference.rs:84`): renamed concept — `skip_platform_wide` — covers both. Existing callers retain the same boolean meaning.
  - ADR conflicts (`adr/conflicts.rs`): widens to include `Platform`.
  - Domain coverage / validation: widens to include `Platform` where the test is "is this ADR enforced project-wide?", stays narrow (`CrossCutting`-only) where the test is "must every feature link this?".
  - Feature link suggestion (`commands/feature_write.rs:59`): stays narrow (`CrossCutting`-only) — platform ADRs do not prompt link suggestions.

### Behaviour

#### Authoring a `platform` ADR

```bash
# Edit the ADR file directly, set:
scope: platform
```

The CLI does not introduce a `product adr scope` flag in this slice — scope is edited in the file. (A separate slice may add a write verb if demand warrants it.)

#### Preflight on a feature

For an ADR with `scope: cross-cutting`:
- Behaviour unchanged. Listed in *Cross-Cutting ADRs* section. Linked / acknowledged / gap.

For an ADR with `scope: platform`:
- Not listed in *Cross-Cutting ADRs*. Listed in a new *Platform Invariants* section (one line: `  •  ADR-XXX  <title>`), with no status symbol, no gap counting.
- Cannot contribute to the cross-cutting gap count or change the exit code.

For an ADR with `scope: domain`:
- Behaviour unchanged.

For an ADR with `scope: feature-specific`:
- Behaviour unchanged.

#### Verify --platform

The set of platform TCs widens to `{TCs validating any ADR with scope ∈ {cross-cutting, platform}} ∪ {Absence TCs}`. No ordering or output change otherwise.

#### Migration of existing repos

This slice ships a one-shot helper:

```bash
product adr scope-audit [--apply]
```

The dry-run lists every ADR currently `scope: cross-cutting` and offers a heuristic recommendation (e.g. "no `features:` backlinks AND linked TCs are all `invariant`/`absence` → suggest `platform`"). With `--apply`, the helper rewrites the `scope:` field in matching ADR files atomically. The user reviews and commits.

Repos that don't run the audit keep working — every `cross-cutting` ADR retains its existing behaviour.

### Invariants

- An ADR carries exactly one scope: `cross-cutting`, `platform`, `domain`, or `feature-specific`.
- `platform` ADRs never appear in `preflight`'s gap list, regardless of feature linkage.
- `cross-cutting` ADRs still appear in `preflight`'s gap list when neither linked nor acknowledged (existing ADR-026 invariant).
- `verify --platform` runs TCs for every ADR where `scope ∈ {cross-cutting, platform}`, plus every Absence TC. The set is a superset of today's set; no platform TC stops running.
- `product adr list --scope <value>` returns all and only ADRs with that exact scope value.

### Error handling

- An ADR with `scope: platform` and **zero linked TCs** is a soft warning, not a hard error — surfaced by `product gap check` as a new gap class `platform-no-enforcement` (severity: warning). Reasoning: if a decision is "enforced by the platform," there must be a platform check; otherwise the scope is wrong and the ADR should be `cross-cutting` (per-feature) or `feature-specific`.
- Invalid scope string in YAML — existing `FromStr` error path returns `error[E007]: invalid scope value '<x>', expected one of cross-cutting|platform|domain|feature-specific`. No new error code.
- `scope-audit --apply` failure on any file aborts the run, rolls back nothing (files are written atomically per-file via `fileops::atomic_write`), and exits with the offending path printed. The user re-runs after fixing.

### Boundaries

- This slice does **not** change which downstream-repo ADRs *should* be `platform`. That decision is per-repo, made by the repo's author. `scope-audit` only **suggests**.
- This slice does **not** change ADR-026's preflight contract for cross-cutting ADRs.
- This slice does **not** introduce a separate config list of "platform ADR IDs" in `config.toml` — the scope field on the ADR is the single source of truth.
- This slice does **not** retroactively re-classify any product-cli repo ADRs. A follow-up slice may audit product-cli's own catalog once the mechanism is in place.
- This slice does **not** add a CLI write verb for setting scope; the field is edited in the ADR file directly.

## Out of scope

- Bulk migration of `decision-cli`'s ADR catalog. The user runs `scope-audit` separately and reviews each suggestion.
- Renaming `cross-cutting` to a clearer name (e.g. `per-feature-attention`). Renaming breaks every downstream repo's ADR front-matter; not worth it for clarity alone.
- A `Fitness` scope distinct from `Platform`. The distinction (TC type: invariant vs. absence vs. property) is already carried on the TC, not the ADR. One scope value is enough.
- Promoting `Platform Invariants` to a non-informational gate (e.g. "fail preflight if any platform ADR has no TC"). That's `product gap check`'s job — flagged via the new `platform-no-enforcement` gap class.

## Test Criteria

[TC-NNN — to be authored at implementation time]

- **scenario**: `scope: platform` round-trips through YAML serde unchanged.
- **scenario**: `product preflight FT-X` on a feature that does **not** link a platform-scoped ADR exits 0 (no gap counted) and lists the ADR in a *Platform Invariants* section.
- **scenario**: `product preflight FT-X` on a feature linking a `cross-cutting` ADR still works exactly as today (regression).
- **scenario**: `product adr list --scope platform` returns exactly the platform-scoped ADRs.
- **scenario**: `product verify --platform` includes a TC validating a `platform`-scoped ADR (proves the widening in `stage_platform.rs`).
- **invariant**: every ADR file parses to exactly one of the four scope values; an unknown value emits `error[E007]`.
- **invariant**: `product gap check` reports `platform-no-enforcement` for a `platform`-scoped ADR with no linked TCs.
- **scenario**: `product adr scope-audit` dry-run prints recommendations without modifying files; `--apply` writes the changes.
- **exit-criteria**: in a fresh fixture repo with 5 ADRs (2 `cross-cutting`, 2 `platform`, 1 `feature-specific`), `product preflight FT-X` reports gaps only against the 2 `cross-cutting` ADRs.

## Impact on Downstream

After this ships, the workflow for the `decision-cli` repo (and any other downstream consumer with a bloated cross-cutting catalog) is:

1. `product adr scope-audit` — review suggestions.
2. `product adr scope-audit --apply` — re-tag obvious platform ADRs (or hand-edit).
3. `product preflight FT-XXX` — the cross-cutting gap count collapses to the genuinely cross-cutting ADRs.
4. The remaining gaps are signal — actual ADRs the feature should consider linking or acknowledging.

FT-103 (in decision-cli) becomes simpler: it only needs to backfill links for the **truly cross-cutting** ADRs after the re-scope, not paper over the over-tag problem.
