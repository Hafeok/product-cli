//! Turtle projection of the store into the `ddd:` namespace for rule queries.
//!
//! The projection exists solely as the surface the SPARQL ontology rules run
//! over (`pf::sparql_rules`); it emits types plus the governed edges, nothing
//! more. Entry ids keep their `/` segments by percent-encoding them into the
//! IRI fragment; [`decode_ref`] restores them when a rule row is rendered
//! back into a violation.

use crate::decision::DecisionKind;
use crate::manifest::UNGOVERNED;
use crate::store::DddStore;

/// The DDD vocabulary namespace (its own ontology — not the What/How one).
pub const NS: &str = "https://decisiondriven.design/ns#";
/// The entry-instance namespace; ids live in the fragment, percent-encoded.
pub const ENTITY: &str = "https://decisiondriven.design/x#";

/// Percent-encode an entry id for use as an IRI fragment.
pub fn encode_id(id: &str) -> String {
    let mut out = String::with_capacity(id.len());
    for ch in id.chars() {
        match ch {
            '%' => out.push_str("%25"),
            '/' => out.push_str("%2F"),
            '#' => out.push_str("%23"),
            ' ' => out.push_str("%20"),
            _ => out.push(ch),
        }
    }
    out
}

/// Undo [`encode_id`] on a rule-row binding (a fragment local name).
pub fn decode_ref(s: &str) -> String {
    s.replace("%2F", "/").replace("%23", "#").replace("%20", " ").replace("%25", "%")
}

fn iri(id: &str) -> String {
    format!("<{ENTITY}{}>", encode_id(id))
}

fn prop(local: &str) -> String {
    format!("<{NS}{local}>")
}

fn escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n")
}

fn triple(out: &mut String, subject: &str, predicate: &str, object: &str) {
    out.push_str(subject);
    out.push(' ');
    out.push_str(predicate);
    out.push(' ');
    out.push_str(object);
    out.push_str(" .\n");
}

fn typed(out: &mut String, id: &str, class: &str) -> String {
    let subject = iri(id);
    triple(out, &subject, "a", &prop(class));
    subject
}

/// Project the whole store as Turtle (full IRIs, no prefix block).
pub fn to_turtle(store: &DddStore) -> String {
    let mut out = String::new();
    for p in &store.predicates {
        let s = typed(&mut out, &p.id, "Predicate");
        for dep in &p.depends_on {
            triple(&mut out, &s, &prop("dependsOn"), &iri(dep));
        }
    }
    for c in &store.claims {
        let s = typed(&mut out, &c.id, "Claim");
        for dep in &c.depends_on {
            triple(&mut out, &s, &prop("dependsOn"), &iri(dep));
        }
        for r in &c.refines {
            triple(&mut out, &s, &prop("refines"), &iri(r));
        }
    }
    for d in &store.decisions {
        let class = match d.kind {
            DecisionKind::Decision => "Decision",
            DecisionKind::RiskAcceptance => "RiskAcceptance",
        };
        let s = typed(&mut out, &d.id, class);
        for basis in &d.based_on {
            triple(&mut out, &s, &prop("basedOn"), &iri(basis.claim_id()));
        }
        if !d.principal.trim().is_empty() {
            triple(&mut out, &s, &prop("principal"), &format!("\"{}\"", escape(&d.principal)));
        }
    }
    manifest_triples(store, &mut out);
    for p in &store.patterns {
        let s = typed(&mut out, &p.id, "PatternInstance");
        if !p.pattern_predicate.trim().is_empty() {
            triple(&mut out, &s, &prop("instantiates"), &iri(&p.pattern_predicate));
        }
    }
    for s in &store.seams {
        typed(&mut out, &s.id, "Seam");
    }
    out
}

fn manifest_triples(store: &DddStore, out: &mut String) {
    for set in &store.manifests {
        for rule in &set.file.rules {
            let s = typed(out, &set.entry_id(rule), "ManifestEntry");
            if let Some(dec) = &rule.decision {
                triple(out, &s, &prop("governedBy"), &iri(dec));
            }
            if rule.governance.as_deref() == Some(UNGOVERNED) {
                triple(out, &s, &prop("ungoverned"), "true");
            }
            if let Some(sup) = &rule.suppression {
                triple(out, &s, &prop("suppressed"), "true");
                if let Some(ra) = &sup.risk_acceptance {
                    triple(out, &s, &prop("cites"), &iri(ra));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_survive_an_encode_decode_round_trip() {
        for id in ["pred/data/shape", "DDD-cat-01", "a b#c%d/e"] {
            assert_eq!(decode_ref(&encode_id(id)), id);
        }
    }

    #[test]
    fn projection_emits_types_plus_edges() {
        let mut store = DddStore::default();
        store.predicates.push(
            serde_yaml::from_str::<crate::predicate::PredicateFile>(
                "predicate:\n  id: pred/a/x\n  format: 1\n  name: X\n  statement: s\n  artifact_class: code\n  ground: []\n  tolerance: t\n  rejection_witness: w\n  depends_on: [pred/a/y]\n",
            )
            .expect("parse")
            .predicate,
        );
        let ttl = to_turtle(&store);
        assert!(ttl.contains("pred%2Fa%2Fx"), "{ttl}");
        assert!(ttl.contains("dependsOn"), "{ttl}");
    }
}
