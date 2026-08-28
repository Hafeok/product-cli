# [PROPOSED] Pre-registration: falsifier for the layer closure claim

**Status:** `[PROPOSED]`. Pre-registration. Nothing executed. Per pre-registration discipline: this document is committed before any execution, the selection rules below are fixed at commit time, and there is no re-roll.

**Claim under test.** A specification that passes a layer completeness predicate produces materially fewer escaped decisions in execution than one that does not.

**Why it needs testing before anything is built on it.** The specification language foundation derives its entire construct list from this claim. If closure has no effect on execution, the foundation is a well-organised schema with no warrant.

---

## 1. Restating the prediction sharply

The claim as usually stated — *closure reduces escaped decisions* — is too weak to falsify well, because a specification that passes completeness is likely to be more thorough, and thoroughness alone would produce the effect.

The mechanism claimed is narrower. Declaring the residual does not settle the open decisions. It makes their existence visible to the actor. So the prediction is not that fewer decisions remain open. It is that open decisions **change category**: from silently escaped to explicitly surfaced.

**Primary prediction.** Holding declared content volume constant, a specification passing the completeness predicate shifts the ratio of escaped to surfaced decisions towards surfaced.

**This is falsified** if the ratio does not shift, even if total defect counts differ. A difference in totals without a shift in ratio indicates thoroughness, not closure.

---

## 2. Design

Three arms, matched on content volume. Volume matching is what separates this from a test of effort.

| Arm | Specification |
|---|---|
| **A** | Passes the completeness predicate: every consequential-property category either bound or declared as non-travelling |
| **B** | Fails it: same number of determinations, but some categories silent and undeclared |
| **C** | As B, plus a generic instruction to ask when uncertain |

**Arm C is the load-bearing arm and must not be dropped.** It tests the strongest available objection: that any effect from arm A is reproducible by appending a generic prompt, in which case closure is decorative and the foundation loses its warrant. If C matches A, the claim fails even though A beat B.

Arms A and B must contain the same number of determinations. B is not a degraded specification; it says the same amount and simply does not declare what it omits.

---

## 3. Measurement

**Unit.** A consequential decision identifiable in the produced output.

**Classification**, applied by a reviewer who does not know which arm produced the output:

- **bound** — settled by the specification
- **surfaced** — the actor raised it as a question or flagged it as unsettled rather than resolving it silently
- **escaped** — resolved silently, with consequence, and not settled by the specification

**Primary measure.** escaped / (escaped + surfaced), per run.

**Secondary measures.** Total consequential decisions per run; retry count; whether surfaced questions were answerable from the specification (a question about something already bound indicates delivery failure rather than closure working).

---

## 4. Fixed before execution

These are the items that must not move once results are visible.

1. **The act set.** Drawn from SPETLR act types, which are owned, small, and closed. Selection rule and count fixed at commit.
2. **The consequential-property category list per act type.** Fixed in advance. If this list can be adjusted after seeing results, the completeness predicate is unfalsifiable, because passing becomes a matter of choosing categories.
3. **Number of pairs and runs per pair.**
4. **Effect direction and the threshold below which the result counts as no effect.**
5. **The reviewer classification rubric**, including how disagreements between reviewers are resolved.

---

## 5. Arms and actors

Each arm is run with:

- **A machine actor**, at a fixed model and fixed settings.
- **A human control**, given the same specification and nothing else.

The human control discriminates between two failure causes that otherwise look identical. If the human also escapes decisions under arm B, the specification is missing ground. If only the machine does, the gap is actor capability. Without this arm, every failure will be attributed to model choice, which is the cheapest and usually wrong attribution.

Where feasible, run the machine arm across a capability ladder rather than one model. Monotone improvement with capability locates a capability threshold; uniform failure indicates specification.

---

## 6. Threats, and what each would mean

**Arm C matches arm A.** Closure is doing nothing a generic instruction does not. Claim fails. This is the outcome to expect if the effect is prompt-shaped rather than structural.

**Reviewer classification is unblinded or drifts.** Escaped and surfaced are judgement calls, and a reviewer who knows the arm will classify towards expectation. Blinding is mandatory; inter-reviewer agreement is reported.

**Category list tuned after the fact.** Would make the predicate unfalsifiable. Fixed at commit, no exceptions.

**Volume matching fails.** If arm A specifications end up longer, the test measures effort. Word or determination counts are reported per arm and a material imbalance invalidates the run.

**Surfaced questions that were already bound.** Indicates the specification was not delivered properly at act time, which is a different defect. Counted separately, not as evidence for or against closure.

---

## 7. What this test does not establish

- Not that the specification is correct. Only that declaring the residual changes actor behaviour.
- Not that closure scales beyond the act types tested.
- Not anything about accrual, which is a separate claim with a separate falsifier.
- Not that closure improves outcome quality. The measure is decision category, not defect severity.

---

## 8. Feasibility

This is runnable without a decision store, a client corpus, or the extractor. It needs a small act set, a fixed category list, a harness, and blinded reviewers. It is a session, not a project.

The accrual falsifier is not in this position — it requires an operating store and a corpus — which is a reason to run this one first and treat the accrual claim as remaining `projected` for longer.

---

## 9. Consequence for downstream work

Until this executes, the specification language foundation rests on an unfalsified proposition. Draft work may proceed, and must record that dependency rather than gloss it. If the claim fails, the foundation does not collapse — the constructs still make the residual expressible — but the warrant for requiring closure at a layer would be gone, and the design driver in section 1 of the foundation would need replacing.
