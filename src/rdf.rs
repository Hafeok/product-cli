//! RDF/Turtle export and SPARQL query (ADR-008)

use crate::graph::KnowledgeGraph;
use crate::error::Result;
use std::path::Path;

/// Export the knowledge graph as RDF Turtle
pub fn export_turtle(graph: &KnowledgeGraph) -> String {
    let mut out = String::new();

    out.push_str("@prefix pm: <https://product-meta/ontology#> .\n");
    out.push_str("@prefix ft: <https://product-meta/feature/> .\n");
    out.push_str("@prefix adr: <https://product-meta/adr/> .\n");
    out.push_str("@prefix tc: <https://product-meta/test/> .\n");
    out.push_str("@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n");
    out.push('\n');

    let centrality = graph.betweenness_centrality();

    // Features
    for f in graph.features.values() {
        let iri = format!("ft:{}", f.front.id);
        out.push_str(&format!("{} a pm:Feature ;\n", iri));
        out.push_str(&format!(
            "    pm:title \"{}\" ;\n",
            escape_turtle(&f.front.title)
        ));
        out.push_str(&format!("    pm:phase {} ;\n", f.front.phase));
        out.push_str(&format!(
            "    pm:status pm:{:?} ",
            f.front.status
        ));

        for adr_id in &f.front.adrs {
            out.push_str(&format!(";\n    pm:implementedBy adr:{} ", adr_id));
        }
        for test_id in &f.front.tests {
            out.push_str(&format!(";\n    pm:validatedBy tc:{} ", test_id));
        }
        for dep_id in &f.front.depends_on {
            out.push_str(&format!(";\n    pm:dependsOn ft:{} ", dep_id));
        }
        out.push_str(".\n\n");
    }

    // ADRs
    for a in graph.adrs.values() {
        let iri = format!("adr:{}", a.front.id);
        out.push_str(&format!("{} a pm:ArchitecturalDecision ;\n", iri));
        out.push_str(&format!(
            "    pm:title \"{}\" ;\n",
            escape_turtle(&a.front.title)
        ));
        out.push_str(&format!(
            "    pm:status pm:{:?} ",
            a.front.status
        ));

        if let Some(c) = centrality.get(&a.front.id) {
            out.push_str(&format!(";\n    pm:betweennessCentrality {:.3} ", c));
        }

        for f_id in &a.front.features {
            out.push_str(&format!(";\n    pm:appliesTo ft:{} ", f_id));
        }
        for sup in &a.front.supersedes {
            out.push_str(&format!(";\n    pm:supersedes adr:{} ", sup));
        }
        out.push_str(".\n\n");
    }

    // Tests
    for t in graph.tests.values() {
        let iri = format!("tc:{}", t.front.id);
        out.push_str(&format!("{} a pm:TestCriterion ;\n", iri));
        out.push_str(&format!(
            "    pm:title \"{}\" ;\n",
            escape_turtle(&t.front.title)
        ));
        out.push_str(&format!(
            "    pm:type pm:{:?} ;\n",
            t.front.test_type
        ));
        out.push_str(&format!(
            "    pm:status pm:{:?} ",
            t.front.status
        ));

        for f_id in &t.front.validates.features {
            out.push_str(&format!(";\n    pm:validates ft:{} ", f_id));
        }
        for a_id in &t.front.validates.adrs {
            out.push_str(&format!(";\n    pm:validates adr:{} ", a_id));
        }
        out.push_str(".\n\n");
    }

    out
}

/// Write the TTL export to a file
pub fn write_index_ttl(graph: &KnowledgeGraph, path: &Path) -> Result<()> {
    let ttl = export_turtle(graph);
    crate::fileops::write_file_atomic(path, &ttl)
}

/// Execute a SPARQL query against the knowledge graph using Oxigraph
pub fn sparql_query(graph: &KnowledgeGraph, query: &str) -> Result<String> {
    use oxigraph::io::RdfFormat;
    use oxigraph::store::Store;
    use oxigraph::sparql::QueryResults;

    let store = Store::new().map_err(|e| {
        crate::error::ProductError::Internal(format!("failed to create Oxigraph store: {}", e))
    })?;

    // Load the TTL into the store
    let ttl = export_turtle(graph);
    store
        .load_from_reader(RdfFormat::Turtle, ttl.as_bytes())
        .map_err(|e| {
            crate::error::ProductError::Internal(format!("failed to load TTL into store: {}", e))
        })?;

    // Execute query
    let results = store.query(query).map_err(|e| {
        crate::error::ProductError::Internal(format!("SPARQL query error: {}", e))
    })?;

    match results {
        QueryResults::Solutions(solutions) => {
            let mut out = String::new();
            let vars: Vec<String> = solutions
                .variables()
                .iter()
                .map(|v| v.as_str().to_string())
                .collect();
            out.push_str(&vars.join("\t"));
            out.push('\n');
            for solution in solutions {
                let solution = solution.map_err(|e| {
                    crate::error::ProductError::Internal(format!("SPARQL solution error: {}", e))
                })?;
                let row: Vec<String> = vars
                    .iter()
                    .map(|v| {
                        solution
                            .get(v.as_str())
                            .map(|t| t.to_string())
                            .unwrap_or_default()
                    })
                    .collect();
                out.push_str(&row.join("\t"));
                out.push('\n');
            }
            Ok(out)
        }
        QueryResults::Boolean(b) => Ok(format!("{}\n", b)),
        QueryResults::Graph(_) => Ok("(graph result)\n".to_string()),
    }
}

fn escape_turtle(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;
    use std::path::PathBuf;

    #[test]
    fn turtle_export_contains_prefixes() {
        let graph = KnowledgeGraph::build(vec![], vec![], vec![]);
        let ttl = export_turtle(&graph);
        assert!(ttl.contains("@prefix pm:"));
        assert!(ttl.contains("@prefix ft:"));
    }

    #[test]
    fn sparql_query_works() {
        let feature = crate::types::Feature {
            front: FeatureFrontMatter {
                id: "FT-001".to_string(),
                title: "Test".to_string(),
                phase: 1,
                status: FeatureStatus::Planned,
                depends_on: vec![],
                adrs: vec![],
                tests: vec![],
                domains: vec![],
                domains_acknowledged: std::collections::HashMap::new(),
                bundle: None,
            },
            body: String::new(),
            path: PathBuf::from("test.md"),
        };
        let graph = KnowledgeGraph::build(vec![feature], vec![], vec![]);
        let result = sparql_query(&graph, "SELECT ?s WHERE { ?s a <https://product-meta/ontology#Feature> }");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("FT-001"));
    }
}
