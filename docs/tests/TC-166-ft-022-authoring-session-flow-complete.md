---
id: TC-166
title: FT-022 authoring session flow complete
type: exit-criteria
status: passing
validates:
  features:
  - FT-022
  adrs:
  - ADR-022
phase: 5
runner: cargo-test
runner-args: "tc_166_ft_022_authoring_session_flow_complete"
last-run: 2026-04-18T10:41:48.879855342+00:00
last-run-duration: 0.2s
---

## Description

End-to-end validation that all FT-022 authoring session components work together:
install-hooks creates an executable pre-commit hook, `adr review --staged` detects
structural issues (missing sections, empty features) in staged ADRs, and correctly
skips non-ADR files. Exit code is always 0 (advisory).