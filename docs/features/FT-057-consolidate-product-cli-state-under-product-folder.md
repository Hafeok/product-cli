---
id: FT-057
title: Consolidate Product CLI State Under `.product/` Folder
phase: 5
status: planned
depends-on:
- FT-051
- FT-056
adrs:
- ADR-022
- ADR-033
- ADR-038
- ADR-039
- ADR-048
tests:
- TC-700
- TC-701
- TC-702
domains: []
domains-acknowledged:
  ADR-045: No interaction with planning annotations. Due dates and started tags remain advisory; the consolidation migration is orthogonal to the planning surface. Tag references in the request log continue to resolve identically post-migration since the log retains repo-root-relative paths (FT-051).
  ADR-042: Uses only existing TC types — `scenario` for the migration and discovery tests (TC-700, TC-701) and `exit-criteria` for the consolidated check-list (TC-702). No new TC types introduced; ADR-042's reserved-structural / open-descriptive partition is unchanged.
  ADR-018: 'Test coverage follows ADR-018 Design 2 (session-based integration tests): TC-700 exercises the migration end-to-end on a legacy-layout fixture, TC-701 verifies discovery walks canonical → alias → legacy paths, and TC-702 is the consolidated exit-criteria. Property tests do not apply — the migration is a deterministic file-move operation already covered by the sessions.'
  ADR-038: The migration command writes a single new `migrate` entry to the request log with sentinel `consolidate-paths`, reusing the established machinery added in FT-051. No new request shape, no new mutation operations — only a new sentinel value on the existing migrate entry type. Hash-chain integrity is preserved by appending to the log via the existing path.
  ADR-046: No interaction with cycle-time visibility. Tag timestamps consumed by `product cycle-times` come from git, not from product-managed paths; relocating `requests.jsonl` to `.product/requests.jsonl` does not affect tag reads. The migration is invisible to the cycle-time pipeline.
  ADR-043: 'Implementation follows the slice + adapter pattern: new `src/migrate/consolidate.rs` slice exposes pure `plan_consolidate` returning a `ConsolidationPlan` and `apply_consolidate` performing the I/O via `fileops::write_batch_atomic` semantics; the `src/commands/migrate.rs` adapter remains thin. Path-consumer updates in `src/author/prompts.rs` and `src/implement/pipeline.rs` only swap hardcoded strings for `config.paths.*` reads.'
  ADR-041: No absence TCs or ADR removes/deprecates interaction. The legacy `docs/` layout is not deprecated by this feature — it remains a supported configuration via explicit `[paths]` overrides; the change is purely additive (new defaults plus a migration command). A future deprecation feature would handle the lifecycle transition.
  ADR-040: No new verify stage and no LLM-boundary change. `product migrate consolidate` is a one-shot administrative operation invoked outside the verify pipeline; it touches `[paths]` and the request log via the existing `migrate` entry sentinel mechanism (FT-051) without extending the six-stage pipeline.
  ADR-044: No interaction with the request builder draft lifecycle. The migration command is a one-shot administrative action; it does not extend the builder's draft surface, does not introduce new mutation operations, and the new `[paths]` keys are read-only configuration consumed by existing path consumers.
---

## Description

Implement the canonical `.product/` layout established by the
governing ADR. Three concrete deliverables:

1. **Default-path change** — `[paths]` defaults in
   `src/config.rs` move to `.product/…`; new `prompts` and `gaps`
   keys are added (replacing the hardcoded
   `benchmarks/prompts` and `gaps.json` strings in
   `src/author/prompts.rs` and `src/implement/pipeline.rs`).
2. **Backward-compat discovery** —
   `ProductConfig::discover` walks up checking in order:
   `.product/config.toml`, `.product/product.toml`,
   `product.toml`. First match wins.
3. **Migration command** — `product migrate consolidate`
   physically moves legacy paths to `.product/`, rewrites
   `[paths]` in the config, manages `.gitignore` entries, and
   records the migration as a `migrate` entry in the request log
   (preserving hash-chain integrity per ADR-039).

The change is **opt-in for existing repos**: nothing moves until
`product migrate consolidate` is run. New repos created via
`product init` use the new defaults from day one.

---

## Depends on

- The governing ADR (proposed in this same request).
- **ADR-022** — prompt locations. The new ADR updates the path;
  this feature implements the update with back-compat read
  fallback so prompts placed at the legacy `benchmarks/prompts/`
  path continue to resolve until migration runs.
- **ADR-033** — `product init`. This feature updates the init
  scaffolder to emit the new layout. Existing TCs for init
  (TC-431…TC-439) need updating to expect the new directory
  structure.
- **ADR-038, ADR-039** — request log. The `migrate` entry type
  already exists; this feature adds a new sentinel
  (`consolidate-paths`) reusing the established machinery.
- **FT-051** — relative paths in the request log. Because paths
  in the log are repo-root-relative (not absolute), moving the
  log file from `requests.jsonl` to `.product/requests.jsonl`
  does not invalidate any prior entries.
- **FT-056** — the prompt-override fix lands first or in
  parallel; both touch `src/author/prompts.rs` and the
  consolidation work needs to pick up the new `prompts` config
  key in the same pass.

---

## Scope of this feature

### In

1. **Config schema**
   - Add `[paths]` keys: `prompts` (default
     `.product/prompts`) and `gaps` (default
     `.product/gaps.json`).
   - Update existing `[paths]` defaults: `features`, `adrs`,
     `tests`, `dependencies`, `graph`, `checklist`, `requests`
     all move under `.product/`.
   - Both legacy default values (`docs/features` etc.) remain
     valid — they just stop being the **default**.
2. **Discovery fallback** in `ProductConfig::discover`
   (`src/config.rs:316-332`) extended to walk:
   (a) `.product/config.toml`,
   (b) `.product/product.toml`,
   (c) `product.toml`.
   Whichever exists first wins.
3. **Path consumers updated** to read the new config keys:
   - `src/author/prompts.rs::init/list/get` —
     `prompts_dir` reads `config.paths.prompts`. Falls back to
     `benchmarks/prompts` if the new key is missing AND the
     legacy directory exists, with a one-shot W-class warning to
     guide migration.
   - `src/implement/pipeline.rs:44` — `gap::GapBaseline::load`
     reads `config.paths.gaps` instead of the hardcoded
     `gaps.json`.
   - All other path readers in the codebase already route
     through `config.paths` or `resolve_path`, so the default
     change is the only change they need.
4. **`product migrate consolidate`** subcommand. Behaviour:
   - **Dry-run mode (default):** prints a plan listing every
     file/directory move, every `[paths]` rewrite, and every
     `.gitignore` line to be added. Exit 0. No filesystem
     writes.
   - **Apply mode (`--apply` / `-a`):** performs every move
     atomically using `fileops::write_batch_atomic` semantics
     (write new, fsync, rename old → backup, rename new →
     target, drop backup on success), then rewrites the config
     file, then appends a `migrate` entry to the request log
     with sentinel `consolidate-paths`.
   - Skips paths already at the canonical location (idempotent).
   - Refuses to run if any path has uncommitted git changes
     (avoid clobbering a contributor's WIP). Override with
     `--force-uncommitted`.
   - Honours user-specified `[paths]` overrides — if a team has
     explicitly configured `features = "docs/features"`, the
     command leaves it there and prints a notice.
5. **`product init` scaffolder updates** to create `.product/`
   skeleton with the new defaults and write
   `.product/config.toml`. Falls back to `product.toml` at root
   only when invoked with `--legacy-layout`.
6. **`.gitignore` management** in `init` and `migrate
   consolidate`: append `.product/graph/` and
   `.product/sessions/` (rather than the legacy
   `docs/graph/`).
7. **AGENTS.md regeneration** — the path table in
   `agent_context::generate` is updated to reflect the new
   canonical paths so that LLM agents reading AGENTS.md see the
   current layout, not the legacy one.
8. **Migration sentinel and verifier rule** —
   `request_log` learns `MIGRATE_LOG_SENTINEL_CONSOLIDATE =
   "consolidate-paths"`. The verifier accepts pre-migration
   entries that reference legacy paths the same way
   FT-051 made it accept absolute paths pre-relativisation
   (entry-hash never recomputed; the migrate entry documents the
   rewrite).
9. **Tests** —
   - Sessions test: legacy-layout repo, run `product migrate
     consolidate --apply`, observe canonical layout, observe
     rewritten `[paths]`, observe `migrate` log entry, observe
     graph still passes `product graph check`.
   - Sessions test: discovery fallback — three tempdir repos
     (one canonical, one `.product/product.toml` legacy alias,
     one root `product.toml`), each runs `product feature list`
     successfully.
   - Property test extension: any combination of `[paths]`
     overrides plus migration produces a config whose
     `discover()` round-trips to the same `ProductConfig`.

### Out

- **Auto-migration on upgrade.** Explicitly out of scope per
  the ADR's Rule of explicit migration.
- **Symlinks from legacy to canonical paths.** Rejected in the
  ADR; not implemented here.
- **Moving `AGENTS.md`/`.mcp.json`/`CLAUDE.md`.** Out of scope
  per the ADR's Rule of external conventions. Teams that want to
  relocate `AGENTS.md` use `[agent-context].output-file`.
- **Cross-repo path templates** (e.g. mono-repo support with
  `.product/` per sub-package). The feature targets one
  `.product/` per repo. Multi-repo consolidation is a different
  problem.
- **Migration of historical `requests.jsonl` paths.** FT-051
  already made the log paths relative; moving the log file does
  not require touching its contents.
- **Renaming `product.toml` → `config.toml` at the repo root.**
  Out of scope. Inside `.product/`, the new canonical name is
  `config.toml`. At the root (legacy), the file remains
  `product.toml` for back-compat.

---

## Commands

- `product migrate consolidate` (new) — dry-run by default;
  `--apply` performs the migration; `--force-uncommitted`
  overrides the dirty-tree guard.
- `product init` (changed) — defaults to `.product/` layout;
  `--legacy-layout` opts into the pre-FT-057 root-based layout.
- `product init --force` semantics unchanged.

---

## Implementation notes

- **`src/config.rs`** —
  - Extend `PathsConfig` with two `Option<String>` fields
    (`prompts`, `gaps`) and corresponding `default_*_path()`
    functions that resolve to `.product/prompts` and
    `.product/gaps.json`.
  - Extend `ProductConfig::discover` with the three-step
    fallback. Keep the function under 50 lines — extract a
    helper `find_config_in_dir(&Path) -> Option<PathBuf>` if
    needed.
- **`src/author/prompts.rs`** —
  - Replace the two hardcoded `"benchmarks/prompts"` strings
    with `config.paths.prompts`. The `get` function gains a
    legacy-fallback: if the configured prompts dir does not
    exist but `benchmarks/prompts` does, read from there and
    emit a W-class warning once per process via a `OnceLock`
    guard.
- **`src/implement/pipeline.rs`** —
  - Replace `root.join("gaps.json")` with
    `config.paths.gaps_resolved(root)` (helper added in
    `PathsConfig`).
- **`src/migrate/`** — add a new submodule `consolidate.rs`
  following the slice + adapter pattern. `plan_consolidate`
  returns a `ConsolidationPlan` (list of moves + config edits +
  gitignore lines + migrate entry); `apply_consolidate` performs
  the I/O. The CLI adapter lives in `src/commands/migrate.rs`.
- **`src/request_log/`** — add the
  `MIGRATE_LOG_SENTINEL_CONSOLIDATE` constant and teach the
  verifier to accept legacy paths in pre-migration entries.
- **File-length budget.** Both `pipeline.rs` and `prompts.rs`
  currently sit comfortably under 400 lines and the additions
  are minimal. The new `consolidate.rs` is the largest addition;
  keep it under 400.
- **Doc updates.** `CLAUDE.md` "Project Structure" section needs
  updating once the migration lands. Generated `AGENTS.md`
  regenerates automatically via `agent_context::generate`.

---

## Acceptance criteria

A developer can:

1. Clone the legacy-layout repo (the current state of
   `product-cli` itself), run
   `product migrate consolidate --apply`, and observe:
   - `product.toml` moved to `.product/config.toml`.
   - `docs/features/`, `docs/adrs/`, `docs/tests/`,
     `docs/dependencies/`, `docs/graph/` moved to
     `.product/{features,adrs,tests,dependencies,graph}/`.
   - `benchmarks/prompts/*` moved to `.product/prompts/`.
   - `gaps.json` (if present) moved to `.product/gaps.json`.
   - `requests.jsonl` moved to `.product/requests.jsonl`.
   - `[paths]` in `.product/config.toml` rewritten to the new
     defaults.
   - `.gitignore` updated to reference `.product/graph/` and
     `.product/sessions/`.
   - One new `migrate` entry in `.product/requests.jsonl` with
     sentinel `consolidate-paths`.
   - `product graph check` exits 0.
   - `product feature list` and `product context` work
     identically to before.
2. Clone a fresh empty directory, run `product init`, and
   observe `.product/config.toml` plus the canonical skeleton.
3. Run `cargo t`, `cargo clippy -- -D warnings -D
   clippy::unwrap_used`, and `cargo build` and observe all pass.
4. Run `product graph check` and observe zero new warnings
   attributable to this feature.

---

## Follow-on work

- **Mono-repo support.** A future feature could allow multiple
  `.product/` roots under a workspace, with discovery picking the
  nearest one. Out of scope here.
- **Diátaxis guides relocation.** `docs/guide/` is currently
  outside Product's scan paths. A future feature could either
  move them under `.product/guide/` (treating them as
  product-managed) or formalize their place in `docs/`. The
  latter matches their user-facing intent; defer the decision.
- **`AGENTS.md` location revisit.** If a future agent-context
  convention emerges (e.g. `.agents/AGENTS.md`), revisit
  Rule 2.
