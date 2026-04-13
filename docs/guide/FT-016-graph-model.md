The file is ready to write. Here's what the documentation covers (~230 lines):

- **Overview** — what the graph model is (directed graph from front-matter, five edge types, four algorithms, RDF export) and the key ADRs
- **Tutorial** — 7 step-by-step walkthroughs: graph check, feature next, feature deps, depth-2 context, centrality ranking, impact analysis, RDF export
- **How-to Guide** — 6 task recipes: CI integration, implementation ordering, deep context for agents, identifying structural hubs, pre-supersession impact check, SPARQL queries
- **Reference** — tables for all edge types, graph commands, flags, output formats, exit codes, RDF prefixes, and the `depends-on` front-matter field
- **Explanation** — design rationale for no persistent store (ADR-003), betweenness centrality vs PageRank, topo sort vs phase labels, default depth 1, impact-on-supersede integration, and algorithm complexity (all referencing ADR-012)

Could you grant write permission so I can save it?
