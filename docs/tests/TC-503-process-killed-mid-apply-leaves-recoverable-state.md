---
id: TC-503
title: process killed mid-apply leaves recoverable state
type: chaos
status: unimplemented
validates:
  features:
  - FT-041
  adrs:
  - ADR-038
phase: 1
runner: cargo-test
runner-args: tc_503_process_killed_mid_apply_leaves_recoverable_state
---

Validates FT-041 / ADR-038 decision 10 under adversarial conditions (chaos).

⟦Γ:Invariants⟧{
  ∀step ∈ {6, 7, 9}:
    ∀request:ValidRequest:
      kill(apply(request), at=step)
      ⇒ post_state(repo) ∈ ⟨PreCommit, LockHeldGhost, PartialRename⟩
      ∧ (state = PreCommit ⇒ checksums(pre) = checksums(post))
      ∧ (state = LockHeldGhost ⇒ next_invocation(product) = ClearStaleLock)
      ∧ (state = PartialRename ⇒ graph_check(repo) = DetectInconsistency ∧ rerun(apply, request) = CleanState)
}

**Method:** wrap the request apply pipeline with a fault injection harness that can trigger `SIGKILL` at a specified step. Run the following scenarios:

1. Kill the apply process between step 6 (new-file tmp writes) and step 7 (mutation tmp writes)
2. Kill the apply process between step 7 and step 9 (batch rename commit)
3. Kill the apply process mid-step-9 (between individual rename syscalls, if the OS permits observation)

For each scenario:
1. Record SHA-256 of every artifact file before the apply attempt
2. Start the apply with fault injection configured
3. After the kill, restart Product and run `product request apply` on the same request file (the user's recovery action) or run `product graph check` to observe the post-kill state

**Assert:**
- Scenarios 1 and 2 leave the repo in **PreCommit** state — no `.product-tmp.*` files visible to the user (or cleaned up by next-run scanner per ADR-015), no new artifact files, no mutations applied; checksums match pre-apply
- Or the repo is in **LockHeldGhost** state — `.product.lock` present with dead PID, temp files may be present. Recovered automatically on next Product invocation via stale-lock detection (ADR-015) and temp-file cleanup on startup
- Scenario 3 is the sole exception. Because step 9 is an unavoidable sequence of per-file `rename(2)` syscalls on POSIX, a kill mid-sequence can leave some target files renamed and others not. This documented **PartialRename** window is the one concession to POSIX rename atomicity; the test asserts that `graph check` detects the inconsistency on next run and that re-running `product request apply` on the same request YAML yields the same end state as a clean (non-killed) apply — the request interface is **replay-safe** by construction (idempotent semantics from ADR-038 decision 4 + pre-apply checksum from decision 10)
- After the user's recovery action (re-running apply), the final state matches what a clean apply would have produced for all three scenarios
