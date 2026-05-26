---
id: TC-785
title: mcp_feature_link_reciprocates_tc_validates_features
type: scenario
status: passing
validates:
  features:
  - FT-066
  adrs:
  - ADR-020
phase: 5
runner: cargo-test
runner-args: tc_785_mcp_feature_link_reciprocates_tc_validates_features
last-run: 2026-05-22T07:23:52.891705159+00:00
last-run-duration: 0.1s
---

## Description

Compose a temp repo with FT-X (status `planned`, no test links) and
TC-Y (status `unimplemented`, `validates.features: []`). Invoke
`product_feature_link` over MCP with `id: FT-X, test: TC-Y`. Assert:

1. `FT-X.tests` array contains `TC-Y` on disk.
2. `TC-Y.validates.features` array contains `FT-X` on disk.
3. Both writes land in the same atomic batch — corrupt the temp
   directory mid-call (drop write permission, fill the disk in a
   sibling fixture, etc.) and confirm that on retry neither file
   reflects a partial update.
4. The MCP response includes `TC-Y` in the `reciprocated` array with
   field `validates.features`.

The equivalent `product feature link FT-X --test TC-Y` against a
sibling temp repo produces byte-identical feature and TC files.