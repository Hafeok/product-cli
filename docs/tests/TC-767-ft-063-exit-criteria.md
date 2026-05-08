---
id: TC-767
title: FT-063 exit criteria
type: exit-criteria
status: passing
validates:
  features:
  - FT-063
  adrs:
  - ADR-049
phase: 1
runner: cargo-test
runner-args: tc_767_ft063_exit_criteria
last-run: 2026-05-08T12:14:59.128626357+00:00
last-run-duration: 0.5s
---

## Exit criteria — FT-063 Per-Model Context Bundle Templates

FT-063 is complete when all of the following hold:

1. `product context FT-XXX --target NAME` selects a resolved template and renders the bundle in the declared format (TC-749 / TC-750 / TC-751 / TC-752 / TC-753).
2. Template resolution order is repo → user → built-in, first-match-wins, and is visible via `product context templates --where` (TC-747 / TC-748 / TC-760).
3. `product context templates`, `--show NAME`, `--where`, `--reset NAME` produce the documented output (TC-759 / TC-761 / TC-762).
4. Built-in templates are read-only — `--reset NAME` on a built-in-only resolution emits **E029** without deleting any file (TC-763).
5. Template validation excludes invalid templates from the targets list with a startup warning, never blocking the binary (TC-743 / TC-744 / TC-745 / TC-746).
6. `[context].default-target` from `product.toml` selects the default; absence falls back to `human` (TC-757 / TC-758).
7. The MCP `product_context` tool accepts a `target` parameter and returns `format`, `target`, `content`, `token_count_approx`, `exceeded_target_max`, `exceeded_hard_max` (TC-764 / TC-765).
8. The `--for-llm` flag is a deprecated alias for `--target claude-opus` and emits a stderr deprecation note (TC-766).
9. Section ordering, `deliverables_at_top`, and `critical_first` are honoured per the template (TC-754 / TC-755 / TC-756).
10. `cargo t`, `cargo clippy -- -D warnings -D clippy::unwrap_used`, and `cargo build` all pass.
11. `product graph check` exits clean on the live repository after the feature lands.
12. AGENTS.md "Key MCP Tools" table reflects the new `target` parameter on `product_context`.

## Validates

- FT-063 — Per-Model Context Bundle Templates (overall)
- ADR-049 — Per-Model Context Bundle Templates as Data Files