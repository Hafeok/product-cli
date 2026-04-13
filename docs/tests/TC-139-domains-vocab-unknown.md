---
id: TC-139
title: domains_vocab_unknown
type: scenario
status: passing
validates:
  features:
  - FT-018
  - FT-019
  adrs:
  - ADR-025
phase: 1
runner: cargo-test
runner-args: "tc_139_domains_vocab_unknown"
---

feature declares `domains: [unknown-domain]`. Assert E012 (unknown domain, not in `product.toml` vocabulary).