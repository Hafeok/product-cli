# Staging note — not part of the registry

This directory is the **registry repository skeleton**, staged inside product-cli for review at
G-1 Gate 3. On Emil's Gate 3 ruling (which names the host org and the repository name), this
directory's contents — minus this file — transplant verbatim to the new repository's root, and
this copy is marked superseded.

Two placeholders resolve at the gate:

1. **The org and repository name** — which also answers the PRD's open item 7: the owning
   organisation is the accountable-principal field of every trust decision that references this
   registry.
2. **The base IRI** — `https://REGISTRY-HOST.example/ns#` throughout is a placeholder; it is set
   from the org naming, and the A1 assumption (PRD §3: the registry vocabulary extends
   product-cli's existing RDF vocabulary) fixes the prefix strategy.

One ruling is presented at the gate rather than assumed:

3. **Per-triple file format — Turtle or YAML.** Both exemplar forms sit in
   `graphs/canonical/` (`_exemplar.ttl`, `_exemplar.yaml`). The CI workflow as staged validates
   the Turtle form; if Emil rules YAML (the per-claim file pattern's literal shape), a
   YAML→RDF conversion step lands in CI at G0 and the Turtle exemplar is dropped.
