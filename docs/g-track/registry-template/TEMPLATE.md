# Registry template

**Template version: 0.1.0** (G-1 session, 2026-08-17; versioned with product-cli — instances pin
this version and re-pin when it moves, by the same re-derive-at-a-named-commit discipline the
G-track itself runs).

This directory is a **parameterised template for any ground registry**. It is not a repository
and names no instance: everything instance-specific is a generation parameter, supplied when an
instance is generated — never filled in the template (G-1 Gate 3 ruling).

## Generation parameters

| Parameter | Placeholder in the template | Filled at generation |
|---|---|---|
| Owning organisation | `{{OWNER_ORG}}` | the accountable-principal field of every trust decision that will reference the instance |
| Repository name | `{{REPO_NAME}}` | — |
| Base IRI | `https://REGISTRY-HOST.example/ns#` (parses as a valid IRI, so the template validates as-is) | derived from a host the owner controls **durably** — IRIs outlive hosting choices, so the host must be one the owner can keep resolving for the registry's lifetime; the template documents the requirement and does not pick a host |
| Ratifier | `{{RATIFIER}}` | the named person whose merge is ratification for this instance |
| Registry display name | `{{DISPLAY_NAME}}` | — |

## Generating an instance

1. Copy the template's contents (minus this file's version header context — the file itself
   travels) to the new repository's root.
2. Substitute every parameter, including the base IRI in `shapes/*.ttl` and
   `graphs/**/*.ttl`.
3. **Record the birth provenance.** The instance's first commit fills `GENERATION.ttl`: template
   version, parameters supplied, generated-by, date — `prov:wasAttributedTo` the generation act.
   This is what lets the instance re-pin when the template moves.
4. **The founding-decision slot** (`graphs/canonical/founding-decision.ttl`) stays empty at
   generation. The owning organisation's decision to keep a registry, filed by its ratifier, is
   the **first ratified content** — the template provides the slot; only the instance can fill it.

## Instance generation is a G0-entry step

The G-track's first instance is generated at G0 entry, its parameters supplied then. No instance
exists at G-1, by ruling.
