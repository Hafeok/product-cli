# spec-flow — the agent host

The delegable half of the specification flow, built on the
[Microsoft Agent Framework](https://learn.microsoft.com/agent-framework/)
(`Microsoft.Agents.AI` 1.21.0). The other half is the Rust `spec` binary
(`spec-core` / `spec-cli`), and the split between them is not a packaging
decision — it is the accountability boundary of the flow itself.

## The split

| Verb | Where | Why |
|---|---|---|
| `import`, `candidates`, `map` | .NET (not yet built) | read-only or re-derivable; Roslyn gives entry points from the semantic model |
| `implement` | **.NET, here** | builds a slice; produces a **pending** record |
| `check` | Rust | the gate; a closed class set, none of it configurable |
| **`close`** | **Rust only** | names a principal, and a machine cannot be one |

A model may do everything up to the decision. It may not commit it.

## Why the process boundary is the point

§14.2 of the PRD asks for the restriction to be *structural, not instructed* —
"a rule held in prose is not enforced, and an MCP surface told not to call a
verb is prose." A process boundary is the cheapest structural version of that:
the agent host does not link the code that writes a closure, so there is no
call it could make.

Three things hold it in place, each independently checkable:

1. `SpecCli.ForbiddenVerbs` throws when a `close` is assembled, wherever in the
   argument list it appears.
2. `BoundaryTests.No_public_member_of_the_flow_assembly_closes_a_record`
   reflects over the exported surface and fails if one ever appears.
3. The workflow graph has no edge to a closure — `HandOffExecutor` renders a
   command string and stops.

On the Rust side the same boundary is `S002`, which delegates to the ledger's
`Identity::model_or_bot_reason` — the identical test `L006` applies to an
acceptor. One identity law, two gates.

## Why a workflow rather than an agent loop

The g-track PRD (§7.8) ruled against ceding the loop to an agent runtime
because "the inner planner's context management and prompting are the
runtime's own — an arrangement whose absorbable forms and retrieval habits we
do not author."

MAF's **Workflows** layer does not have that problem. The graph is authored
here: executors, edges, and the order they run in are ours. The framework
supplies the superstep scheduler (deterministic traversal, type-checked edges
at `Build()`, checkpoints at superstep boundaries) and stays out of the
ordering. That is the part worth switching for; the agent layer is incidental.

The one place a model enters is `BuildSliceExecutor`, and its strategy is
*injected* — which model, which endpoint, which instructions stay arrangement
parameters, so the model ladder remains an experiment rather than a constant.

## The graph

```
ImplementRequest
   ↓
[open-record]    → `spec implement` opens the act-time record, first, before any work
   ↓ ActRecordOpened
[build-slice]    → the delegable half; an AIAgent, or anything else you inject
   ↓ SliceBuilt
[draft-closure]  → shapes what arose into a draft
   ↓ ClosureDraft
[closure-draft-review]  ← RequestPort<ClosureDraft, DraftReview>: the human-in-the-loop port
   ↓ DraftReview
[hand-off]       → yields the `spec close …` command. Renders it; never runs it.
```

The record opens *first* on purpose: a record opened after the work is a record
a crashed run never opens, and the write-back leg exists precisely because a
write that can be skipped will be skipped.

## Running it

```bash
cargo build -p spec-cli                       # the Rust half
dotnet test spec-flow/SpecFlow.slnx           # the .NET half (drives the real binary)

dotnet run --project spec-flow/src/SpecFlow.Cli -- \
  --slice checkout-totals --act act/settle-basket --root . --spec target/debug/spec
```

Always exits **3** — work completed, closure pending. Then, as yourself:

```bash
spec close <id> --principal you@example.com --nothing-arose
spec check                                     # 0 only once every record is closed
```

To wire a model, set `SPECFLOW_MODEL_ENDPOINT` (any OpenAI-compatible endpoint
— Scaleway, Ollama, vLLM), plus `SPECFLOW_MODEL` and `SPECFLOW_MODEL_KEY`.

## Known gaps

- **Checkpointing is not wired.** MAF takes checkpoints at superstep
  boundaries, but only `CheckpointManager.CreateInMemory()` is documented for
  .NET — Python has a pluggable `CheckpointStorage`, .NET appears not to. It
  does not block this flow: the act-time record on disk *is* the durable pause,
  and it outlives the process by design. Wire checkpoints only for
  intra-run resilience on long builds, and expect to write the store.
- **Executor identity is fragile.** A checkpoint resumes only into a graph with
  matching executor identities, and ids assigned later do not repair earlier
  checkpoints. `ExecutorIds` holds logical-role constants for this reason; keep
  request and conversation ids out of them.
- **No signing.** `S002` establishes that the named principal does not *look*
  like a machine. It does not establish that the named human closed it. See
  §5 of `docs/spec-flow-act-record-v1.md`.
