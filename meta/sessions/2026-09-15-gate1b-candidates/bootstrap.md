# Bootstrap — Gate 1b: entry-point candidates and the delta

**Session:** `2026-09-15-gate1b-candidates`
**Session kind:** measurement under gates, continuing the C# stack binding
(`2026-09-13-implement-csharp-binding/`, held at Gate 1a after run 8). Gate 1b was blocked on
an act vocabulary; CG-R-105 unblocks it with a vocabulary derived from external integration
points.
**Principal:** Emil. The session proposes; Emil ratifies. Each gate holds until an explicit
ratification message.
**Commit identity:** `Claude <noreply@anthropic.com>` — session-neutral, as before.

---

## Invocation

The chat message is filed verbatim as `invocation.md`. The instruction set is filed verbatim as
`prompt.md` (`session-gate1b-candidates.md`, sha256 `f067201d…`); the rulings as
`inputs/rulings-cg-r-105-110.md` (sha256 `565cc7cc…`). Where the message and the prompt
disagree, the prompt governs and the disagreement is reported at Gate A. None found on reading.

## Parameters

| Parameter | Value |
|---|---|
| Repository | `Hafeok/product-cli` |
| Branch | `claude/csharp-stack-binding-dbx7vr` |
| Base commit | `8a33ab962223781bf16d4aac6584be57f0fc44db` (run 8 committed) |
| Governance | `Hafeok/canon-governance` at `c5383be0…`, as bound at the prior bootstrap; not re-read |
| Solution | **A only** — eShopOnWeb at `03d8cff`, the run-8 inventory (`3197d7e0…`) |
| Instrument | reader v6 and the Rust consumer at the base commit, **closed** (CG-R-103 as the prompt restates it) |
| Gates | **A** the candidate set · **B** the delta · **C** the report — each held |

## The vocabulary, and what it is not

CG-R-105: two vocabularies, graded differently. This gate derives the **measurement**
vocabulary — one candidate per external integration point, transport-shaped by construction.
Every figure computed against it is labelled *transport-derived*. It is not an accrual
vocabulary: no determination is written against it, and no candidate is an act until someone
names it (CG-R-106).

## Rules in force that bind this gate

- **CG-R-105** measurement vocabulary from entry points; every figure transport-derived.
- **CG-R-106** a candidate is a prompt: transport name, path, observed positions; no act name,
  no "what it settles", no "who answers". Merges and splits are the signal.
- **CG-R-107** `channel` is a named extent axis — bears on determinations later, not here.
- **CG-R-108** observed actor evidence labelled *observed*; expected actor kinds, population,
  rate are **unfilled ground slots**, and unfilled reads *not stated*, never *no actors*.
- **CG-R-109** supported throughput (sustained, peak, window, behaviour above the limit) is an
  **invited determination**: the field is carried, empty.
- **CG-R-110** `[PROPOSED]`, no falsifier: scale sets the consequence of escape. Bears on how
  Gate C weights the uncovered list; not on what Gate A or B measures.
- Carried from Gate 1a: CG-R-52 (the separator is a declared proxy), CG-R-57 (attribution
  graded), CG-R-62 (never fold), CG-R-71 (recall against a hand enumeration), CG-R-77 (every
  proxy carries its incidence or says *unmeasured*), CG-R-78 (the Razor blind spot beside every
  headline), CG-R-88 (a step that does not run is a failure of the run), CG-R-89 (the unscored
  fraction is the error bound), CG-R-98 (scoped termination).

## Prohibitions

From the prompt, in force for every gate here:

- **Do not name acts.** No act name, no settlement, no answerer on any candidate.
- **Do not generate slice declarations.** R-D: the candidate set is a prompt, never a queue.
- **Do not infer expected actors or scale.** Unfilled slots stay unfilled.
- **Do not author the invited determination.**
- **Do not treat a library's public surface as entry points.**
- **Do not improve the reader.** A defect found is reported, not repaired.
- **Do not proceed past a gate without ratification.**
- Standing: never infer from naming (attributes and framework APIs by resolved symbol id);
  where a framework's *own* discovery rule is a name convention (Razor page handlers,
  ViewComponent `Invoke`), the rule is declared as a proxy with the framework's rule cited and
  its incidence measured — never as this session's inference.

## What Gate A will do, stated before it runs (CG-rule-08)

1. State the entry-point criterion — which inventory facts make a symbol an external
   integration point, by symbol id — in `gateA-criterion.md`, with every proxy and every known
   instrument limit named, **before** any candidate is computed.
2. Build the consumer-side derivation (`pf/csharp_candidates*.rs`, `product csharp candidates`)
   over the existing walk. The reader is not touched. The existing measurements must not move:
   the run-8 A reach output is regenerated with the new binary and diffed; a difference is a
   defect of this gate.
3. Run it on A's run-8 inventory through a self-building script with an instrument record
   (CG-R-88), and check the reader's recall against a hand enumeration of A's entry points from
   source (CG-R-71's form — the enumeration is committed first).
4. Report the candidate set, the count, the overlaps, the blind spot, the limits — and hold.

## Environment facts at bootstrap

- The run-8 A inventory and the eShopOnWeb checkout are still present in the session scratch
  directory; the reader is not rebuilt or re-run. If the inventory were lost, it would be
  regenerated from the same checkout with the same reader (v6) and its hash compared.
- The .NET SDK remains installed for the session; it is not used by this gate.
- Rulings CG-R-99 … CG-R-104 were not received (see `arrived-inputs.md` §4).

## What this record is not

Not a plan of the candidate set and not a design of the delta. The criterion is the first
proposal, held like every other.
