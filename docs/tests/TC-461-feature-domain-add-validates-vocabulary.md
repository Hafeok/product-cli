---
id: TC-461
title: feature domain add validates vocabulary
type: scenario
status: passing
validates:
  features:
  - FT-038
  adrs:
  - ADR-037
phase: 1
runner: cargo-test
runner-args: "tc_461_feature_domain_add_validates_vocabulary"
last-run: 2026-04-15T11:45:30.776758337+00:00
last-run-duration: 0.2s
---

Run `product feature domain FT-XXX --add invalid-domain` where `invalid-domain` is not in the `[domains]` vocabulary in `product.toml`. Assert exit code 1 and error E012 with the invalid domain name and a hint to check `product.toml`. Then run with a valid domain name. Assert exit code 0 and the domain appears in the feature's front-matter `domains` list.