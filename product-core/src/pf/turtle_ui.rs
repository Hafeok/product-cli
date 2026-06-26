//! Turtle emission for the §3.2.1–§4.6 UI-layer nodes.
//!
//! Split from [`super::turtle`] for the 400-line gate. Interactions a UI step
//! `surfaces`/`offers` are emitted as blank nodes carrying their `pf:typedAs`
//! AIO so the (projection, aio) / (command, aio) pairing round-trips
//! unambiguously (a flat `pf:typedAs` on the step could not be re-paired).

use super::model;
use super::turtle::lit;

/// §3.2.1 — a UI step: typed interactions as blank nodes, state meanings,
/// content/style references, and the deprecated free-text aliases.
pub(super) fn emit_wireframe(out: &mut String, w: &model::WireframeStep) {
    out.push_str(&format!("d:{} a pf:WireframeStep ;\n  rdfs:label {}", w.id, lit(&w.label)));
    if let Some(i) = &w.intent {
        out.push_str(&format!(" ;\n  pf:intent {}", lit(i)));
    }
    for s in &w.surfaces {
        out.push_str(&format!(" ;\n  pf:surfaces [ pf:projection d:{} ; pf:typedAs d:{} ]", s.projection, s.aio));
    }
    for o in &w.offers {
        out.push_str(&format!(" ;\n  pf:offers [ pf:command d:{} ; pf:typedAs d:{} ]", o.command, o.aio));
    }
    for m in &w.state_meanings {
        out.push_str(&format!(" ;\n  pf:stateMeaning [ pf:smProjection d:{} ; pf:smState {}", m.projection, lit(&m.state)));
        if let Some(meaning) = &m.meaning {
            out.push_str(&format!(" ; pf:smMeaning {}", lit(meaning)));
        }
        if let Some(waiver) = &m.waiver {
            out.push_str(&format!(" ; pf:smWaiver {}", lit(waiver)));
        }
        out.push_str(" ]");
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
    for s in &w.styles {
        out.push_str(&format!(" ;\n  pf:style {}", lit(s)));
    }
    if let Some(t) = &w.triggers {
        out.push_str(&format!(" ;\n  pf:triggers d:{}", t));
    }
    if let Some(d) = &w.displays {
        out.push_str(&format!(" ;\n  pf:displays d:{}", d));
    }
    out.push_str(" .\n\n");
}

/// §3.2.2 — an adopter-registered AIO and the WCAG criteria it carries.
pub(super) fn emit_aio(out: &mut String, a: &model::Aio) {
    out.push_str(&format!("d:{} a pf:Aio ;\n  rdfs:label {}", a.id, lit(&a.label)));
    if let Some(m) = &a.means {
        out.push_str(&format!(" ;\n  pf:means {}", lit(m)));
    }
    for c in &a.must_satisfy {
        out.push_str(&format!(" ;\n  pf:mustSatisfy d:{}", c));
    }
    out.push_str(" .\n\n");
}

/// §3.2.2 — a declared context of use (dimension + value).
pub(super) fn emit_context_of_use(out: &mut String, c: &model::ContextOfUse) {
    out.push_str(&format!("d:{} a pf:ContextOfUse ;\n  rdfs:label {}", c.id, lit(&c.label)));
    if let Some(d) = &c.dimension {
        out.push_str(&format!(" ;\n  pf:dimension {}", lit(d)));
    }
    if let Some(v) = &c.value {
        out.push_str(&format!(" ;\n  pf:contextValue {}", lit(v)));
    }
    out.push_str(" .\n\n");
}

/// §3.2.4 — the page-graph root and its global destinations.
pub(super) fn emit_application_root(out: &mut String, r: &model::ApplicationRoot) {
    out.push_str(&format!("d:{} a pf:ApplicationRoot", r.id));
    if let Some(l) = &r.label {
        out.push_str(&format!(" ;\n  rdfs:label {}", lit(l)));
    }
    for d in &r.navigates_from_root {
        out.push_str(&format!(" ;\n  pf:navigatesFromRoot d:{}", d));
    }
    out.push_str(" .\n\n");
}

/// §3.2.3 — an ingested WCAG success criterion.
pub(super) fn emit_wcag(out: &mut String, c: &model::WcagCriterion) {
    out.push_str(&format!("d:{} a pf:WcagCriterion", c.id));
    if let Some(l) = &c.label { out.push_str(&format!(" ;\n  rdfs:label {}", lit(l))); }
    if let Some(l) = &c.level { out.push_str(&format!(" ;\n  pf:level {}", lit(l))); }
    if let Some(v) = &c.verification { out.push_str(&format!(" ;\n  pf:verification {}", lit(v))); }
    out.push_str(&format!(" ;\n  pf:satisfied {} .\n\n", c.satisfied));
}

/// §3.2.3 — a dated, attributed attestation that a criterion was met.
pub(super) fn emit_attestation(out: &mut String, a: &model::Attestation) {
    out.push_str(&format!(
        "d:{} a pf:Attestation ;\n  pf:attestsStep d:{} ;\n  pf:attestsCriterion d:{} ;\n  pf:date {} ;\n  pf:attestedBy {} .\n\n",
        a.id, a.step, a.criterion, lit(&a.date), lit(&a.by)
    ));
}

/// §4.6 — a content store: locales + (key, locale) → value resolutions.
pub(super) fn emit_content_store(out: &mut String, s: &model::ContentStore) {
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

/// §4.5 — the design system: its CIO catalog + token surface.
pub(super) fn emit_design_system(out: &mut String, d: &model::DesignSystem) {
    out.push_str(&format!("d:{} a pf:DesignSystem", d.id));
    if let Some(l) = &d.label { out.push_str(&format!(" ;\n  rdfs:label {}", lit(l))); }
    for c in &d.cios { out.push_str(&format!(" ;\n  pf:hasCio d:{}", c)); }
    for t in &d.tokens { out.push_str(&format!(" ;\n  pf:hasToken {}", lit(t))); }
    out.push_str(" .\n\n");
}

/// §4.5 — a Concrete Interaction Object.
pub(super) fn emit_cio(out: &mut String, c: &model::Cio) {
    out.push_str(&format!("d:{} a pf:Cio", c.id));
    if let Some(l) = &c.label { out.push_str(&format!(" ;\n  rdfs:label {}", lit(l))); }
    out.push_str(" .\n\n");
}

/// §4.5 — a design token.
pub(super) fn emit_token(out: &mut String, t: &model::Token) {
    out.push_str(&format!("d:{} a pf:Token", t.id));
    if let Some(k) = &t.kind { out.push_str(&format!(" ;\n  pf:tokenKind {}", lit(k))); }
    out.push_str(" .\n\n");
}

/// §4.5 — a reify(AIO, context) → CIO rule with rationale.
pub(super) fn emit_reification_rule(out: &mut String, r: &model::ReificationRule) {
    out.push_str(&format!(
        "d:{} a pf:ReificationRule ;\n  pf:reifies d:{} ;\n  pf:inContext d:{} ;\n  pf:toCio d:{}",
        r.id, r.aio, r.context, r.cio
    ));
    if let Some(why) = &r.rationale { out.push_str(&format!(" ;\n  pf:rationale {}", lit(why))); }
    out.push_str(" .\n\n");
}

/// §4.5 — a declared coverage gap (an AIO unreifiable in an interaction class).
pub(super) fn emit_unreifiable(out: &mut String, u: &model::UnreifiableRule) {
    out.push_str(&format!("d:{} a pf:UnreifiableRule ;\n  pf:reifies d:{} ;\n  pf:unreifiableIn {}", u.id, u.aio, lit(&u.class)));
    if let Some(why) = &u.rationale {
        out.push_str(&format!(" ;\n  pf:rationale {}", lit(why)));
    }
    out.push_str(" .\n\n");
}
