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
last-run: 2026-04-18T10:41:39.917241637+00:00
last-run-duration: 0.2s
---

feature declares `domains: [unknown-domain]`. Assert E012 (unknown domain, not in `product.toml` vocabulary).