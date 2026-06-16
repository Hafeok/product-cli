//! Turtle projection of a How contract into the framework graph.
//!
//! Emits the §4 nodes (decisions, principles, patterns, the two contracts,
//! interfaces) plus the synthesised Verification nodes (from `enforced_by`)
//! and Work Unit nodes (from a pattern's `applied_by`) the cross-node shapes
//! need — so the output validates against `how.shacl.ttl` (incl. the crown
//! trace-truth rule) and `shapes.shacl.ttl` together.

use std::collections::{BTreeMap, BTreeSet};

use super::how::HowContract;

const PF: &str = "https://productframework.org/ns#";

/// Project a How contract to Turtle under a per-archetype namespace.
pub fn how_to_turtle(c: &HowContract) -> String {
    let mut out = String::new();
    prefixes(&mut out, &c.archetype);
    for d in &c.top_decisions {
        out.push_str(&format!("d:{} a pf:TopDecision ;\n  pf:rationale {}", d.id, lit(&d.rationale)));
        // Emit every licenses edge as authored — dangling ones (target is no
        // Principle) are caught by the `licenses-resolves` graph rule.
        for l in &d.licenses {
            out.push_str(&format!(" ;\n  pf:licenses d:{}", slug(l)));
        }
        out.push_str(" .\n\n");
    }
    for p in &c.principles {
        out.push_str(&format!("d:{} a pf:Principle ;\n  pf:statement {}", p.id, lit(&p.statement)));
        for r in &p.realized_by {
            out.push_str(&format!(" ;\n  pf:realizedBy d:{}", slug(r)));
        }
        out.push_str(" .\n\n");
    }
    for p in &c.patterns {
        out.push_str(&format!("d:{} a pf:Pattern", p.id));
        for r in &p.realizes {
            out.push_str(&format!(" ;\n  pf:realizes d:{}", r));
        }
        out.push_str(" .\n\n");
    }
    emit_contracts(&mut out, c);
    for i in &c.interface_contracts {
        out.push_str(&format!("d:{} a pf:InterfaceContract ;\n  rdfs:label {}", i.id, lit(&i.surface)));
        for from in &i.derived_from {
            out.push_str(&format!(" ;\n  pf:derivedFrom d:{}", slug(from)));
        }
        out.push_str(" .\n\n");
    }
    emit_verifications(&mut out, c);
    emit_work_units(&mut out, c);
    out
}

fn prefixes(out: &mut String, archetype: &str) {
    out.push_str(&format!("@prefix pf: <{}> .\n", PF));
    out.push_str(&format!("@prefix d: <https://productframework.org/archetype/{}#> .\n", slug(archetype)));
    out.push_str("@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n\n");
}

fn emit_contracts(out: &mut String, c: &HowContract) {
    let a = &c.application_contract;
    out.push_str(&format!("d:{} a pf:ApplicationContract ;\n  rdfs:label {} .\n\n", a.id, lit(&a.language)));
    if let Some(infra) = &c.infrastructure_contract {
        out.push_str(&format!("d:{} a pf:InfrastructureContract ;\n  pf:conformsTo d:{} .\n\n", infra.id, infra.satisfies));
    }
}

/// One Verification node per distinct `enforced_by` id, carrying every
/// `enforces` edge it backs plus a kind (so VerificationShape is satisfied).
fn emit_verifications(out: &mut String, c: &HowContract) {
    let mut enforces: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut record = |vid: &str, target: &str| {
        enforces.entry(vid.to_string()).or_default().insert(target.to_string());
    };
    for d in &c.top_decisions {
        for v in &d.enforced_by { record(v, &d.id); }
    }
    for p in &c.principles {
        for v in &p.enforced_by { record(v, &p.id); }
    }
    for p in &c.patterns {
        for v in &p.enforced_by { record(v, &p.id); }
    }
    for (vid, targets) in enforces {
        out.push_str(&format!("d:{} a pf:Verification ;\n  pf:verificationKind pf:domainConformance", vid));
        for t in targets {
            out.push_str(&format!(" ;\n  pf:enforces d:{}", t));
        }
        out.push_str(" .\n\n");
    }
}

/// One Work Unit node per distinct `applied_by` id; it applies the principles
/// realised by the patterns that name it, and is derived from the app contract.
fn emit_work_units(out: &mut String, c: &HowContract) {
    let mut applies: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for pat in &c.patterns {
        for wid in &pat.applied_by {
            let entry = applies.entry(wid.to_string()).or_default();
            for pid in &pat.realizes {
                entry.insert(pid.clone());
            }
        }
    }
    for (wid, principles) in applies {
        out.push_str(&format!("d:{} a pf:WorkUnit ;\n  pf:derivedFrom d:{}", wid, c.application_contract.id));
        for pid in principles {
            out.push_str(&format!(" ;\n  pf:applies d:{}", pid));
        }
        out.push_str(" .\n\n");
    }
}

/// Sanitize a reference (e.g. `domain:Task`) into a Turtle local name.
pub(super) fn slug(s: &str) -> String {
    let mut out: String = s.chars().map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '_' }).collect();
    if !out.chars().next().map(|c| c.is_ascii_alphabetic()).unwrap_or(false) {
        out.insert(0, 'x');
    }
    out
}

fn lit(s: &str) -> String {
    let escaped = s.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n");
    format!("\"{}\"", escaped)
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE: &str = include_str!("../../../schema/examples/how-contract.example.yaml");

    #[test]
    fn projects_nodes_and_synthesised_links() {
        let c = HowContract::from_yaml(EXAMPLE).expect("parse");
        let ttl = how_to_turtle(&c);
        assert!(ttl.contains("a pf:TopDecision"));
        assert!(ttl.contains("a pf:Principle"));
        assert!(ttl.contains("a pf:Pattern"));
        assert!(ttl.contains("pf:conformsTo"));
        // synthesised verification + work unit for the trace
        assert!(ttl.contains("a pf:Verification"));
        assert!(ttl.contains("pf:enforces"));
        assert!(ttl.contains("a pf:WorkUnit"));
        assert!(ttl.contains("pf:applies"));
    }
}
