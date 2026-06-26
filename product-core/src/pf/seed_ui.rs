//! Turtle parsing for the §3.2.1–§4.6 UI-layer nodes.
//!
//! Peer of [`super::turtle_ui`]: reconstructs UI steps (with their blank-node
//! `surfaces`/`offers`/`stateMeaning`/`referencesContent` groups), AIOs, the
//! page graph, accessibility, content, and the design system. Split from
//! [`super::seed`] for the 400-line gate.

use std::collections::HashMap;

use oxigraph::store::Store;

use super::model::*;
use super::seed::{lit, local, multi, opt, select};
use crate::error::Result;

/// Parse every UI-layer kind into `g`.
pub(super) fn parse_ui(store: &Store, g: &mut DomainGraph) -> Result<()> {
    parse_wireframes(store, g)?;
    parse_aios(store, g)?;
    parse_contexts_of_use(store, g)?;
    parse_application_roots(store, g)?;
    parse_wcag(store, g)?;
    parse_attestations(store, g)?;
    parse_content_stores(store, g)?;
    parse_design_systems(store, g)?;
    parse_cios(store, g)?;
    parse_tokens(store, g)?;
    parse_reification_rules(store, g)?;
    Ok(())
}

/// Collect a UI step's surface interactions, keyed by step id.
fn step_surfaces(store: &Store) -> Result<HashMap<String, Vec<Surface>>> {
    let mut out: HashMap<String, Vec<Surface>> = HashMap::new();
    for row in select(store, "?s ?proj ?aio",
        "?s a pf:WireframeStep ; pf:surfaces ?b . ?b pf:projection ?proj ; pf:typedAs ?aio")? {
        out.entry(local(row.get("s"))).or_default()
            .push(Surface { projection: local(row.get("proj")), aio: local(row.get("aio")) });
    }
    Ok(out)
}

/// Collect a UI step's offered interactions, keyed by step id.
fn step_offers(store: &Store) -> Result<HashMap<String, Vec<Offer>>> {
    let mut out: HashMap<String, Vec<Offer>> = HashMap::new();
    for row in select(store, "?s ?cmd ?aio",
        "?s a pf:WireframeStep ; pf:offers ?b . ?b pf:command ?cmd ; pf:typedAs ?aio")? {
        out.entry(local(row.get("s"))).or_default()
            .push(Offer { command: local(row.get("cmd")), aio: local(row.get("aio")) });
    }
    Ok(out)
}

/// Collect a UI step's state meanings, keyed by step id.
fn step_meanings(store: &Store) -> Result<HashMap<String, Vec<StateMeaning>>> {
    let mut out: HashMap<String, Vec<StateMeaning>> = HashMap::new();
    for row in select(store, "?s ?proj ?st ?meaning ?waiver",
        "?s a pf:WireframeStep ; pf:stateMeaning ?b . ?b pf:smProjection ?proj ; pf:smState ?st . OPTIONAL { ?b pf:smMeaning ?meaning } OPTIONAL { ?b pf:smWaiver ?waiver }")? {
        out.entry(local(row.get("s"))).or_default().push(StateMeaning {
            projection: local(row.get("proj")), state: lit(row.get("st")),
            meaning: opt(row.get("meaning")), waiver: opt(row.get("waiver")),
        });
    }
    Ok(out)
}

/// Collect a UI step's content references, keyed by step id.
fn step_content_refs(store: &Store) -> Result<HashMap<String, Vec<ContentRef>>> {
    let mut out: HashMap<String, Vec<ContentRef>> = HashMap::new();
    for row in select(store, "?s ?key ?role",
        "?s a pf:WireframeStep ; pf:referencesContent ?b . ?b pf:contentKey ?key ; pf:role ?role")? {
        out.entry(local(row.get("s"))).or_default()
            .push(ContentRef { key: lit(row.get("key")), role: lit(row.get("role")) });
    }
    Ok(out)
}

fn parse_wireframes(store: &Store, g: &mut DomainGraph) -> Result<()> {
    let mut surfaces = step_surfaces(store)?;
    let mut offers = step_offers(store)?;
    let mut meanings = step_meanings(store)?;
    let mut refs = step_content_refs(store)?;
    let transitions = multi(store, "pf:WireframeStep", "pf:transitionsTo")?;
    let satisfy = multi(store, "pf:WireframeStep", "pf:mustSatisfy")?;
    let styles = multi(store, "pf:WireframeStep", "pf:style")?;
    for row in select(store, "?s ?label ?intent ?triggers ?displays",
        "?s a pf:WireframeStep . OPTIONAL { ?s rdfs:label ?label } OPTIONAL { ?s pf:intent ?intent } OPTIONAL { ?s pf:triggers ?triggers } OPTIONAL { ?s pf:displays ?displays }")? {
        let id = local(row.get("s"));
        g.wireframe_steps.push(WireframeStep {
            label: lit(row.get("label")), intent: opt(row.get("intent")),
            surfaces: surfaces.remove(&id).unwrap_or_default(),
            offers: offers.remove(&id).unwrap_or_default(),
            transitions_to: transitions.get(&id).cloned().unwrap_or_default(),
            state_meanings: meanings.remove(&id).unwrap_or_default(),
            must_satisfy: satisfy.get(&id).cloned().unwrap_or_default(),
            content_refs: refs.remove(&id).unwrap_or_default(),
            styles: styles.get(&id).cloned().unwrap_or_default(),
            triggers: opt(row.get("triggers")).map(|_| local(row.get("triggers"))),
            displays: opt(row.get("displays")).map(|_| local(row.get("displays"))),
            id,
        });
    }
    Ok(())
}

fn parse_aios(store: &Store, g: &mut DomainGraph) -> Result<()> {
    let satisfy = multi(store, "pf:Aio", "pf:mustSatisfy")?;
    for row in select(store, "?s ?label ?means",
        "?s a pf:Aio . OPTIONAL { ?s rdfs:label ?label } OPTIONAL { ?s pf:means ?means }")? {
        let id = local(row.get("s"));
        g.aios.push(Aio {
            must_satisfy: satisfy.get(&id).cloned().unwrap_or_default(),
            id: id.clone(), label: lit(row.get("label")), means: opt(row.get("means")),
        });
    }
    Ok(())
}

fn parse_contexts_of_use(store: &Store, g: &mut DomainGraph) -> Result<()> {
    for row in select(store, "?s ?label ?dim ?val",
        "?s a pf:ContextOfUse . OPTIONAL { ?s rdfs:label ?label } OPTIONAL { ?s pf:dimension ?dim } OPTIONAL { ?s pf:contextValue ?val }")? {
        g.contexts_of_use.push(ContextOfUse {
            id: local(row.get("s")), label: lit(row.get("label")),
            dimension: opt(row.get("dim")), value: opt(row.get("val")),
        });
    }
    Ok(())
}

fn parse_application_roots(store: &Store, g: &mut DomainGraph) -> Result<()> {
    let dests = multi(store, "pf:ApplicationRoot", "pf:navigatesFromRoot")?;
    for row in select(store, "?s ?label", "?s a pf:ApplicationRoot . OPTIONAL { ?s rdfs:label ?label }")? {
        let id = local(row.get("s"));
        g.application_roots.push(ApplicationRoot {
            navigates_from_root: dests.get(&id).cloned().unwrap_or_default(),
            id: id.clone(), label: opt(row.get("label")),
        });
    }
    Ok(())
}

fn parse_wcag(store: &Store, g: &mut DomainGraph) -> Result<()> {
    for row in select(store, "?s ?label ?level ?verif ?sat",
        "?s a pf:WcagCriterion . OPTIONAL { ?s rdfs:label ?label } OPTIONAL { ?s pf:level ?level } OPTIONAL { ?s pf:verification ?verif } OPTIONAL { ?s pf:satisfied ?sat }")? {
        g.wcag_criteria.push(WcagCriterion {
            id: local(row.get("s")), label: opt(row.get("label")),
            level: opt(row.get("level")), verification: opt(row.get("verif")),
            satisfied: lit(row.get("sat")) == "true",
        });
    }
    Ok(())
}

fn parse_attestations(store: &Store, g: &mut DomainGraph) -> Result<()> {
    for row in select(store, "?s ?step ?crit ?date ?by",
        "?s a pf:Attestation . OPTIONAL { ?s pf:attestsStep ?step } OPTIONAL { ?s pf:attestsCriterion ?crit } OPTIONAL { ?s pf:date ?date } OPTIONAL { ?s pf:attestedBy ?by }")? {
        g.attestations.push(Attestation {
            id: local(row.get("s")), step: local(row.get("step")), criterion: local(row.get("crit")),
            date: lit(row.get("date")), by: lit(row.get("by")),
        });
    }
    Ok(())
}

fn parse_content_stores(store: &Store, g: &mut DomainGraph) -> Result<()> {
    let locales = multi(store, "pf:ContentStore", "pf:locale")?;
    let mut resolutions: HashMap<String, Vec<Resolution>> = HashMap::new();
    for row in select(store, "?s ?key ?loc ?val",
        "?s a pf:ContentStore ; pf:resolves ?b . ?b pf:contentKey ?key ; pf:inLocale ?loc ; pf:value ?val")? {
        resolutions.entry(local(row.get("s"))).or_default().push(Resolution {
            key: lit(row.get("key")), locale: lit(row.get("loc")), value: lit(row.get("val")),
        });
    }
    for row in select(store, "?s ?label", "?s a pf:ContentStore . OPTIONAL { ?s rdfs:label ?label }")? {
        let id = local(row.get("s"));
        g.content_stores.push(ContentStore {
            locales: locales.get(&id).cloned().unwrap_or_default(),
            resolutions: resolutions.remove(&id).unwrap_or_default(),
            id: id.clone(), label: opt(row.get("label")),
        });
    }
    Ok(())
}

fn parse_design_systems(store: &Store, g: &mut DomainGraph) -> Result<()> {
    let cios = multi(store, "pf:DesignSystem", "pf:hasCio")?;
    let tokens = multi(store, "pf:DesignSystem", "pf:hasToken")?;
    for row in select(store, "?s ?label", "?s a pf:DesignSystem . OPTIONAL { ?s rdfs:label ?label }")? {
        let id = local(row.get("s"));
        g.design_systems.push(DesignSystem {
            cios: cios.get(&id).cloned().unwrap_or_default(),
            tokens: tokens.get(&id).cloned().unwrap_or_default(),
            id: id.clone(), label: opt(row.get("label")),
        });
    }
    Ok(())
}

fn parse_cios(store: &Store, g: &mut DomainGraph) -> Result<()> {
    for row in select(store, "?s ?label", "?s a pf:Cio . OPTIONAL { ?s rdfs:label ?label }")? {
        g.cios.push(Cio { id: local(row.get("s")), label: opt(row.get("label")) });
    }
    Ok(())
}

fn parse_tokens(store: &Store, g: &mut DomainGraph) -> Result<()> {
    for row in select(store, "?s ?kind", "?s a pf:Token . OPTIONAL { ?s pf:tokenKind ?kind }")? {
        g.tokens.push(Token { id: local(row.get("s")), kind: opt(row.get("kind")) });
    }
    Ok(())
}

fn parse_reification_rules(store: &Store, g: &mut DomainGraph) -> Result<()> {
    for row in select(store, "?s ?aio ?ctx ?cio ?rat",
        "?s a pf:ReificationRule . OPTIONAL { ?s pf:reifies ?aio } OPTIONAL { ?s pf:inContext ?ctx } OPTIONAL { ?s pf:toCio ?cio } OPTIONAL { ?s pf:rationale ?rat }")? {
        g.reification_rules.push(ReificationRule {
            id: local(row.get("s")), aio: local(row.get("aio")), context: local(row.get("ctx")),
            cio: local(row.get("cio")), rationale: opt(row.get("rat")),
        });
    }
    Ok(())
}
