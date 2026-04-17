---
id: TC-504
title: request interface ready for production use
type: exit-criteria
status: unimplemented
validates:
  features:
  - FT-041
  adrs:
  - ADR-038
phase: 1
runner: cargo-test
runner-args: tc_504_request_interface_ready_for_production_use
---

Exit criteria for FT-041. The request interface is considered ready for production use when all of the following hold:

1. Both MCP tools (`product_request_validate`, `product_request_apply`) and all six CLI commands (`request create`, `change`, `validate`, `apply`, `diff`, `draft`) exist and behave per the spec (`docs/product-request-spec.md`).
2. All three request types (`create`, `change`, `create-and-change`) accept well-formed YAML matching the spec and apply atomically.
3. Every validation rule listed in ADR-038 and the spec validation table (§Within the request, §Against the existing graph, §Advisory) is enforced, with findings carrying the documented `code`, `severity`, `message`, and JSONPath `location` fields.
4. The 13-step apply pipeline (FT-041 §Apply pipeline) is implemented end-to-end, including: pre-apply checksum snapshot, batch-write-tmp + batch-rename, post-apply `graph check` health monitor, `.product/request-log.jsonl` append.
5. All four invariants (failed-apply zero-files-changed, successful-apply graph-check-never-exit-1, validate-never-writes, append-remove-idempotent) are verified by TCs TC-496 and TC-498 passing.
6. TC-486 through TC-503 all pass (`product verify FT-041` clean).
7. Graph health: `product graph check` clean (exit 0) in the repository; `product gap check` on ADR-038 closes G001 (every decision in ADR-038's test coverage table has at least one passing TC linked).
8. Documentation: the spec `docs/product-request-spec.md` matches implementation behaviour; the author-feature and author-adr prompts (FT-022) are updated to mention the request interface as the preferred multi-artifact entry point; the CHECKLIST.md regeneration reflects FT-041 complete.
9. Coexistence demonstrated: TC-502 passes showing granular tools continue to function alongside the request interface with no deprecation warnings.
10. Drift check clean: `product drift check` finds no spec-vs-code gaps for `src/request.rs` (the primary implementation module declared in ADR-038 `source-files`).

When all ten conditions hold, FT-041 can be marked `complete` and ADR-038 moved from `proposed` to `accepted` per ADR-034 (Lifecycle Gate).
