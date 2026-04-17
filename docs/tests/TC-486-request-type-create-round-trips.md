---
id: TC-486
title: request type create round-trips
type: scenario
status: passing
validates:
  features:
  - FT-041
  adrs:
  - ADR-038
phase: 1
runner: cargo-test
runner-args: tc_486_request_type_create_round_trips
last-run: 2026-04-17T11:51:10.501467904+00:00
last-run-duration: 1.8s
---

Validates FT-041 / ADR-038 decision 1.

**Setup:** an empty or minimal test repository fixture.

**Act:** write a `type: create` request YAML containing a single feature with `title`, `phase`, `domains`, and a valid `reason:`. No `ref:` needed. Run `product request validate FILE` then `product request apply FILE`.

**Assert:**
- `validate` exits 0 with no findings
- `apply` exits 0
- A new `docs/features/FT-NNN-*.md` file exists with front-matter containing the declared title, phase, and domains
- The MCP equivalent (`product_request_apply`) returns `{ "applied": true, "created": [{ "ref": null, "id": "FT-NNN", "file": "..." }], "changed": [] }`
- Running the exact same request a second time against the fresh-state repo produces an identical file (byte-for-byte modulo ID assignment)