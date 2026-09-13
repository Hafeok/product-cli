# Rulings CG-R-51 … CG-R-56 — C# binding Gate 0

*Filed verbatim as received, 2026-09-13. The identifiers are Emil's; this session issues no
rulings and draws nothing from the `CG-R-` space. The register append (`registry/rulings.yaml` in
`canon-governance`, CG-R-17 … CG-R-56) is recorded by Emil as his debt.*

---

**Issued by Emil, 2026-09-13.** Gate 0 ratified. `c5383be0…` is the intended ref — it is where the CG-R-10…16 register rows landed. Future invocations carry the ref rather than a placeholder; that omission is mine, twice now.

---

## CG-R-51 — The delta requires an act vocabulary. Gate 1 splits, and a commercial claim is corrected.

The finding is correct and the defect is in the PRD. Every §5 region and both ratios are defined against an act vocabulary. With none, there is an inventory and no delta.

**This corrects more than the gate.** The claim that this is a day-one qualification instrument — run it on a client's codebase and tell them something they do not know — is wrong as stated. The delta comes *after* modelling, not before it. What can be offered on day one is an inventory and structural measures; the delta is downstream of an act vocabulary somebody has to author.

Better to find that now than in a proposal.

**Ruled: Gate 1 splits.**

**Gate 1a — no vocabulary required.** The .NET inventory tool, the Rust consumer, and reachability measured from conventional entry points found by Roslyn: public API surface, `Main`, framework-registered handlers. **§12.1 is testable here and fires or clears at 1a.** If reachability exceeds the threshold on a real DI-heavy solution, the ratio is dropped before anything is built on it.

**Gate 1b — vocabulary required.** The three regions, the two delta ratios, §12.2.

No heuristic substitutes for the vocabulary at 1b. "A type that reads HTTP context and writes to a DbContext is probably spanning an act boundary" is inference from code, and R-A does not bend because the alternative is inconvenient.

---

## CG-R-52 — The declarable/unstructured separator is accepted as a declared proxy

The proposed criterion — one act's realised facts is declarable, two or more is unstructured — is mechanical, which is what §12.2 needs, and it is a proxy. It is therefore recorded as one, with its three fields:

| | |
|---|---|
| **proxy** | a type references realised facts of two or more acts |
| **original predicate** | the type contains decision logic belonging to more than one act |
| **known divergence** | a shared value object, DTO or mapping type referenced across many acts reads as unstructured while being neither. Shared types are shared, not unstructured. |

**Ruled:** accepted, recorded with the divergence, and the §12.2 prediction is made knowing this is the criterion. A proxy recorded without its divergence would be a coverage claim overstating itself — the defect the binding's own PR-3 exists to prevent.

If the divergence dominates the measurement on the real solution, that is a §12.2 firing, not a reason to add exceptions to the criterion.

---

## CG-R-53 — Pattern-level determinations use the existing travel form; the anchor defect is recorded

`act_instance` is required, so a determination cannot address an act *type*. The proposed form — address one instance, travel to all command slices, profile body bound by content hash — is not a workaround. It is the designed pattern: DSC-0002 in the shipped example is exactly this shape.

**Ruled:** use it. The content-hash binding for the profile body is accepted.

**And record the defect it exposes.** The anchor instance is arbitrary and privileged: a pattern-level determination is filed against whichever instance happened to be chosen, and deleting that slice orphans a determination that was never about it. That is a schema gap in the binding, found by use, and it is the first one found by use rather than by analysis.

Not fixed here. Filed as a candidate schema change against the domain state change binding, with this session as its evidence.

---

## CG-R-54 — `Role.Handler` is struck

Correct, and it is my error. A C# enum of roles compiles the profile into the tool, which is R-C violated in the same document that states R-C.

**Ruled:** roles are strings, resolved against the profile store at check time. An unresolvable role name is a check failure, not a compile error. The PRD's §2 example is corrected.

---

## CG-R-55 — A second reader is admitted, as a stated exception superseding `lsp-as-seam`

Recording it *beside* the existing decision is not enough — that leaves two decisions in tension and the next session gets to pick.

**Ruled: a superseding record**, with the reason stated: LSP is an interactive-query seam, designed for editor request-response against an open document. Whole-solution symbol inventory with a call graph is a batch read, and the seam was never shaped for it. The exception is bounded to batch inventory; interactive queries continue through `ddd-lsp`.

**Two constraints on the exception**, or it becomes a general licence:

1. The workspace tool emits facts only. Every rule, classification and check stays Rust-side against the store (§4). If it starts deciding what counts as a slice, the exception has widened into a second decision engine.
2. Where both readers can answer the same question, `ddd-lsp` is used. The exception covers what LSP cannot do, not what it does less conveniently.

---

## CG-R-56 — PRD defects, corrected

- **§9 pointer dangles** after the renumbering that inserted §10 and §12. Corrected.
- **§7 names a check §6 does not list** — the alongside-mode cutover check. It belongs in §6's table and is added there rather than removed from §7.
- **The handler-role Decider reuse needs a slice-to-aggregate edge**, which the binding's event model does not carry. Recorded as an open item against the binding, not resolved in this session. The Decider rules are reused where the edge exists and the gap is stated where it does not; do not invent an aggregate to close it.

---

## Before Gate 1a

Mine to supply: the named brownfield solution, attached. The permission refusal on a third-party organisation repository was correct conduct and is not worked around.

**The three §12 predictions are committed before any measurement**, per the PRD — and §12.1's now runs at Gate 1a, §12.2's at Gate 1b, with CG-R-52's criterion known.

Register debt: CG-R-17 … CG-R-56. Mine.
