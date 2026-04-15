---
id: TC-459
title: tag_namespace_format
type: invariant
status: unimplemented
validates:
  features: [FT-037]
  adrs: [ADR-035, ADR-036]
phase: 1
---

## Invariant

All tags created by Product follow the `product/{artifact-id}/{event}` namespace format.

## Formal Specification

⟦Σ:Types⟧{
  Tag ≜ { name: String }
  ArtifactId ≜ String  // matches [A-Z]+-\d{3,}
  Event ≜ String       // matches [a-z][a-z0-9-]*
}

⟦Γ:Invariants⟧{
  ∀t:Tag created by Product: t.name matches "product/{ID}/{EVENT}"
    where ID matches [A-Z]+-\d{3,} ∧ EVENT matches [a-z][a-z0-9-]*

  ∀t:Tag created by Product: ¬starts_with(t.name, prefix) for prefix ∉ {"product/"}

  ∀(aid:ArtifactId, e:Event, tags:Set<Tag>): create_tag(aid, e, tags) is deterministic
}

### Verification
- Unit test on the `create_tag` function: given any artifact_id and event, the resulting tag name matches the pattern `product/{id}/{event}`
- Unit test: `next_event_version` always returns a string that produces a valid namespace tag
- Property test: for random valid artifact IDs and events, the tag name always matches `^product/[A-Z]+-\d{3,}/[a-z][a-z0-9-]*$`