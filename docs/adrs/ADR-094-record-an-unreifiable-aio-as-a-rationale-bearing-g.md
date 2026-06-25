---
id: ADR-094
title: Record an unreifiable AIO as a rationale-bearing gap
status: accepted
features:
- FT-152
supersedes: []
superseded-by: []
domains: []
scope: domain
---

**Status:** Accepted

**Context:**

Reification was modelled positively only (reify(AIO, context) -> CIO); a missing
reification surfaced as an absence, indistinguishable from an oversight.
Framework §4.5 says some (AIO, interaction class) pairs are genuinely
unreifiable — a `display-collection` of images has no faithful TUI form — and
that the framework should *record* the boundary with a rationale rather than
paper over it.

**Decision:**

Model the gap as a first-class `UnreifiableRule` node carrying the AIO, the
interaction class (gui/tui), and a required rationale. Validate that the AIO is
recognised, the class is a recognised core class, and the rationale is present —
because the entire point of the construct is that the gap is *recorded with a
reason*, never a silent omission. Defer the seam-verification consumption (a UI
step using an unreifiable AIO in a targeted class is a finding; a declared gap
satisfies coverage) to a later pass, since it depends on the
step->flow->system->class link.

**Rationale:**

A recorded gap with a rationale is the same honesty the framework already applies
to manual WCAG criteria and the Polanyi floor: name the boundary instead of
pretending it does not exist. Requiring the rationale at validation time is what
makes the node meaningful — an unreifiable rule with no reason would be exactly
the silent omission §4.5 exists to prevent.

**Rejected alternatives:**

- **Infer "unreifiable" from the absence of a reification rule.** Rejected: an
  absence cannot be distinguished from an oversight and carries no rationale,
  which is precisely the silent gap the spec forbids.
- **A boolean flag on the ReificationRule.** Rejected: a positive reification
  targets a CIO, while an unreifiable gap has no CIO and must carry a rationale;
  conflating them muddles both shapes.

**Test coverage:** TC-1036.
