---
id: TC-459
title: tag_namespace_format
type: invariant
status: unimplemented
validates:
  features: []
  adrs: []
phase: 1
---

## Invariant

All tags created by Product follow the `product/{artifact-id}/{event}` namespace format.

```precondition
∀ tag T created by Product:
  T.name is a string
```

```postcondition
∀ tag T created by Product:
  T.name matches "product/{ID}/{EVENT}"
  where ID matches [A-Z]+-\d{3,}
  and EVENT matches [a-z][a-z0-9-]*
```

```invariant
No tag created by Product uses a name outside the product/ namespace.
Tag names are deterministic given (artifact_id, event, existing_tags).
```

### Verification
- Unit test on the `create_tag` function: given any artifact_id and event, the resulting tag name matches the pattern `product/{id}/{event}`
- Unit test: `next_event_version` always returns a string that produces a valid namespace tag
- Property test: for random valid artifact IDs and events, the tag name always matches `^product/[A-Z]+-\d{3,}/[a-z][a-z0-9-]*$`