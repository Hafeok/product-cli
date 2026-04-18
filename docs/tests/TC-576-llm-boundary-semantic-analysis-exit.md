---
id: TC-576
title: llm_boundary_semantic_analysis_exit
type: exit-criteria
status: unimplemented
validates:
  features:
  - FT-045
  adrs:
  - ADR-019
  - ADR-022
  - ADR-023
  - ADR-040
phase: 1
runner: cargo-test
runner-args: tc_576_llm_boundary_semantic_analysis_exit
---

## Exit Criteria — FT-045 LLM Boundary — Semantic Analysis Bundles

FT-045 is complete when all of the following hold:

1. `product gap bundle ADR-XXX` produces a single markdown document to stdout containing an Instructions block (G001–G008) and the depth-2 Context Bundle for the ADR (TC-563).
2. `product gap bundle --changed` and `product gap bundle --all` scope correctly per ADR-019 (TC-564, TC-565).
3. `product gap check` is structural-only: no network calls, completes in under one second (TC-566), flags G002 (TC-567) and G003 (TC-568) deterministically.
4. `product drift diff FT-XXX` produces a single markdown document with Instructions, Implementation Anchor, Changes Since Completion, and Governing ADRs sections (TC-569, TC-571). Missing completion tag emits W020 and still produces a well-formed bundle (TC-570).
5. `product drift check FT-XXX` is structural-only: reports changed files since the completion tag (TC-572), exits 0 when there are no changes (TC-573), no LLM call.
6. `product adr conflict-bundle ADR-XXX` produces a bundle containing the proposed ADR plus the union of cross-cutting + same-domain + top-5-centrality ADRs (TC-574).
7. `product adr check-conflicts ADR-XXX` is structural-only: no network call, completes in under one second (TC-575).
8. `product adr review --staged` runs only structural checks (five sections, status, feature link, TC link, evidence). No LLM call inside the command.
9. `product.toml` no longer accepts a `[gap-analysis]` section (deprecated keys emit a W-class warning on first load). `max-files-per-adr` under `[drift]` is deprecated with a W-class warning.
10. `benchmarks/prompts/gap-analysis-v1.md`, `drift-analysis-v1.md`, `conflict-check-v1.md` exist and are listed by `product prompts list`.
11. The full inventory in ADR-040 holds: Product makes zero LLM API calls in production use (TC-566, TC-575 as the invariant anchors).
12. `cargo test`, `cargo clippy -- -D warnings -D clippy::unwrap_used`, and `cargo build` all pass.
13. Every TC in the feature (TC-563 through TC-576) has `runner: cargo-test` and `runner-args` matching the Rust test function name.
