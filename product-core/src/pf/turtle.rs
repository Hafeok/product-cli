//! Turtle serialization of the What graph.
//!
//! Emits the typed model as RDF that validates against
//! `schema/shapes/shapes.shacl.ttl`. Node classes use the `pf:` ontology
//! namespace; instances live under a per-product namespace `d:`.

use super::model::DomainGraph;

const PF: &str = "https://productframework.org/ns#";

/// Serialize a domain graph to Turtle. `product` keys the instance namespace.
pub fn to_turtle(graph: &DomainGraph, product: &str) -> String {
    let mut out = String::new();
    prefixes(&mut out, product);
    graph.contexts.iter().for_each(|c| emit_context(&mut out, c));
    graph.entities.iter().for_each(|e| emit_entity(&mut out, e));
    graph.value_objects.iter().for_each(|vo| emit_value_object(&mut out, vo));
    graph.relations.iter().for_each(|r| emit_relation(&mut out, r));
    graph.invariants.iter().for_each(|i| emit_invariant(&mut out, i));
    graph.context_mappings.iter().for_each(|m| emit_mapping(&mut out, m));
    graph.commands.iter().for_each(|c| emit_command(&mut out, c));
    graph.events.iter().for_each(|ev| emit_event(&mut out, ev));
    graph.read_models.iter().for_each(|rm| emit_read_model(&mut out, rm));
    graph.wireframe_steps.iter().for_each(|w| emit_wireframe(&mut out, w));
    graph.flows.iter().for_each(|f| emit_flow(&mut out, f));
    graph.aios.iter().for_each(|a| emit_aio(&mut out, a));
    graph.contexts_of_use.iter().for_each(|c| emit_context_of_use(&mut out, c));
    graph.application_roots.iter().for_each(|r| emit_application_root(&mut out, r));
    graph.wcag_criteria.iter().for_each(|c| emit_wcag(&mut out, c));
    graph.attestations.iter().for_each(|a| emit_attestation(&mut out, a));
    graph.content_stores.iter().for_each(|s| emit_content_store(&mut out, s));
    graph.design_systems.iter().for_each(|d| emit_design_system(&mut out, d));
    graph.reification_rules.iter().for_each(|r| emit_reification_rule(&mut out, r));
    graph.reference_sets.iter().for_each(|rs| emit_reference_set(&mut out, rs));
    graph.data_shapes.iter().for_each(|s| emit_data_shape(&mut out, s));
    graph.production_datasets.iter().for_each(|d| emit_dataset(&mut out, d));
    for c in &graph.cios {
        out.push_str(&format!("d:{} a pf:Cio", c.id));
        if let Some(l) = &c.label { out.push_str(&format!(" ;\n  rdfs:label {}", lit(l))); }
        out.push_str(" .\n\n");
    }
    for t in &graph.tokens {
        out.push_str(&format!("d:{} a pf:Token", t.id));
        if let Some(k) = &t.kind { out.push_str(&format!(" ;\n  pf:tokenKind {}", lit(k))); }
        out.push_str(" .\n\n");
    }
    out
}

fn emit_context(out: &mut String, c: &super::model::BoundedContext) {
    out.push_str(&format!("d:{} a pf:BoundedContext ;\n  rdfs:label {}", c.id, lit(&c.label)));
    if let Some(p) = &c.purpose {
        out.push_str(&format!(" ;\n  pf:purpose {}", lit(p)));
    }
    for t in &c.glossary {
        out.push_str(&format!(" ;\n  pf:ubiquitousTerm {}", lit(t)));
    }
    out.push_str(" .\n\n");
}

fn emit_value_object(out: &mut String, vo: &super::model::ValueObject) {
    out.push_str(&format!("d:{} a pf:ValueObject ;\n  rdfs:label {} ;\n  pf:inContext d:{}", vo.id, lit(&vo.label), vo.context));
    if let Some(d) = &vo.definition {
        out.push_str(&format!(" ;\n  pf:definition {}", lit(d)));
    }
    out.push_str(" .\n\n");
}

fn emit_invariant(out: &mut String, i: &super::model::Invariant) {
    out.push_str(&format!("d:{} a pf:Invariant ;\n  pf:statement {}", i.id, lit(&i.statement)));
    if let Some(c) = &i.context {
        out.push_str(&format!(" ;\n  pf:inContext d:{}", c));
    }
    if let Some(a) = &i.applies_to {
        out.push_str(&format!(" ;\n  pf:appliesTo d:{}", a));
    }
    out.push_str(" .\n\n");
}

fn emit_mapping(out: &mut String, m: &super::model::ContextMapping) {
    out.push_str(&format!("d:{} a pf:ContextMapping ;\n  pf:mapsTo d:{} ;\n  pf:mapsTo d:{}", m.id, m.concept_a, m.concept_b));
    if let Some(k) = &m.kind {
        out.push_str(&format!(" ;\n  pf:mappingKind {}", lit(k)));
    }
    out.push_str(&format!(" ;\n  pf:rationale {} .\n\n", lit(&m.rationale)));
}

fn emit_event(out: &mut String, ev: &super::model::Event) {
    out.push_str(&format!("d:{} a pf:Event ;\n  rdfs:label {} ;\n  pf:inContext d:{} ;\n  pf:changes d:{} .\n\n",
        ev.id, lit(&ev.label), ev.context, ev.changes));
}

fn emit_read_model(out: &mut String, rm: &super::model::ReadModel) {
    out.push_str(&format!("d:{} a pf:ReadModel ;\n  rdfs:label {}", rm.id, lit(&rm.label)));
    for p in &rm.projects {
        out.push_str(&format!(" ;\n  pf:projects d:{}", p));
    }
    for s in &rm.states {
        out.push_str(&format!(" ;\n  pf:hasState {}", lit(s)));
    }
    out.push_str(" .\n\n");
}

fn emit_flow(out: &mut String, f: &super::model::Flow) {
    out.push_str(&format!("d:{} a pf:Flow ;\n  rdfs:label {}", f.id, lit(&f.label)));
    for s in &f.steps {
        out.push_str(&format!(" ;\n  pf:contains d:{}", s));
    }
    if let Some(e) = &f.entry_page {
        out.push_str(&format!(" ;\n  pf:entryPage d:{}", e));
    }
    out.push_str(" .\n\n");
}

fn emit_design_system(out: &mut String, d: &super::model::DesignSystem) {
    out.push_str(&format!("d:{} a pf:DesignSystem", d.id));
    if let Some(l) = &d.label { out.push_str(&format!(" ;\n  rdfs:label {}", lit(l))); }
    for c in &d.cios { out.push_str(&format!(" ;\n  pf:hasCio d:{}", c)); }
    for t in &d.tokens { out.push_str(&format!(" ;\n  pf:hasToken {}", lit(t))); }
    out.push_str(" .\n\n");
}

fn emit_reification_rule(out: &mut String, r: &super::model::ReificationRule) {
    out.push_str(&format!(
        "d:{} a pf:ReificationRule ;\n  pf:reifies d:{} ;\n  pf:inContext d:{} ;\n  pf:toCio d:{}",
        r.id, r.aio, r.context, r.cio
    ));
    if let Some(why) = &r.rationale { out.push_str(&format!(" ;\n  pf:rationale {}", lit(why))); }
    out.push_str(" .\n\n");
}

fn emit_content_store(out: &mut String, s: &super::model::ContentStore) {
    out.push_str(&format!("d:{} a pf:ContentStore", s.id));
    if let Some(l) = &s.label { out.push_str(&format!(" ;\n  rdfs:label {}", lit(l))); }
    for loc in &s.locales {
        out.push_str(&format!(" ;\n  pf:locale {}", lit(loc)));
    }
    for r in &s.resolutions {
        out.push_str(&format!(
            " ;\n  pf:resolves [ pf:contentKey {} ; pf:inLocale {} ; pf:value {} ]",
            lit(&r.key), lit(&r.locale), lit(&r.value)
        ));
    }
    out.push_str(" .\n\n");
}

fn emit_wcag(out: &mut String, c: &super::model::WcagCriterion) {
    out.push_str(&format!("d:{} a pf:WcagCriterion", c.id));
    if let Some(l) = &c.label { out.push_str(&format!(" ;\n  rdfs:label {}", lit(l))); }
    if let Some(l) = &c.level { out.push_str(&format!(" ;\n  pf:level {}", lit(l))); }
    if let Some(v) = &c.verification { out.push_str(&format!(" ;\n  pf:verification {}", lit(v))); }
    out.push_str(&format!(" ;\n  pf:satisfied {} .\n\n", c.satisfied));
}

fn emit_attestation(out: &mut String, a: &super::model::Attestation) {
    out.push_str(&format!(
        "d:{} a pf:Attestation ;\n  pf:attestsStep d:{} ;\n  pf:attestsCriterion d:{} ;\n  pf:date {} ;\n  pf:attestedBy {} .\n\n",
        a.id, a.step, a.criterion, lit(&a.date), lit(&a.by)
    ));
}

fn emit_application_root(out: &mut String, r: &super::model::ApplicationRoot) {
    out.push_str(&format!("d:{} a pf:ApplicationRoot", r.id));
    if let Some(l) = &r.label {
        out.push_str(&format!(" ;\n  rdfs:label {}", lit(l)));
    }
    for d in &r.navigates_from_root {
        out.push_str(&format!(" ;\n  pf:navigatesFromRoot d:{}", d));
    }
    out.push_str(" .\n\n");
}

fn emit_aio(out: &mut String, a: &super::model::Aio) {
    out.push_str(&format!("d:{} a pf:Aio ;\n  rdfs:label {}", a.id, lit(&a.label)));
    if let Some(m) = &a.means {
        out.push_str(&format!(" ;\n  pf:means {}", lit(m)));
    }
    for c in &a.must_satisfy {
        out.push_str(&format!(" ;\n  pf:mustSatisfy d:{}", c));
    }
    out.push_str(" .\n\n");
}

fn emit_context_of_use(out: &mut String, c: &super::model::ContextOfUse) {
    out.push_str(&format!("d:{} a pf:ContextOfUse ;\n  rdfs:label {}", c.id, lit(&c.label)));
    if let Some(d) = &c.dimension {
        out.push_str(&format!(" ;\n  pf:dimension {}", lit(d)));
    }
    if let Some(v) = &c.value {
        out.push_str(&format!(" ;\n  pf:contextValue {}", lit(v)));
    }
    out.push_str(" .\n\n");
}

fn prefixes(out: &mut String, product: &str) {
    out.push_str(&format!("@prefix pf: <{}> .\n", PF));
    out.push_str(&format!("@prefix d: <https://productframework.org/product/{}#> .\n", product));
    out.push_str("@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .\n");
    out.push_str("@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n\n");
}

fn emit_entity(out: &mut String, e: &super::model::Entity) {
    out.push_str(&format!("d:{} a pf:Entity ;\n  rdfs:label {} ;\n  pf:definition {} ;\n  pf:inContext d:{}",
        e.id, lit(&e.label), lit(&e.definition), e.context));
    if e.is_aggregate_root {
        out.push_str(" ;\n  pf:isAggregateRoot \"true\"");
    }
    if let Some(id) = &e.identity {
        out.push_str(&format!(" ;\n  pf:identity {}", lit(id)));
    }
    out.push_str(" .\n\n");
}

fn emit_relation(out: &mut String, r: &super::model::Relation) {
    out.push_str(&format!("d:{} a pf:Relation", r.id));
    if let Some(l) = &r.label {
        out.push_str(&format!(" ;\n  rdfs:label {}", lit(l)));
    }
    out.push_str(&format!(" ;\n  pf:from d:{} ;\n  pf:to d:{} ;\n  pf:cardinality {} ;\n  pf:rationale {} .\n\n",
        r.from, r.to, lit(&r.cardinality), lit(&r.rationale)));
}

fn emit_command(out: &mut String, cmd: &super::model::Command) {
    out.push_str(&format!("d:{} a pf:Command ;\n  rdfs:label {} ;\n  pf:inContext d:{} ;\n  pf:targets d:{}",
        cmd.id, lit(&cmd.label), cmd.context, cmd.targets));
    for ev in &cmd.emits {
        out.push_str(&format!(" ;\n  pf:emits d:{}", ev));
    }
    out.push_str(" .\n\n");
}

fn emit_wireframe(out: &mut String, w: &super::model::WireframeStep) {
    out.push_str(&format!("d:{} a pf:WireframeStep ;\n  rdfs:label {}", w.id, lit(&w.label)));
    if let Some(i) = &w.intent {
        out.push_str(&format!(" ;\n  pf:intent {}", lit(i)));
    }
    for s in &w.surfaces {
        out.push_str(&format!(" ;\n  pf:surfaces d:{} ;\n  pf:typedAs d:{}", s.projection, s.aio));
    }
    for o in &w.offers {
        out.push_str(&format!(" ;\n  pf:offers d:{} ;\n  pf:typedAs d:{}", o.command, o.aio));
    }
    for t in &w.transitions_to {
        out.push_str(&format!(" ;\n  pf:transitionsTo d:{}", t));
    }
    for c in &w.must_satisfy {
        out.push_str(&format!(" ;\n  pf:mustSatisfy d:{}", c));
    }
    for cr in &w.content_refs {
        out.push_str(&format!(" ;\n  pf:referencesContent [ pf:contentKey {} ; pf:role {} ]", lit(&cr.key), lit(&cr.role)));
    }
    if let Some(t) = &w.triggers {
        out.push_str(&format!(" ;\n  pf:triggers d:{}", t));
    }
    if let Some(d) = &w.displays {
        out.push_str(&format!(" ;\n  pf:displays d:{}", d));
    }
    out.push_str(" .\n\n");
}

fn emit_reference_set(out: &mut String, rs: &super::model::ReferenceSet) {
    out.push_str(&format!("d:{} a pf:ReferenceSet ;\n  pf:referenceDataFor d:{}", rs.id, rs.concept));
    if let Some(l) = &rs.label {
        out.push_str(&format!(" ;\n  rdfs:label {}", lit(l)));
    }
    for v in &rs.values {
        out.push_str(&format!(" ;\n  pf:referenceValue {}", lit(v)));
    }
    out.push_str(" .\n\n");
}

fn emit_data_shape(out: &mut String, s: &super::model::DataShape) {
    out.push_str(&format!("d:{} a pf:DataShape ;\n  pf:shapeTarget d:{}", s.id, s.target));
    if let Some(l) = &s.label {
        out.push_str(&format!(" ;\n  rdfs:label {}", lit(l)));
    }
    for r in &s.required {
        out.push_str(&format!(" ;\n  pf:requiredField {}", lit(r)));
    }
    for c in &s.enums {
        out.push_str(&format!(
            " ;\n  pf:enumConstraint [ pf:field {} ; pf:fromReferenceSet d:{} ]",
            lit(&c.field), c.reference_set
        ));
    }
    for c in &s.types {
        out.push_str(&format!(
            " ;\n  pf:typeConstraint [ pf:field {} ; pf:datatype {} ]",
            lit(&c.field), lit(&c.datatype)
        ));
    }
    out.push_str(" .\n\n");
}

fn emit_dataset(out: &mut String, d: &super::model::ProductionDataset) {
    out.push_str(&format!(
        "d:{} a pf:ProductionDataset ;\n  pf:conformsToShape d:{} ;\n  pf:dataSource {}",
        d.id, d.shape, lit(&d.source)
    ));
    if let Some(l) = &d.label {
        out.push_str(&format!(" ;\n  rdfs:label {}", lit(l)));
    }
    out.push_str(" .\n\n");
}

/// Render a string as a Turtle literal, escaping per the grammar.
fn lit(s: &str) -> String {
    let escaped = s
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t");
    format!("\"{}\"", escaped)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pf::model::*;

    #[test]
    fn emits_prefixes_and_classes() {
        let mut g = DomainGraph::default();
        g.contexts.push(BoundedContext { id: "Tasks".into(), label: "Tasks".into(), ..Default::default() });
        g.entities.push(Entity { id: "Task".into(), label: "Task".into(), context: "Tasks".into(), definition: "a \"unit\"".into(), is_aggregate_root: true, ..Default::default() });
        let ttl = to_turtle(&g, "demo");
        assert!(ttl.contains("@prefix pf:"));
        assert!(ttl.contains("product/demo#"));
        assert!(ttl.contains("d:Task a pf:Entity"));
        assert!(ttl.contains("pf:inContext d:Tasks"));
        assert!(ttl.contains("isAggregateRoot"));
        assert!(ttl.contains("\\\"unit\\\""), "literal must be escaped: {ttl}");
    }
}
