//! Turtle projection of an assembled blueprint — its How plus layout rules.
//!
//! Extends the How projection (`how_turtle`) with the §4.3 layout rules so the
//! SPARQL conformance rules (`sparql_rules`) can resolve cross-references that
//! span the two parts — chiefly each layout rule's `enforces` edge against the
//! principles and decisions the How defines. Reuses the same `d:` namespace as
//! `how_to_turtle` so the references line up.

use super::how::HowContract;
use super::how_turtle::{how_to_turtle, slug};
use super::layout::LayoutModel;

/// Project a How contract plus its layout model into one Turtle document. The
/// layout triples are appended after `how_to_turtle`'s prefixes + body, so they
/// share the already-declared `pf:`/`d:` prefixes.
pub fn blueprint_to_turtle(how: &HowContract, layout: Option<&LayoutModel>) -> String {
    let mut out = how_to_turtle(how);
    if let Some(model) = layout {
        for rule in &model.layout {
            let rid = slug(&rule.id);
            out.push_str(&format!("d:{rid} a pf:LayoutRule"));
            for target in &rule.enforces {
                out.push_str(&format!(" ;\n  pf:enforces d:{}", slug(target)));
            }
            out.push_str(" .\n\n");
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const HOW: &str = include_str!("../../../schema/examples/how-contract.example.yaml");
    const LAYOUT: &str = include_str!("../../../schema/examples/layout-model.example.yaml");

    #[test]
    fn projects_layout_rules_with_enforces_edges() {
        let how = HowContract::from_yaml(HOW).expect("how");
        let layout = LayoutModel::from_yaml(LAYOUT).expect("layout");
        let ttl = blueprint_to_turtle(&how, Some(&layout));
        assert!(ttl.contains("a pf:LayoutRule"));
        assert!(ttl.contains("pf:enforces d:feature-cohesion"));
        // The How body is still present (combined projection).
        assert!(ttl.contains("a pf:Principle"));
    }

    #[test]
    fn no_layout_is_just_the_how() {
        let how = HowContract::from_yaml(HOW).expect("how");
        assert_eq!(blueprint_to_turtle(&how, None), how_to_turtle(&how));
    }
}
