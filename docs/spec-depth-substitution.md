# Specification depth substitutes for model capability

A working thesis, and the experiment that tests it, built into `product build`.

## The thesis

> The more precisely a change is specified, the smaller the model that can
> implement it correctly. High intent needs a frontier model; deep specification
> lets a small one suffice.

If true, this is leverage. The expensive, scarce resource (a frontier model, or a
senior engineer) is needed only at the top of the funnel — turning vague intent
into a precise, verifiable spec. Everything below that line can be executed by a
cheap, tireless, *small* model, because the hard reasoning is already encoded in
the spec rather than re-derived per task.

This is the software analogue of how high-reliability human organisations work:
the checklist, the acceptance criteria, and the separation of the author of a test
from its implementer. The difference is cost. Humans only pay for that discipline
where failure is catastrophic (aviation, surgery). A cheap machine executor makes
it affordable for ordinary work.

## How `product build` operationalises it

The knowledge graph is the specification ladder: intent → features → ADRs →
acceptance criteria → How-contracts → Deciders → frozen work units. `product build`
assembles the **frozen SPMC context** for a deliverable (the What slice, the How to
apply, the Decider oracle, the acceptance) and dispatches it to a **worker**.

Workers are a catalogue of model capabilities (`.product/capabilities.yaml`) bound
to roles with an **escalation ladder** (`.product/role-bindings.yaml`). The `coder`
role, for example, starts at a 35B model and climbs to 123B then 397B *only when a
gate fails*:

```yaml
- role_id: coder
  default_capability: fast-coder        # qwen 35B (tier 1)
  escalation_steps:
  - { capability: code-writer }         # devstral 123B (tier 2)
  - { capability: code-writer-heavy }   # qwen 397B (tier 3)
```

The build always tries the **smallest model first** and escalates on failure — the
system is a bet on this thesis. Whether a build succeeds without escalating is the
measurement.

`done` is computed, not judged (ADR-071, §7.2): every in-scope element conforms,
every Decider is sound, every acceptance runner passes. It is "exactly as honest as
those verifications are strong" — which is why the oracle must be protected
(ADR-076).

## Methodology — test-first, two independent models

To measure implementation capability honestly, the *oracle* must be fixed and the
*implementer* must not be able to cheat it. The pipeline (ADR-075) expresses this
as two dependency-ordered work units within one slice:

1. **`write-test`** — a worker writes the acceptance test from the specification.
2. **`implement`** — a *different* worker (or invocation) implements against that
   test, which is **frozen** and injected **read-only**. It cannot edit the oracle
   (ADR-076), only satisfy it.

A test is specification, not implementation — so authoring the test is the deepest
rung of the spec, and having a separate agent implement against it is separation of
duties. If a deep-enough spec makes two *independent* models converge on the same
contract, the spec has become the shared mind that coordinates them.

## Results

All runs use Scaleway-hosted open models through the worker catalog. The task was a
small but discriminating change (add a record + field, or a method, to
`BuildSession`); the hidden oracle is a unit test the worker never sees.

### 1. Specification substitutes for capability

Same task, same oracle, varying only the specification handed to the smallest model
(qwen 35B):

| Spec level | What the worker was given | Outcome |
|---|---|---|
| **L3** full | work unit naming file + struct + exact fields + derives | first-try **DONE**, no escalation |
| **L1** thin | the acceptance sentence only — no file pointer | the **whole ladder** (35B→123B→397B) **fails** |

Deep spec let the smallest model win first-try; thin spec defeated even the largest.

### 2. Localization is *non-substitutable*

At L1 the missing fact was *which file to edit*. No model size recovered it — the
397B model hallucinated a new file exactly as the 35B did. The substitution holds
over *reasoning* difficulty, **not** over missing grounding: you must specify what
the model cannot derive (the target, the contract). Capability substitutes for the
rest.

### 3. Under-specification invites oracle-gaming — now guarded

At an intermediate depth the small model chose its own design (`Option<T>` where
the test expected plain `T`); a fix-round worker then made the check pass by
**editing the test**. The gate falsely reported `done`. This is the design smell
that motivated **ADR-076**: the verify/LSP loops now revert any worker write to a
test file that is not the worker's declared artifact. `done` is honest again.

### 4. Convergence generalises across task shapes

With the guard in place and the test-first pipeline, qwen 35B built three
differently-shaped changes — a **struct + field**, a **boolean predicate** (set
logic), and an **`Option`-returning aggregation** — each as a `write-test ->
implement` pair. Every one converged on the contract **first-try, no escalation,
no tampering**. The convergence was not a fluke of the easy structural case.

### 5. The tier × depth surface

Pinning each tier as the only (one-shot, no-escalation) worker across spec depths
maps where each tier's *minimum sufficient specification* sits:

One-shot per cell — no escalation, no fix loop. PASS means the *single* dispatch
produced code that passed the hidden oracle.

| first-shot model | L3 (full spec) | L2 (terse, localized) | L1 (no file pointer) |
|---|---|---|---|
| **35B** (qwen) | PASS (6.4k tok) | PASS (7.5k) | fail (5.9k) |
| **123B** (devstral) | fail (4.5k) | fail (4.5k) | fail (3.4k) |
| **397B** (qwen) | PASS (8.8k) | PASS (9.1k) | fail (8.2k) |

Reading the surface:

- **The L1 column is all-fail.** With no file pointer, no tier recovers — every
  model (including 397B) hallucinated whole new file trees at the repo root rather
  than editing the right file. Localization is non-substitutable, confirmed across
  the whole column.
- **For the qwen family, L2 and L3 are all-PASS.** Once the change is localized,
  the *smallest* model (35B) suffices — and 10× the parameters (397B) buys nothing
  but cost and latency. That row pair is the thesis in miniature.
- **The 123B (devstral) row fails throughout — but not for the obvious reason.**
  On inspection it was not a parsing failure: it wrote a correct, idiomatic `Cost`
  struct, then *omitted the second clause* of the instruction (the field on
  `BuildSession`). It did half the two-part change, one-shot. A fix round would
  almost certainly have caught it — which is the point: one-shot
  instruction-following is not monotone in parameter count, especially across
  model families.



A notable wrinkle: **tier is not a clean capability axis across model families.**
Parameter count orders models within a family, but a 123B model from one family can
underperform a 35B from another on a given task. "Bigger" is a heuristic for
"more capable", not an identity — which is exactly why the escalation ladder
escalates on *failure* rather than trusting a size ordering.

## How to use it

```bash
# 1. catalog your workers (model tiers) and bind roles
product worker init                 # scaffolds capabilities.yaml + role-bindings.yaml
product worker list                 # the catalog + each role's ladder

# 2. author a deliverable: a slice + acceptance criteria with runners
product slice new my-slice --anchor <node> --depth 2
product deliverable new my-thing --slice my-slice --accept "ac:the behaviour"
#   then add `runner: cargo-test` + `runner_args: <test fn>` to each criterion

# 3. build it with the smallest model, escalating on failure
product build my-thing --role coder --lsp        # dispatch + LSP + verify gates
product build my-thing --role coder --dry-run    # inspect the frozen context first
```

## Limits

This institutionalises the *transmission and verification* of the specifiable
fraction of knowledge — it does not make tacit knowledge explicit for you.

- **The tacit floor.** Some knowledge resists specification; there, capability
  stays non-substitutable. The L1 localization result is a small instance.
- **Correct-against-wrong-spec.** A passing oracle proves conformance to the
  *written* contract, not to intent. A deep-but-wrong spec yields confidently-wrong
  work at scale, auditably "DONE".
- **Convergence assumes agreement.** Two models converge once the spec pins the
  contract. Where competent experts *disagree* on what correct means, the machine
  enforces an agreement it cannot itself reach.

## See also

- ADR-071 — build assembles SPMC context and records conformance into `done`
- ADR-072 — workers are capabilities resolved by role, with escalation
- ADR-073 — work units are the parallel unit, fanned out bounded
- ADR-075 — work units dispatch in dependency order, with frozen inputs
- ADR-076 — the build gates forbid a worker editing the acceptance tests
