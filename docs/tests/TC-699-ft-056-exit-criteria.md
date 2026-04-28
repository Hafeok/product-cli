---
id: TC-699
title: FT-056 exit criteria
type: exit-criteria
status: passing
validates:
  features:
  - FT-056
  adrs: []
phase: 5
runner: cargo-test
runner-args: "tc_699_ft_056_exit_criteria"
last-run: 2026-04-28T09:50:28.631429697+00:00
last-run-duration: 0.2s
---

**Test Type:** exit-criteria

FT-056 is complete when all of the following hold:

1. **Override flows through.** With
   `<repo>/benchmarks/prompts/implement-v1.md` containing a
   sentinel string `S`, running `product implement FT-X --dry-run`
   writes a temp prompt file whose content starts with `S`,
   followed by the dynamic suffix (TC table, hard constraints,
   context bundle). Verified by TC-698.
2. **Fallback preserved.** With
   `<repo>/benchmarks/prompts/implement-v1.md` absent, running
   `product implement FT-X --dry-run` writes a temp prompt file
   whose content begins with the embedded
   `src/author/prompts/implement.txt` body, followed by the same
   dynamic suffix. Verified by TC-698.
3. **No regressions to interactive / headless paths.** Existing
   integration tests for `product implement` continue to pass
   unchanged. The change is internal — no flag, no env var, no
   CLI surface change.
4. **Build, test, lint clean.**
   - `cargo build` succeeds.
   - `cargo t` (the `--no-fail-fast` alias) runs every binary and
     reports no failing tests.
   - `cargo clippy -- -D warnings -D clippy::unwrap_used` reports
     no warnings.
   - File-length and SRP fitness tests
     (`tests/code_quality_tests.rs`) still pass —
     `src/implement/pipeline.rs` stays under 400 lines and its
     first `//!` line still does not contain "and".
5. **Graph health.** `product graph check` exits 0 and emits no
   new W-class warnings attributable to this feature.