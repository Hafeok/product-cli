---
id: TC-434
title: init errors on existing product.toml without --force
type: scenario
status: passing
validates:
  features: [FT-035]
  adrs: [ADR-033]
phase: 1
runner: cargo-test
runner-args: "tc_434_init_errors_on_existing_product_toml_without_force"
last-run: 2026-04-14T14:52:43.866547207+00:00
---

## Description

Create a temporary directory containing a `product.toml` file. Run `product init --yes`. Assert:

1. Exit code is 1.
2. Stderr contains "product.toml already exists".
3. Stderr contains a hint mentioning `--force`.
4. The original `product.toml` content is unchanged.