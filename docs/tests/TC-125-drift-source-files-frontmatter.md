---
id: TC-125
title: drift_source_files_frontmatter
type: scenario
status: passing
validates:
  features:
  - FT-028
  adrs:
  - ADR-023
phase: 1
runner: cargo-test
runner-args: "tc_125_drift_source_files_frontmatter"
last-run: 2026-04-18T10:41:56.996985101+00:00
last-run-duration: 0.2s
---

ADR with `source-files` in front-matter. Assert those files are used for analysis regardless of pattern config.