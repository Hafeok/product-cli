//! Entry-point seeds over a C# inventory — the measurement vocabulary of Gate 1b (CG-R-105).
//!
//! One candidate per external integration point, by resolved symbol id:
//! controller actions (E-1), Razor page handlers (E-2), ViewComponents (E-3),
//! FastEndpoints endpoints (E-4), hosted services (E-5). A candidate carries
//! the transport name, the path, the HTTP method or trigger kind, and the
//! observed authorisation positions — and nothing that names an act, says
//! what it settles or who answers (CG-R-106). Expected actor kinds,
//! population and rate are unfilled ground slots (CG-R-108); supported
//! throughput is an invited determination, carried empty (CG-R-109). Where a
//! framework's own discovery rule is a name convention (page handlers,
//! `Invoke`) it is declared as a proxy in `csharp_candidates_report`, never
//! as this module's inference. The criterion in full:
//! `meta/sessions/2026-09-15-gate1b-candidates/gateA-criterion.md`.

use std::collections::BTreeMap;

use serde::Serialize;

use super::csharp_inventory::{AttributeUse, Index, MemberFact, TypeFact};

pub const CONTROLLER_BASE: &str = "T:Microsoft.AspNetCore.Mvc.ControllerBase";
pub const PAGE_MODEL: &str = "T:Microsoft.AspNetCore.Mvc.RazorPages.PageModel";
pub const VIEW_COMPONENT: &str = "T:Microsoft.AspNetCore.Mvc.ViewComponent";
pub const FASTENDPOINTS_BASE: &str = "T:FastEndpoints.BaseEndpoint";
pub const HOSTED: &[&str] = &["T:Microsoft.Extensions.Hosting.IHostedService", "T:Microsoft.Extensions.Hosting.BackgroundService"];
pub const NON_ACTION: &str = "T:Microsoft.AspNetCore.Mvc.NonActionAttribute";
pub const ROUTE: &str = "T:Microsoft.AspNetCore.Mvc.RouteAttribute";
pub const AREA: &str = "T:Microsoft.AspNetCore.Mvc.AreaAttribute";
pub const AUTHORIZE: &str = "T:Microsoft.AspNetCore.Authorization.AuthorizeAttribute";
pub const ALLOW_ANONYMOUS: &str = "T:Microsoft.AspNetCore.Authorization.AllowAnonymousAttribute";
pub const ANTI_FORGERY: &str = "T:Microsoft.AspNetCore.Mvc.ValidateAntiForgeryTokenAttribute";

/// HTTP-method attributes (E-1), by type id.
pub const HTTP_METHOD_ATTRIBUTES: &[(&str, &str)] = &[
    ("T:Microsoft.AspNetCore.Mvc.HttpGetAttribute", "GET"),
    ("T:Microsoft.AspNetCore.Mvc.HttpPostAttribute", "POST"),
    ("T:Microsoft.AspNetCore.Mvc.HttpPutAttribute", "PUT"),
    ("T:Microsoft.AspNetCore.Mvc.HttpDeleteAttribute", "DELETE"),
    ("T:Microsoft.AspNetCore.Mvc.HttpPatchAttribute", "PATCH"),
    ("T:Microsoft.AspNetCore.Mvc.HttpHeadAttribute", "HEAD"),
    ("T:Microsoft.AspNetCore.Mvc.HttpOptionsAttribute", "OPTIONS"),
    ("T:Microsoft.AspNetCore.Mvc.AcceptVerbsAttribute", "verbs (list not read)"),
];

/// FastEndpoints route-configuring members (E-4), by member-id prefix — the
/// overload's parameter list follows the `(`; the route argument is not read (L-EP-2).
pub const FASTENDPOINTS_METHODS: &[(&str, &str)] = &[
    ("M:FastEndpoints.Endpoint`2.Get(", "GET"),
    ("M:FastEndpoints.Endpoint`2.Post(", "POST"),
    ("M:FastEndpoints.Endpoint`2.Put(", "PUT"),
    ("M:FastEndpoints.Endpoint`2.Delete(", "DELETE"),
    ("M:FastEndpoints.Endpoint`2.Patch(", "PATCH"),
    ("M:FastEndpoints.Endpoint`2.Head(", "HEAD"),
    ("M:FastEndpoints.Endpoint`2.Verbs(", "verbs (list not read)"),
    ("M:FastEndpoints.Endpoint`2.Routes(", "routes (list not read)"),
];

/// FastEndpoints authorisation-configuring members, by member-id prefix.
pub const FASTENDPOINTS_AUTH: &[&str] = &[
    "M:FastEndpoints.Endpoint`2.AllowAnonymous(",
    "M:FastEndpoints.Endpoint`2.Roles(",
    "M:FastEndpoints.Endpoint`2.AuthSchemes(",
    "M:FastEndpoints.Endpoint`2.Claims(",
    "M:FastEndpoints.Endpoint`2.Permissions(",
    "M:FastEndpoints.Endpoint`2.Policies(",
    "M:FastEndpoints.Endpoint`2.Policy(",
];

/// The Razor Pages handler rule `On{Verb}[{Handler}][Async]` — the
/// framework's, declared as proxy P-EP-1. Longest verb first.
pub const HANDLER_VERBS: &[(&str, &str)] = &[("Options", "OPTIONS"), ("Delete", "DELETE"), ("Patch", "PATCH"), ("Post", "POST"), ("Head", "HEAD"), ("Get", "GET"), ("Put", "PUT")];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Kind {
    #[default]
    ControllerAction,
    RazorPageHandler,
    ViewComponent,
    /// Serialised as its label, `fastendpoints`, so JSON and text agree.
    #[serde(rename = "fastendpoints")]
    FastEndpoints,
    HostedService,
}

impl Kind {
    pub fn label(self) -> &'static str {
        match self {
            Kind::ControllerAction => "controller-action",
            Kind::RazorPageHandler => "razor-page-handler",
            Kind::ViewComponent => "view-component",
            Kind::FastEndpoints => "fastendpoints",
            Kind::HostedService => "hosted-service",
        }
    }
}

pub const OBSERVED_NOTE: &str = "observed: attributes on the member, the type and its in-solution bases; FastEndpoints calls by member id. Empty means no attribute or call was found — conventions in call arguments (AuthorizePage, AuthorizeFolder, a fallback policy) are not read (L-EP-1) — never that the endpoint is anonymous";

/// Authorisation evidence read from code, labelled observed (CG-R-108).
#[derive(Debug, Clone, Default, Serialize)]
pub struct Observed {
    pub authorisation: Vec<String>,
    /// Constant fields referenced from the member that made an authorisation
    /// call; the binding of symbol to call is not read (L-EP-2).
    pub argument_symbols: Vec<String>,
    pub anti_forgery: bool,
    /// `member → identity member` on the path, filled by the walk.
    pub identity_checks: Vec<String>,
    pub note: &'static str,
}

/// Unfilled ground slots: *not stated*, never *no actors* (CG-R-108).
#[derive(Debug, Clone, Serialize)]
pub struct Ground {
    pub expected_actor_kinds: Option<Vec<String>>,
    pub population_order_of_magnitude: Option<String>,
    pub rate_order_of_magnitude: Option<String>,
    pub note: &'static str,
}

impl Default for Ground {
    fn default() -> Self {
        Ground { expected_actor_kinds: None, population_order_of_magnitude: None, rate_order_of_magnitude: None, note: "unfilled ground slots — not stated, never no actors (CG-R-108); supplied at acceptance, expected actor kinds referencing Layer 1 kinds" }
    }
}

/// The invited determination, carried empty (CG-R-109).
#[derive(Debug, Clone, Serialize)]
pub struct Throughput {
    pub sustained: Option<String>,
    pub peak: Option<String>,
    pub window: Option<String>,
    pub behaviour_above_limit: Option<String>,
    pub note: &'static str,
}

impl Default for Throughput {
    fn default() -> Self {
        Throughput { sustained: None, peak: None, window: None, behaviour_above_limit: None, note: "invited determination — supported throughput is settled by someone after acceptance, not authored here (CG-R-109)" }
    }
}

/// One candidate. `path_types`, `path_edges`, `unscored`, `facts` and the
/// identity checks are filled by `csharp_candidates_path`.
#[derive(Debug, Clone, Default, Serialize)]
pub struct Candidate {
    pub id: String,
    pub kind: Kind,
    pub type_id: String,
    pub project: String,
    pub member: String,
    /// Walk roots: the handler member(s) plus the type's constructors, or the type.
    pub roots: Vec<String>,
    pub transport: String,
    pub method: String,
    pub path: String,
    pub path_source: String,
    /// `complete`, or `incomplete — missing: <field>`: a candidate without its
    /// route keeps its place, labelled; the route is supplied by hand at
    /// ratification, from the source (CG-R-114).
    pub identity: String,
    pub observed: Observed,
    pub path_types: Vec<String>,
    pub path_edges: BTreeMap<String, usize>,
    pub unscored: usize,
    pub facts: super::csharp_candidates_path::Facts,
    pub ground: Ground,
    pub supported_throughput: Throughput,
}

/// Does `type_id`'s base chain (in-solution `base_type`, then the external
/// chain) reach one of `targets`, by base or declared interface?
pub fn chain_reaches(ix: &Index<'_>, type_id: &str, targets: &[&str]) -> bool {
    let mut cur = Some(type_id.to_string());
    for _ in 0..32 {
        let Some(id) = cur else { return false };
        if id != type_id && targets.contains(&id.as_str()) {
            return true;
        }
        let (base, interfaces) = match ix.types.get(id.as_str()) {
            Some(t) => (t.base_type.clone(), &t.interfaces),
            None => match ix.external.get(id.as_str()) {
                Some(e) => (e.base_type.clone(), &e.interfaces),
                None => return false,
            },
        };
        if interfaces.iter().any(|i| targets.contains(&i.as_str())) {
            return true;
        }
        cur = base;
    }
    false
}

/// Public instance methods of a type, in inventory order.
pub fn public_methods<'a>(ix: &Index<'a>, type_id: &str) -> Vec<&'a MemberFact> {
    ix.members_of.get(type_id).into_iter().flatten().filter_map(|m| ix.members.get(m).copied()).filter(|m| m.kind == "method" && m.accessibility == "public" && !m.is_static).collect()
}

pub fn constructors<'a>(ix: &Index<'a>, type_id: &str) -> Vec<&'a str> {
    ix.members_of.get(type_id).into_iter().flatten().copied().filter(|m| ix.members.get(m).is_some_and(|f| f.kind == "constructor")).collect()
}

/// The in-solution base chain of a type, base-most first, the type last.
pub fn solution_chain<'a>(ix: &Index<'a>, type_id: &str) -> Vec<&'a TypeFact> {
    let mut chain = Vec::new();
    let mut cur = ix.types.get(type_id).copied();
    while let Some(t) = cur {
        if chain.len() > 32 || chain.iter().any(|c: &&TypeFact| c.id == t.id) {
            break;
        }
        chain.push(t);
        cur = t.base_type.as_deref().and_then(|b| ix.types.get(b).copied());
    }
    chain.reverse();
    chain
}

/// Type name without a trailing `Controller`, the framework's `[controller]` token.
pub fn controller_token(name: &str) -> &str {
    name.strip_suffix("Controller").unwrap_or(name)
}

/// `On{Verb}[{Handler}][Async]` → (HTTP method, handler name), P-EP-1.
pub fn handler_of(name: &str) -> Option<(&'static str, &str)> {
    let rest = name.strip_prefix("On")?;
    for (prefix, verb) in HANDLER_VERBS {
        if let Some(h) = rest.strip_prefix(prefix) {
            let handler = h.strip_suffix("Async").unwrap_or(h);
            if handler.is_empty() || handler.starts_with(|c: char| c.is_ascii_uppercase()) {
                return Some((verb, handler));
            }
        }
    }
    None
}

/// The page route from the code-behind's file path, P-EP-2: `Pages/X/Y.cshtml.cs`
/// → `/X/Y`; `Areas/A/Pages/X.cshtml.cs` → `/A/X`.
pub fn page_path(file: &str) -> Option<String> {
    let f = file.replace('\\', "/");
    let stem = |s: &str| s.strip_suffix(".cshtml.cs").map(str::to_string);
    if let Some(i) = f.find("/Areas/") {
        let after = &f[i + "/Areas/".len()..];
        let (area, rest) = after.split_once("/Pages/")?;
        return stem(rest).map(|r| format!("/{area}/{r}"));
    }
    let i = f.find("/Pages/")?;
    stem(&f[i + "/Pages/".len()..]).map(|r| format!("/{r}"))
}

/// Authorisation attributes on a member, a type and its in-solution bases.
pub fn authorisation(ix: &Index<'_>, type_id: &str, member: Option<&MemberFact>) -> (Vec<String>, bool) {
    let mut out = Vec::new();
    for t in solution_chain(ix, type_id) {
        let site = if t.id == type_id { "type".to_string() } else { format!("base {}", t.name) };
        out.extend(t.attributes.iter().filter_map(|a| auth_label(a, &site)));
    }
    let mut anti_forgery = false;
    if let Some(m) = member {
        out.extend(m.attributes.iter().filter_map(|a| auth_label(a, "member")));
        anti_forgery = m.attributes.iter().any(|a| a.attribute_type == ANTI_FORGERY);
    }
    (out, anti_forgery)
}

fn auth_label(a: &AttributeUse, site: &str) -> Option<String> {
    let args: Vec<String> = a.named.iter().map(|(k, v)| format!("{k}={}", v.as_str().unwrap_or(&v.to_string()))).collect();
    let args = if args.is_empty() { String::new() } else { format!(" ({})", args.join(", ")) };
    match a.attribute_type.as_str() {
        AUTHORIZE => Some(format!("Authorize on {site}{args}")),
        ALLOW_ANONYMOUS => Some(format!("AllowAnonymous on {site}")),
        _ => None,
    }
}

/// The last namespace segment, for the recall key.
pub fn namespace_tail(t: &TypeFact) -> &str {
    t.namespace.rsplit('.').next().unwrap_or("")
}

#[cfg(test)]
#[path = "csharp_candidates_tests.rs"]
pub(crate) mod tests;
