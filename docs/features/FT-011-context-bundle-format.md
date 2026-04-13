---
id: FT-011
title: Context Bundle Format
phase: 1
status: in-progress
depends-on: []
adrs:
- ADR-006
- ADR-008
- ADR-012
tests:
- TC-016
- TC-017
- TC-018
- TC-019
- TC-020
- TC-024
- TC-025
- TC-026
- TC-041
- TC-042
- TC-043
- TC-044
- TC-045
- TC-046
- TC-047
- TC-048
- TC-049
- TC-050
- TC-051
- TC-052
- TC-053
- TC-054
domains: []
domains-acknowledged: {}
---

The context command assembles a deterministic markdown bundle. Order is always: feature → ADRs (by ID ascending) → test criteria (by phase, then type: exit-criteria, scenario, invariant, chaos).

The bundle opens with an AISP-influenced formal header block (see ADR-011) that an agent can parse without reading the full document. It declares the bundle's identity, all linked artifact IDs, and aggregate evidence metrics derived from the test criteria evidence blocks.

```markdown
# Context Bundle: FT-001 — Cluster Foundation

⟦Ω:Bundle⟧{
  feature≜FT-001:Feature
  phase≜1:Phase
  status≜InProgress:FeatureStatus
  generated≜2026-04-11T09:00:00Z
  implementedBy≜⟨ADR-001,ADR-002,ADR-003,ADR-006⟩:Decision+
  validatedBy≜⟨TC-001,TC-002,TC-003,TC-004⟩:TestCriterion+
}
⟦Ε⟧⟨δ≜0.92;φ≜75;τ≜◊⁺⟩

---