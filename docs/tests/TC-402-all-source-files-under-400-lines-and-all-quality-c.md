---
id: TC-402
title: All source files under 400 lines and all quality checks pass
type: exit-criteria
status: passing
validates:
  features:
  - FT-031
  adrs:
  - ADR-029
  - ADR-043
phase: 3
runner: cargo-test
runner-args: tc_402_all_source_files_under_400_lines_and_all_quality_checks_pass
last-run: 2026-04-14T16:41:17.424364011+00:00
---

All five TC-CQ scripts pass on the Product codebase. No source file exceeds 400 lines. No function exceeds 40 statement lines. All required modules exist. All files have single-responsibility doc comments.