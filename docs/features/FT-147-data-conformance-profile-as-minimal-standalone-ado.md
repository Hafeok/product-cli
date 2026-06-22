---
id: FT-147
title: Data conformance profile as minimal standalone adoption
phase: 7
status: complete
depends-on:
- FT-145
adrs:
- ADR-089
tests:
- TC-1029
domains: []
domains-acknowledged: {}
---

## Description

§13 of the framework — **🅿 PREVIEW**, non-normative — names the framework's
**minimal standalone adoption**: data conformance used on its own, by a system
that uses none of the rest of the framework. It introduces **no new
requirements** — it is §3.1 (structure and data) and §6.3 (data conformance)
packaged as an entry point (ADR-089). This feature does not add a mechanism; it
asserts that the mechanism FT-145 already provides is adoptable *alone*, and
documents it as the recommended doorway.

## Functional Specification

### What a conforming standalone adoption provides (§13.2)

1. **A declared structure** — entity types, relations, cardinalities, and
   invariants expressed as validatable shapes (data-shapes over entities), and
   nothing more.
2. **A bound dataset** — a declared production dataset the shapes are evaluated
   against (the oracle, §3.1).
3. **Continuous assertion** — the shapes are evaluable on demand and on a
   schedule, so conformance is a standing signal (`product domain data`).
4. **Bidirectional triage** — a posture for reading a failure as data defect or
   spec drift (§3.1); the tool reports the divergence, not a verdict that
   presumes the data is at fault.

### Behaviour

- A graph carrying **only** a domain structure (a context, an entity, a
  reference set, a data-shape) and a **production dataset** — with no event
  model, Decider, Projector, UI model, or work units — validates and runs data
  conformance end to end. This is the proof that the data side is adoptable with
  none of the rest of the framework present.
- The signal surfaced is the **data-divergence rate** (§13.3, FT-145), not just
  pass/fail — the trend is the early warning a single verdict hides.

## Out of scope

- The data-conformance **mechanism** itself (data-shapes, production datasets,
  the divergence rate, validation) is FT-145/ADR-089; this feature is its
  packaging as an entry point and introduces no new mechanism.
- Graduating §13 to normative status, which awaits a reference adoption running
  data conformance continuously against a real production system (Why-Preview).

## Acceptance

- TC-1029 — a graph with only structure (context + entity + reference set +
  data-shape) and a production dataset — no event model, Decider, Projector, or
  UI — passes `product domain validate` and `product domain data`, reporting the
  data-divergence rate. Data conformance is adoptable standalone.
