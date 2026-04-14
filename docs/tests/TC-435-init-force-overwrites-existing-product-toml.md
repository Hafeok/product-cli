---
id: TC-435
title: init --force overwrites existing product.toml
type: scenario
status: passing
validates:
  features: [FT-035]
  adrs: [ADR-033]
phase: 1
runner: cargo-test
runner-args: "tc_435_init_force_overwrites_existing_product_toml"
last-run: 2026-04-14T14:52:43.866547207+00:00
---

## Description

Create a temporary directory with a `product.toml` containing `name = "old"`. Run `product init --yes --force --name new-project`. Assert:

1. Exit code is 0.
2. `product.toml` now contains `name = "new-project"`.
3. The old content is fully replaced.
4. Existing artifact directories (if any) are not deleted.