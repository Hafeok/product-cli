---
id: TC-528
title: log replay produces same graph
type: invariant
status: passing
validates:
  features:
  - FT-042
  adrs:
  - ADR-039
phase: 5
runner: cargo-test
runner-args: tc_528_log_replay_produces_same_graph
last-run: 2026-04-18T10:42:11.121063959+00:00
last-run-duration: 0.3s
---

## Description

Replay of the log produces the same graph as the files on disk. This is the equivalence proof between the two representations of graph state.

## Formal

⟦Σ:Types⟧{
Log ≜ Entry+
Graph ≜ ⟨features: Artifact*, adrs: Artifact*, tests: Artifact*, deps: Artifact*, links: Link*⟩
files ≜ Log → Directory
replay ≜ Log → Directory
graph ≜ Directory → Graph
}

⟦Γ:Invariants⟧{
∀ log ∈ Log: graph(replay(log)) = graph(files(log))
}

## Property test

For all generated `log` values (via `proptest` strategy producing a sequence of valid `create` / `change` / `create-and-change` / `undo` / `verify` entries, each chained correctly):

1. Apply each entry in sequence to a fresh repository skeleton via the real apply pipeline, producing a directory `A`.
2. Run `replay --full` against `log` into a separate directory `B`.
3. Parse the graph from `A`'s `docs/` tree and the graph from `B`'s `docs/` tree.
4. Assert the two graphs are equal: same artifact set, same IDs, same front-matter fields, same links, same hashes.
5. Additionally assert byte-equality of every file under `docs/`.

## Pinning integration test

A non-property integration test replays the real project log (or a fixture modelled on it) against the working tree and asserts byte-equality.

## Invariant

**The most important invariant in this feature.** If replay ever diverges from apply, the log is decorative. Everything in ADR-039 hinges on this property holding.