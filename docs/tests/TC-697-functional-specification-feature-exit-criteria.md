---
id: TC-697
title: functional_specification_feature_exit_criteria
type: exit-criteria
status: unimplemented
validates:
  features:
  - FT-055
  adrs:
  - ADR-047
phase: 1
---

**Exit criteria for FT-055 — Feature Functional Specification Section.**

All of the following must pass for the feature to be considered complete:

1. **Parser TCs pass** — TC-681 (section detection) and TC-682 (subsection detection).
2. **W030 structural checks pass** — TC-683 (top-level section missing), TC-684 (subsection missing), TC-685 (all present clears W030), TC-686 (`required-from-phase` exemption), TC-692 (absent-section emission).
3. **Severity promotion works** — TC-687 (default is warning tier), TC-688 (error tier uses stable code `W030`), TC-689 (error tier blocks `planned → in-progress` transition).
4. **Empty vs absent distinction** — TC-690 (empty-meaning satisfies W030), TC-691 (whitespace-only does not).
5. **Context bundle integration** — TC-693 (full body in bundle), TC-694 (subsection structure preserved verbatim).
6. **Configuration surface** — TC-695 (`required-sections` override), TC-696 (`functional-spec-subsections` override).
7. **Code-quality fitness** — every touched file remains under 400 lines; every module's first `//!` line describes a single responsibility (no `and`).
8. **Tooling** — `cargo t`, `cargo clippy -- -D warnings -D clippy::unwrap_used`, `cargo build` all pass after the changes.
9. **Documentation** — `docs/product-functional-spec.md` exists (already committed) and ADR-047 links it in its body.
10. **`product graph check` clean on the repo after migration** — every non-stub feature in this repo either (a) has the required sections, (b) has an acknowledged gap, or (c) sits below `required-from-phase`. If severity is ever promoted to `"error"` in this repo's `product.toml`, the migration pass has already happened.

On all items passing, `product verify FT-055` marks the feature `complete` and a `product/FT-055/complete` tag is authored per ADR-036.
