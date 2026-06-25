---
id: ADR-091
title: Trigger generalises user external automated as one block
status: accepted
features:
- FT-149
supersedes: []
superseded-by: []
domains: []
scope: domain
---

**Status:** Accepted

**Context:**

The event model already captured commands, events, read models, and UI steps,
but had no first-class node for *what initiates a command*. A UI step carried a
deprecated free-text `triggers` alias, but automation and cross-system
communication had no representation at all — they were implicit. Framework
§3.2.0 names a single **Trigger** block whose `source` is one of user, external,
or automated, and shows that the Automation and Translation patterns are just an
automated trigger that watches a View.

**Decision:**

Model the Trigger as one captured node with a `source` enum (user / external /
automated), an `issues` edge to the command it starts, an optional `watches`
edge to a View (for the Automation/Translation patterns), and an optional
`translates_from` edge to one source System (for Translation). Validate the
pattern shapes that are safe to enforce on a declared trigger: source is one of
the three values, the issued command resolves, an automated trigger watches a
declared View, and a `translates_from` system resolves. Defer Command-pattern
*completeness* (every command has a trigger), consistent with FT-148's deferral
of flow→system completeness, to avoid invalidating pre-existing What graphs.

**Rationale:**

Generalising "what issues a command" into one block with a source dimension is
what lets a robot and a user be the *same shape*, differing only in source — so
automation and translation become checkable patterns rather than special cases.
Enforcing only the shapes that fire on an existing trigger keeps the change
backward-compatible: graphs without triggers are unaffected, and the moment a
trigger is declared its Automation/Translation shape is checked.

**Rejected alternatives:**

- **Three separate node kinds (UserTrigger / ApiTrigger / AutomatedTrigger).**
  Rejected: it would triplicate the wiring and lose the insight that they are one
  shape with a source dimension; patterns become harder to query.
- **Keep the UI step's free-text `triggers` alias.** Rejected: prose cannot be
  checked, and it cannot express automation or translation, which have no UI.
- **Enforce command→trigger completeness immediately.** Rejected: it would
  invalidate existing graphs whose commands predate the Trigger block; deferred.

**Test coverage:** TC-1032, TC-1033.
