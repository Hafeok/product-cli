# meta/sessions/

Session records for `product-cli`. One directory per session, `YYYY-MM-DD-<slug>/`, holding the
session's **prompt** and **bootstrap** as committed *before* the session ran, its arrived inputs with
their sha256, and its gate outputs.

The convention is `CG-rule-06` in `Hafeok/canon-governance`: a session's prompt and bootstrap are
committed before the session begins work, ahead of any canon edit, so that what a session was asked
to do can be diffed against what it did. The rule is held there and not restated here.

Inventories, measurements and other re-derivable artefacts are **not** session records and are not
committed under this directory.

## Index

| Session | Type |
|---|---|
| `2026-09-13-implement-csharp-binding/` | Implementation under five gates — the C# stack binding beneath the domain state change binding |
