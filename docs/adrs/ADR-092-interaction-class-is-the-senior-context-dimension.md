---
id: ADR-092
title: Interaction class is the senior context dimension with a closed core
status: accepted
features:
- FT-150
supersedes: []
superseded-by: []
domains: []
scope: domain
---

**Status:** Accepted

**Context:**

Context of use was modelled as flat `dimension`/`value` pairs with no notion that
one dimension is senior to the others. Framework §3.2.2 establishes the
interaction class (GUI / TUI) as the *gating* dimension: it is chosen first and
selects which sub-dimensions apply. There is no node kind for registering an
adopter-defined class, unlike AIOs which register as `Aio` nodes.

**Decision:**

Recognise a closed core of interaction classes — `gui` and `tui` — as a named
constant (`CORE_INTERACTION_CLASSES`). A `System`'s `target_classes` and any
`ContextOfUse` declaring the `interaction-class` dimension are validated against
that core; an unrecognised class is a finding. Platform remains an open
dimension (no value restriction) because platforms genuinely vary and are not a
gating choice. Per-class reification coverage and the `unreifiable_in` recorded
gap are deferred to the §4.5 feature.

**Rationale:**

With no class-registration mechanism, a closed core is the only checkable
option, and it mirrors `CORE_AIOS` (closed core, adopter extends via a node).
Validating the gating dimension now — cheaply, on the System and ContextOfUse —
gives reification rules a settled dimension to be written against, without
touching the design-system coverage logic yet.

**Rejected alternatives:**

- **Leave interaction class as an unvalidated free-text dimension.** Rejected:
  it could not catch a typo'd or meaningless class, and gives the seam no senior
  dimension to gate coverage on.
- **Add an InteractionClass node kind for registration now.** Rejected as
  premature: the core suffices, and a registration node can be added later if an
  adopter needs voice-only or another class, exactly as AIOs do.

**Test coverage:** TC-1034.
