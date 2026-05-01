---
id: TC-702
title: FT-057 exit criteria
type: exit-criteria
status: passing
validates:
  features:
  - FT-057
  adrs: []
phase: 5
runner: cargo-test
runner-args: "--test sessions tc_702_ft_057_exit_criteria"
last-run: 2026-04-30T09:25:27.837372343+00:00
last-run-duration: 0.2s
---

**Test Type:** exit-criteria

FT-057 is complete when:

1. **Migration works.** TC tagged
   `tc-migrate-consolidate` passes (legacy → canonical
   migration including dry-run, apply, idempotency, and
   dirty-tree guard).
2. **Discovery fallback works.** TC tagged
   `tc-discovery-fallback` passes (canonical, alias, legacy,
   precedence, and walk-up scenarios all succeed).
3. **Existing tests pass on the new defaults.** All session
   tests (`tests/sessions/`) and integration tests
   (`tests/integration.rs`) pass when run against a fresh
   `.product/`-layout fixture.
4. **Existing tests pass on the legacy layout.** A
   dedicated session test that constructs a legacy-layout
   fixture exercises `product feature list`,
   `product context FT-X`, `product graph check`, and
   `product verify FT-X`, and all succeed without invoking
   migration. (Legacy support remains live until a future
   deprecation feature.)
5. **AGENTS.md is consistent.** The regenerated AGENTS.md
   lists `.product/` paths (not legacy `docs/` or
   `benchmarks/prompts/` paths) when generated against a
   canonical-layout repo.
6. **`CLAUDE.md` updated.** The "Project Structure" section
   in `CLAUDE.md` reflects the new layout. (This file is
   not auto-generated; the implementing change set must
   include the manual edit.)
7. **Build / test / lint clean.**
   - `cargo build` succeeds.
   - `cargo t` reports zero failing tests across all six
     binaries.
   - `cargo clippy -- -D warnings -D clippy::unwrap_used`
     reports zero warnings.
   - File-length and SRP fitness tests
     (`tests/code_quality_tests.rs`) still pass.
8. **Graph health.** `product graph check` on this repo,
   after self-consolidation, exits 0 with no new W-class
   warnings attributable to FT-057.