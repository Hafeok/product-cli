---
id: TC-461
title: feature domain add validates vocabulary
type: scenario
status: unimplemented
validates:
  features: []
  adrs: []
phase: 1
---

Run `product feature domain FT-XXX --add invalid-domain` where `invalid-domain` is not in the `[domains]` vocabulary in `product.toml`. Assert exit code 1 and error E012 with the invalid domain name and a hint to check `product.toml`. Then run with a valid domain name. Assert exit code 0 and the domain appears in the feature's front-matter `domains` list.