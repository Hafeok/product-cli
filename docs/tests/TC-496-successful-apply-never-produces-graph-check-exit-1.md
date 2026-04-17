---
id: TC-496
title: successful apply never produces graph check exit 1
type: invariant
status: unimplemented
validates:
  features: []
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_496_successful_apply_never_produces_graph_check_exit_1
---

Validates FT-041 / ADR-038 decision 8 — invariant.

⟦Γ:Invariants⟧{
  ∀request:ValidRequest:
    apply(request) = Success
    ⇒ exit(graph-check(post-apply-repo)) ∈ {0, 2}
    ∧ exit(graph-check(post-apply-repo)) ≠ 1
}

**Method:** property test (`proptest`) generates arbitrary well-formed request YAMLs that pass validation. For each generated request:
1. Apply it to a fresh fixture repo
2. If apply exits 0, immediately run `product graph check`
3. Assert graph-check exits either 0 (clean) or 2 (advisory W-class findings only), never 1 (E-class)

**Rationale:** a successful apply must leave the graph structurally valid. Exit 1 after apply means the request layer let through a state that `graph check` rejects, which is a Product bug. This invariant keeps validation-before-apply honest.

The test seed covers: all three request types, multiple artifact types, requests with forward refs, requests with bidirectional supersession, requests with `create-and-change` mixing both sections.
