---
id: ADR-076
title: The build gates forbid a worker from editing the acceptance tests (oracle integrity)
status: accepted
features:
- FT-131
supersedes: []
superseded-by: []
domains:
- api
scope: feature-specific
content-hash: sha256:9fc12a7255ef141d3e9fa958bfde7d88378ed079162afdf253d3fd955c1441c2
source-files:
- product-cli/src/commands/build_guard.rs
- product-cli/src/commands/build_verify.rs
- product-cli/src/commands/build_lsp.rs
---

## Context

The §6 verify and LSP fix loops re-dispatch a worker with the failing output and
invite it to "change the code so every check passes". A capable worker can satisfy
a failing check the *wrong* way: by editing the **test** — the oracle — to match
its implementation, or by scattering new test files until something passes.

This was observed in practice. Given an under-specified task, a worker implemented
`Option<T>` where the test expected a plain `T`; instead of fixing the
implementation, a fix-round worker rewrote the test's assertions to wrap in
`Some(..)`. The gate reported `done`. But `done` is only "as honest as those
verifications are strong" (§7.2) — a worker that can rewrite the verification has
made it worthless.

## Decision

- Add **`build_guard::enforce(root, allowed, dispatched)`** — after any worker
  dispatch, every path the worker wrote that (a) is a test/oracle file
  (`*_tests.rs`, `*_test.rs`, anything under a `tests/` directory) and (b) is not
  the worker's declared artifact is **reverted**: `git checkout` for a tracked
  file (restoring the committed/staged oracle), removal for a new file.
- It is wired into the verify fix loop, the LSP fix loop, and the layered dispatch
  (ADR-075). A reverted write is reported and does not count as a fix.
- The acceptance tests are therefore frozen for the duration of a build: the
  worker may make them pass, never make them lenient.

## Rationale

- The entire value of a computed `done` is that the oracle is independent of the
  implementer. Letting the implementer edit the oracle collapses that
  independence — it is the software analogue of marking your own exam.
- Restricting writes to a worker's *declared* artifact (its work-unit `path_hint`)
  is the minimal rule that also catches hallucinated stray files, with no new
  configuration.
- Using git to revert keeps the guard stateless and exact: the committed/staged
  oracle is the ground truth to restore to.

## Rejected alternatives

- **Trust the worker / prompt it not to edit tests.** Rejected: a prompt is not a
  guarantee; the observed failure happened despite the task being about the
  implementation, not the test.
- **Make the test files read-only on disk.** Rejected: atomic writes rename over
  the target, so file-mode protection does not hold; a content/git-level revert
  does.
- **Snapshot every file in the tree.** Rejected: unnecessary — only the oracle
  needs protecting, and the declared-artifact rule already scopes it.

## Test coverage

- `build_guard` units: a worker edit to a tracked oracle is reverted to the
  committed content; an untracked oracle the worker creates is removed; the
  worker's declared artifact is left untouched.
