# Session: closure falsifier — pre-registration artefacts

**Session kind:** proposal only. This session produces no executable code and runs no experiment.

**Commit this prompt to `meta/sessions/` before any canonical act, per `DDD-dec-20`. Use the session-neutral commit identity, not a personal email.**

---

## Standing rules for this session

- **You propose. Emil merges and ratifies.** Nothing you produce is ratified by your having produced it, and you must not proceed past a gate on your own assessment that the output looks correct.
- All output is marked `[PROPOSED]`.
- Three-register discipline: keep Rulings, `[PROPOSED]`, and `[OPEN]` explicitly distinct in every gate output.
- British spelling.
- Supersession never rewrites. If you find an error in earlier output, note it and correct going forward. Do not force-push. Do not fix-and-re-record.
- Report defects honestly, including defects in your own prior gate output within this session.
- Name the weakest link in each proposal rather than leaving it for a sceptic.

## Explicit prohibitions

You must not, in this session:

- write any harness or experiment code
- write the arm A, B or C specifications
- execute anything
- adjust the claim under test to make it easier to satisfy
- proceed past a gate without an explicit ratification message

---

## Context

The claim under test: *a specification passing a layer completeness predicate produces materially fewer escaped decisions in execution than one that does not.*

The sharpened prediction, which is what will actually be measured: holding declared content volume constant, closure shifts open decisions from **escaped** (resolved silently) to **surfaced** (raised as questions), rather than reducing the number of open decisions.

Read `prereg-closure-falsifier.md` before starting. It is the design; this session produces the four artefacts that design leaves open.

The reason those four are open is that they determine what "complete" means. If they are drafted and approved in the same session, the pre-registration is self-ratified and worthless.

---

## Gate 0 — orientation

Read the pre-registration document and the specification language foundation. Then produce, without writing anything to the repository:

1. A restatement of the claim under test and the primary measure, in your own words.
2. A list of anything in the pre-registration you believe is unsound, ambiguous, or unfalsifiable as written.
3. Confirmation of what SPETLR's act types are, from the public repository, and whether the act type set is genuinely closed.

**Hold.** Do not proceed until ratified.

---

## Gate 1 — act set selection rule

Propose a rule for selecting the acts the experiment runs over.

Requirements:

- The rule must be stated such that a third party could apply it and reach the same set.
- It must be fixed before any specification is written.
- SPETLR act types are the intended source. Justify the choice or propose better.
- State how many acts, and why that number.

State the weakest point in the rule.

**Hold.**

---

## Gate 2 — consequential-property category list

This is the critical artefact of the session. Take it slowly.

Propose, per act type, the list of consequential-property categories against which completeness is assessed. A specification passes the completeness predicate when, for every act and every category, it either binds a determination or declares that no determination travels there.

Requirements:

- Categories must be recognisable rather than requiring recall. A reviewer should be able to say whether a category is addressed without first reconstructing what the categories should have been.
- The list must be fixed before execution and never adjusted afterwards. If it can be tuned once results are visible, passing becomes a matter of choosing categories and the predicate is unfalsifiable.
- State explicitly what the list omits and why.

**Warn yourself here.** You will be able to generate a plausible, well-organised category list quickly. Plausible is not pre-registered, and fluency is not warrant. Present the list with its derivation, so that the basis of each category can be checked rather than accepted on presentation.

**Hold.**

---

## Gate 3 — quantities and thresholds

Propose:

- number of acts, arms per act, runs per arm
- the effect threshold below which the result counts as no effect
- how volume matching between arms will be measured and what imbalance invalidates a run
- how variance across runs will be reported

State the smallest effect the design could detect, and say plainly whether that is smaller than the effect worth caring about.

**Hold.**

---

## Gate 4 — classification rubric

Propose the rubric a blinded reviewer applies to classify each consequential decision in an output as **bound**, **surfaced**, or **escaped**.

Requirements:

- Usable by a reviewer who has not read this session.
- Includes worked examples of each class, and at least two borderline cases with the ruling stated.
- Specifies how two reviewers disagreeing is resolved.
- Specifies the separate handling of surfaced questions about matters the specification already bound, which indicate a delivery defect rather than evidence about closure.

**Hold.**

---

## Gate 5 — assembly

Assemble the four ratified artefacts into a single pre-registration file for commit. No new content. Nothing that was not ratified at gates 1–4.

Then stop. The harness is a separate session, and it must run against the committed file.

---

## Open items to carry forward

Record these in the session output rather than resolving them:

1. Whether arm B should be mechanically derived from arm A by stripping non-travel declarations only, leaving all bound determinations intact. This would remove the risk of arm B being an unintentional straw man, and would make volume matching automatic. It is an amendment to the pre-registration and is legitimate now, before commit, and not afterwards.
2. Whether the human control arm is feasible with available people, and what the experiment loses without it.
3. Whether the machine actor should be a single model or a capability ladder.
