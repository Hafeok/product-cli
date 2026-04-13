---
id: ADR-010
title: Auto-Orphan Test Criteria on Feature Abandonment
status: accepted
features: []
supersedes: []
superseded-by: []
domains: []
scope: domain
---

**Status:** Accepted

**Context:** Test criteria are linked to features via `validates.features` in their front-matter. When a feature is marked `abandoned`, the tests that validated it have no active feature to belong to. The question is whether Product should require the developer to manually clean up those links, or handle it automatically.

**Decision:** When a feature's status is set to `abandoned` (via `product feature status FT-XXX abandoned`), Product automatically removes that feature's ID from the `validates.features` list of all linked test criteria. Test criteria that end up with an empty `validates.features` list are orphaned. `product graph check` reports them as warnings (exit code 2). No test criteria are deleted.

**Rationale:**
- Requiring manual cleanup is friction that will routinely be skipped. Orphaned tests with stale feature links produce silent graph inconsistencies — `product graph check` reports the link as broken (exit code 1), blocking CI, for a situation that is not actually an error
- Auto-orphaning on abandonment is the less surprising behaviour: the developer said the feature is gone; Product cleans up the edges
- Tests are not deleted because they may still be useful: they can be re-linked to a successor feature, or they document behaviour that was specified but not built
- Orphaned tests surface as warnings, not errors. A warning prompts the developer to decide: re-link, or explicitly delete. An error would block CI for something that requires a judgment call
- The mutation is logged to stdout during the command so the developer sees exactly what was changed

**Rejected alternatives:**
- **Require explicit `product test unlink TC-XXX --feature FT-001`** — correct but creates friction. Abandoned features often have several linked tests. Requiring individual unlinking is a multi-step cleanup that will be deferred or forgotten.
- **Delete tests automatically** — too destructive. A test criterion represents specified behaviour. Deleting it erases the record that the behaviour was ever intended. Orphaning preserves the history.
- **No action — leave stale links** — stale links produce broken-link errors in `product graph check`. This would cause CI failures for abandoned features, which is a false positive. Not acceptable.