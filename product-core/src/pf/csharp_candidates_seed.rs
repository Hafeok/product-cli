//! The seeds — one candidate per integration point the criterion admits (Gate 1b, E-1 … E-5).
//!
//! Types are classified by resolved base chain; each kind's entry members
//! are read as `csharp_candidates` states. The two framework name rules
//! (page handlers, `Invoke`) are counted here so the report can print their
//! incidence (CG-R-77).

use super::csharp_candidates::{
    authorisation, chain_reaches, constructors, controller_token, handler_of, namespace_tail, page_path, public_methods, solution_chain, Candidate, Kind, Observed, AREA, CONTROLLER_BASE,
    FASTENDPOINTS_AUTH, FASTENDPOINTS_BASE, FASTENDPOINTS_METHODS, HOSTED, HTTP_METHOD_ATTRIBUTES, NON_ACTION, OBSERVED_NOTE, PAGE_MODEL, ROUTE, VIEW_COMPONENT,
};
use super::csharp_inventory::{Index, Inventory, MemberFact, TypeFact};

/// The candidates plus the counts behind the proxies' incidences.
#[derive(Debug, Default)]
pub struct Seeds {
    pub candidates: Vec<Candidate>,
    /// Public `PageModel` methods (matching the handler rule, not matching).
    pub handler_rule: (usize, usize),
    /// ViewComponents (with `Invoke`/`InvokeAsync`, without).
    pub invoke_rule: (usize, usize),
    /// Types the criterion admits by base but that yield no candidate, with why.
    pub excluded: Vec<String>,
}

struct Spec<'s> {
    kind: Kind,
    member: &'s str,
    transport: &'s str,
    method: &'s str,
    path: &'s str,
    path_source: &'s str,
}

fn base(t: &TypeFact, spec: &Spec<'_>, roots: Vec<String>) -> Candidate {
    Candidate {
        id: format!("{}:{}@{}.{}#{}", spec.kind.label(), t.name, namespace_tail(t), spec.member, spec.method),
        kind: spec.kind,
        type_id: t.id.clone(),
        project: t.project.clone(),
        member: spec.member.to_string(),
        roots,
        transport: spec.transport.to_string(),
        method: spec.method.to_string(),
        path: spec.path.to_string(),
        path_source: spec.path_source.to_string(),
        identity: "complete".to_string(),
        observed: Observed { note: OBSERVED_NOTE, ..Default::default() },
        ..Default::default()
    }
}

/// Every candidate the inventory yields under the criterion.
pub fn seeds(inv: &Inventory, ix: &Index<'_>) -> Seeds {
    let mut s = Seeds::default();
    for t in inv.types.iter().filter(|t| !ix.is_test(&t.id) && t.kind == "class") {
        if chain_reaches(ix, &t.id, &[CONTROLLER_BASE]) {
            controller(ix, t, &mut s);
        } else if chain_reaches(ix, &t.id, &[PAGE_MODEL]) {
            page(ix, t, &mut s);
        } else if chain_reaches(ix, &t.id, &[VIEW_COMPONENT]) {
            view_component(ix, t, &mut s);
        } else if chain_reaches(ix, &t.id, &[FASTENDPOINTS_BASE]) {
            fast_endpoint(ix, t, &mut s);
        } else if chain_reaches(ix, &t.id, HOSTED) {
            let spec = Spec { kind: Kind::HostedService, member: "(type)", transport: "hosted", method: "hosted", path: "none — a trigger, not an address", path_source: "hosted service (E-5)" };
            let mut c = base(t, &spec, vec![t.id.clone()]);
            (c.observed.authorisation, c.observed.anti_forgery) = authorisation(ix, &t.id, None);
            s.candidates.push(c);
        }
    }
    s
}

fn with_roots(ix: &Index<'_>, t: &TypeFact, m: &MemberFact) -> Vec<String> {
    std::iter::once(m.id.clone()).chain(constructors(ix, &t.id).into_iter().map(str::to_string)).collect()
}

/// E-1: public instance methods without `[NonAction]`.
fn controller(ix: &Index<'_>, t: &TypeFact, s: &mut Seeds) {
    let actions: Vec<&MemberFact> = public_methods(ix, &t.id).into_iter().filter(|m| !m.attributes.iter().any(|a| a.attribute_type == NON_ACTION)).collect();
    if t.is_abstract || actions.is_empty() {
        s.excluded.push(format!("{}: controller type with no action{}", t.id, if t.is_abstract { " (abstract)" } else { "" }));
        return;
    }
    for m in actions {
        let verbs: Vec<&str> = HTTP_METHOD_ATTRIBUTES.iter().filter(|(id, _)| m.attributes.iter().any(|a| a.attribute_type == *id)).map(|(_, v)| *v).collect();
        let method = if verbs.is_empty() { "any (conventional)".to_string() } else { verbs.join("|") };
        let (path, path_source) = route(ix, t, m);
        let spec = Spec { kind: Kind::ControllerAction, member: &m.name, transport: "HTTP", method: &method, path: &path, path_source: &path_source };
        let mut c = base(t, &spec, with_roots(ix, t, m));
        if path == "conventional" {
            c.identity = "incomplete — missing: route (the conventional template is a call argument, L-EP-1; CG-R-114)".to_string();
        }
        (c.observed.authorisation, c.observed.anti_forgery) = authorisation(ix, &t.id, Some(m));
        s.candidates.push(c);
    }
}

/// Attribute route: the nearest type in the chain declaring `[Route]`
/// supplies the prefix; the member's `[Route]` or `[Http*("…")]` the rest.
fn route(ix: &Index<'_>, t: &TypeFact, m: &MemberFact) -> (String, String) {
    let chain = solution_chain(ix, &t.id);
    let type_template = chain.iter().rev().find_map(|c| c.attributes.iter().filter(|a| a.attribute_type == ROUTE).find_map(|a| a.arg(0))).unwrap_or("");
    let is_method = |a: &&super::csharp_inventory::AttributeUse| a.attribute_type == ROUTE || HTTP_METHOD_ATTRIBUTES.iter().any(|(id, _)| *id == a.attribute_type);
    let member_template = m.attributes.iter().filter(is_method).find_map(|a| a.arg(0)).unwrap_or("");
    let raw = if member_template.starts_with('/') || member_template.starts_with("~/") {
        member_template.trim_start_matches('~').to_string()
    } else {
        [type_template, member_template].iter().filter(|p| !p.is_empty()).copied().collect::<Vec<_>>().join("/")
    };
    if raw.is_empty() {
        return ("conventional".to_string(), "conventional — the MapControllerRoute template is a call argument the reader does not emit (L-EP-1)".to_string());
    }
    let area = chain.iter().rev().find_map(|c| c.attributes.iter().filter(|a| a.attribute_type == AREA).find_map(|a| a.arg(0))).unwrap_or("[area]");
    let substituted = raw.replace("[controller]", controller_token(&t.name)).replace("[action]", &m.name).replace("[area]", area);
    (format!("/{}", substituted.trim_start_matches('/')), format!("attribute template: {raw}"))
}

/// E-2: handlers by the framework's name rule (P-EP-1); a page without one
/// still renders on GET.
fn page(ix: &Index<'_>, t: &TypeFact, s: &mut Seeds) {
    let base_path = page_path(&t.file).unwrap_or_else(|| "not derived — no Pages/ segment in the code-behind's file path".to_string());
    let mut any = false;
    for m in public_methods(ix, &t.id) {
        let Some((verb, handler)) = handler_of(&m.name) else {
            s.handler_rule.1 += 1;
            continue;
        };
        s.handler_rule.0 += 1;
        any = true;
        let path = if handler.is_empty() { base_path.clone() } else { format!("{base_path}?handler={handler}") };
        let spec = Spec { kind: Kind::RazorPageHandler, member: &m.name, transport: "HTTP", method: verb, path: &path, path_source: "code-behind file path (P-EP-2)" };
        let mut c = base(t, &spec, with_roots(ix, t, m));
        (c.observed.authorisation, c.observed.anti_forgery) = authorisation(ix, &t.id, Some(m));
        s.candidates.push(c);
    }
    if !any {
        let spec = Spec { kind: Kind::RazorPageHandler, member: "(render)", transport: "HTTP", method: "GET", path: &base_path, path_source: "code-behind file path (P-EP-2); no handler method — the page renders on GET" };
        let mut c = base(t, &spec, vec![t.id.clone()]);
        (c.observed.authorisation, c.observed.anti_forgery) = authorisation(ix, &t.id, None);
        s.candidates.push(c);
    }
}

/// E-3: `Invoke` / `InvokeAsync` (P-EP-3).
fn view_component(ix: &Index<'_>, t: &TypeFact, s: &mut Seeds) {
    let invokes: Vec<&MemberFact> = public_methods(ix, &t.id).into_iter().filter(|m| m.name == "Invoke" || m.name == "InvokeAsync").collect();
    if invokes.is_empty() {
        s.invoke_rule.1 += 1;
        s.excluded.push(format!("{}: ViewComponent without Invoke/InvokeAsync", t.id));
        return;
    }
    s.invoke_rule.0 += 1;
    for m in invokes {
        let spec = Spec { kind: Kind::ViewComponent, member: &m.name, transport: "view-component", method: "view-component", path: "(invoked from a Razor view the reader does not read, CG-R-78)", path_source: "framework-activated from a view (E-3)" };
        let mut c = base(t, &spec, with_roots(ix, t, m));
        (c.observed.authorisation, c.observed.anti_forgery) = authorisation(ix, &t.id, Some(m));
        s.candidates.push(c);
    }
}

/// E-4: the verb from the configuring call, by member id; the route is not read.
fn fast_endpoint(ix: &Index<'_>, t: &TypeFact, s: &mut Seeds) {
    let (mut verbs, mut symbols) = (Vec::new(), Vec::new());
    let mut auth: std::collections::BTreeMap<String, Vec<String>> = std::collections::BTreeMap::new();
    for m in ix.members_of.get(t.id.as_str()).into_iter().flatten() {
        let mut configuring = false;
        for r in ix.out.get(*m).into_iter().flatten().filter(|r| r.kind == "call") {
            verbs.extend(FASTENDPOINTS_METHODS.iter().filter(|(p, _)| r.to.starts_with(p)).map(|(_, v)| v.to_string()));
            for p in FASTENDPOINTS_AUTH.iter().filter(|p| r.to.starts_with(*p)) {
                configuring = true;
                auth.entry(p.trim_end_matches('(').rsplit('.').next().unwrap_or(p).to_string()).or_default().push(short(m));
            }
        }
        if configuring {
            symbols.extend(ix.out.get(*m).into_iter().flatten().filter(|r| r.kind == "access" && r.to.starts_with("F:")).map(|r| r.to.clone()));
        }
    }
    verbs.sort();
    verbs.dedup();
    symbols.sort();
    symbols.dedup();
    let method = if verbs.is_empty() { "not read".to_string() } else { verbs.join("|") };
    let spec = Spec { kind: Kind::FastEndpoints, member: "(type)", transport: "HTTP", method: &method, path: "not read (L-EP-2)", path_source: "the route is an argument to the configuring call, which the reader does not emit (L-EP-2)" };
    let mut c = base(t, &spec, vec![t.id.clone()]);
    c.identity = "incomplete — missing: route (a call argument the reader does not emit, L-EP-2; supplied by hand at ratification, CG-R-114)".to_string();
    (c.observed.authorisation, c.observed.anti_forgery) = authorisation(ix, &t.id, None);
    c.observed.authorisation.extend(auth.into_iter().map(|(call, members)| format!("{call}(..) called in {}", members.join(", "))));
    c.observed.argument_symbols = symbols;
    s.candidates.push(c);
}

/// A symbol id for display: the last two dotted segments before any `(`.
pub fn short(id: &str) -> String {
    let body = id.split_once(':').map(|(_, b)| b).unwrap_or(id);
    let body = body.split('(').next().unwrap_or(body);
    let segs: Vec<&str> = body.rsplit('.').take(2).collect();
    segs.into_iter().rev().collect::<Vec<_>>().join(".")
}
