//! Canonical ordering of a parsed [`DomainGraph`].
//!
//! RDF is an unordered set of triples, so a graph reconstructed from Turtle
//! carries no insertion order. To keep `to_turtle` output stable across a
//! load → save cycle (and so the committed `.ttl` does not churn), every node
//! list and every multi-valued field is sorted into a deterministic order
//! after parsing. List order is not load-bearing in the What model (the
//! timeline view derives column order from command/event causality, not from
//! flow-step order).

use super::model::*;

/// Sort every node list and inner list of `g` into a stable, content-derived
/// order. Idempotent.
pub(super) fn canonicalize(g: &mut DomainGraph) {
    canon_structure(g);
    canon_behaviour(g);
    canon_ui(g);
    canon_data(g);
    canon_boundary(g);
}

/// §3.0–§3.6 product boundary — products, journeys, quality demands. Each owns
/// or composes other nodes by id; those id lists come back from SPARQL
/// unordered, so they are sorted here like every other multi-valued field.
fn canon_boundary(g: &mut DomainGraph) {
    g.products.sort_by(|a, b| a.id.cmp(&b.id));
    g.products.iter_mut().for_each(|p| {
        p.owns_domain.sort();
        p.owns_system.sort();
    });
    g.journeys.sort_by(|a, b| a.id.cmp(&b.id));
    g.journeys.iter_mut().for_each(|j| {
        j.composes_flow.sort();
        j.crosses_via.sort();
    });
    g.quality_demands.sort_by(|a, b| a.id.cmp(&b.id));
}

/// §3.1 structure — contexts, entities, value objects, relations, invariants,
/// mappings (whose unordered `mapsTo` pair is sorted).
fn canon_structure(g: &mut DomainGraph) {
    g.contexts.sort_by(|a, b| a.id.cmp(&b.id));
    g.contexts.iter_mut().for_each(|c| c.glossary.sort());
    g.entities.sort_by(|a, b| a.id.cmp(&b.id));
    g.entities.iter_mut().for_each(|e| e.attributes.sort_by(|a, b| a.name.cmp(&b.name)));
    g.commands.iter_mut().for_each(|c| c.fields.sort_by(|a, b| a.name.cmp(&b.name)));
    g.events.iter_mut().for_each(|e| e.fields.sort_by(|a, b| a.name.cmp(&b.name)));
    g.value_objects.sort_by(|a, b| a.id.cmp(&b.id));
    g.relations.sort_by(|a, b| a.id.cmp(&b.id));
    g.invariants.sort_by(|a, b| a.id.cmp(&b.id));
    g.context_mappings.sort_by(|a, b| a.id.cmp(&b.id));
    g.context_mappings.iter_mut().for_each(|m| {
        if m.concept_a > m.concept_b {
            std::mem::swap(&mut m.concept_a, &mut m.concept_b);
        }
    });
}

/// §3.2 behaviour — commands, events, read models, flows, systems, triggers.
fn canon_behaviour(g: &mut DomainGraph) {
    g.commands.sort_by(|a, b| a.id.cmp(&b.id));
    g.commands.iter_mut().for_each(|c| c.emits.sort());
    g.events.sort_by(|a, b| a.id.cmp(&b.id));
    g.read_models.sort_by(|a, b| a.id.cmp(&b.id));
    g.read_models.iter_mut().for_each(|r| { r.projects.sort(); r.states.sort(); });
    g.flows.sort_by(|a, b| a.id.cmp(&b.id));
    g.flows.iter_mut().for_each(|f| f.steps.sort());
    g.systems.sort_by(|a, b| a.id.cmp(&b.id));
    g.systems.iter_mut().for_each(|s| { s.target_platforms.sort(); s.target_classes.sort(); });
    g.triggers.sort_by(|a, b| a.id.cmp(&b.id));
}

/// §3.2.1–§4.5 UI layer — steps, AIOs, page graph, accessibility, content,
/// the design system, reification.
fn canon_ui(g: &mut DomainGraph) {
    g.wireframe_steps.sort_by(|a, b| a.id.cmp(&b.id));
    g.wireframe_steps.iter_mut().for_each(canon_step);
    g.aios.sort_by(|a, b| a.id.cmp(&b.id));
    g.aios.iter_mut().for_each(|a| a.must_satisfy.sort());
    g.contexts_of_use.sort_by(|a, b| a.id.cmp(&b.id));
    g.application_roots.sort_by(|a, b| a.id.cmp(&b.id));
    g.application_roots.iter_mut().for_each(|r| r.navigates_from_root.sort());
    g.wcag_criteria.sort_by(|a, b| a.id.cmp(&b.id));
    g.attestations.sort_by(|a, b| a.id.cmp(&b.id));
    g.content_stores.sort_by(|a, b| a.id.cmp(&b.id));
    g.content_stores.iter_mut().for_each(|s| {
        s.locales.sort();
        s.resolutions.sort_by(|a, b| (&a.key, &a.locale).cmp(&(&b.key, &b.locale)));
    });
    g.design_systems.sort_by(|a, b| a.id.cmp(&b.id));
    g.design_systems.iter_mut().for_each(|d| { d.cios.sort(); d.tokens.sort(); });
    g.cios.sort_by(|a, b| a.id.cmp(&b.id));
    g.tokens.sort_by(|a, b| a.id.cmp(&b.id));
    g.reification_rules.sort_by(|a, b| a.id.cmp(&b.id));
    g.unreifiable_rules.sort_by(|a, b| a.id.cmp(&b.id));
}

/// §3.1 data side — reference sets, shapes, production datasets.
fn canon_data(g: &mut DomainGraph) {
    g.reference_sets.sort_by(|a, b| a.id.cmp(&b.id));
    g.reference_sets.iter_mut().for_each(|r| r.values.sort());
    g.data_shapes.sort_by(|a, b| a.id.cmp(&b.id));
    g.data_shapes.iter_mut().for_each(|s| {
        s.required.sort();
        s.enums.sort_by(|a, b| a.field.cmp(&b.field));
        s.types.sort_by(|a, b| a.field.cmp(&b.field));
    });
    g.production_datasets.sort_by(|a, b| a.id.cmp(&b.id));
}

fn canon_step(w: &mut WireframeStep) {
    w.surfaces.sort_by(|a, b| (&a.projection, &a.aio).cmp(&(&b.projection, &b.aio)));
    w.offers.sort_by(|a, b| (&a.command, &a.aio).cmp(&(&b.command, &b.aio)));
    w.transitions_to.sort();
    w.state_meanings.sort_by(|a, b| (&a.projection, &a.state).cmp(&(&b.projection, &b.state)));
    w.must_satisfy.sort();
    w.content_refs.sort_by(|a, b| (&a.key, &a.role).cmp(&(&b.key, &b.role)));
    w.styles.sort();
}
