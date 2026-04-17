---
id: TC-498
title: failed apply leaves every file unchanged
type: invariant
status: unimplemented
validates:
  features: []
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_498_failed_apply_leaves_every_file_unchanged
---

Validates FT-041 / ADR-038 decision 10 — invariant.

⟦Γ:Invariants⟧{
  ∀request:MalformedRequest:
    apply(request) = Failure
    ⇒ ∀file ∈ artifacts(repo):
        sha256(pre_apply(file)) = sha256(post_apply(file))
    ∧ ¬∃new_file ∈ post_apply(repo) ∖ pre_apply(repo)
}

**Method:** property test. For each of several failure injection scenarios:

1. Request with an E-class finding discovered during validation (fails at step 2 of the pipeline — never reaches write)
2. Request that passes validation but hits a simulated I/O failure during step 6 (new-file write)
3. Request that passes validation but hits a simulated I/O failure during step 7 (mutation write)
4. Request that passes validation but hits a simulated I/O failure during step 9 (rename commit point)

For each scenario:
1. SHA-256 every file under `docs/features/`, `docs/adrs/`, `docs/tests/`, `docs/deps/` before apply
2. Attempt the apply
3. SHA-256 every file after apply
4. Assert: apply exited non-zero, every pre-hash equals its post-hash, no new files exist in those directories, no `.product-tmp.*` files remain

**Rationale:** the zero-files-changed invariant is the atomicity promise the request interface makes. Failure scenarios 2–4 specifically exercise the batch-write-tmp + batch-rename pattern from ADR-038 decision 10 — the pattern must restore cleanly from any failure point.
