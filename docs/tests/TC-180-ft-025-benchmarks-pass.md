---
id: TC-180
title: ft_025_benchmarks_pass
type: exit-criteria
status: passing
validates:
  features:
  - FT-025
  adrs:
  - ADR-018
phase: 3
runner: cargo-test
runner-args: "tc_180_ft_025_benchmarks_pass"
---

Run `cargo bench`. All four benchmarks complete without error and produce timing results within expected bounds.