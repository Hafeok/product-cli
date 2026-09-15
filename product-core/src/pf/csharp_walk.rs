//! The walk — one graph, composition edges resolved through the container.
//!
//! From the root symbols, `call`, `construct`, `access` and `type-reference`
//! edges are followed directly inside the solution. A **composition edge** —
//! a constructor parameter of a type the container constructs, or a
//! service-locator argument — is where the container chooses an
//! implementation: its target's role is read from [`csharp_roles`], roles
//! outside the denominator are recorded as excluded, factory/provider
//! targets are recorded as *partial*, an external type a reached but
//! unparsed framework call registers is *registration-not-read* (CG-R-75),
//! an external abstraction nothing in the solution implements or registers
//! is a *boundary* edge (CG-R-68 — the used library surface, outside the
//! denominator), and the rest are resolved through [`Resolver`] or recorded
//! unresolved with the reason (CG-R-62). Container-constructed is the root
//! set plus every implementation a reached registration names (O-17).
//! A property carrying a property-injection attribute (matched by symbol id)
//! on a container-constructed type is a composition edge too. Registrations
//! count once the member holding them is reached, so the walk repeats to a
//! fixpoint.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use serde::Serialize;

use super::csharp_di::{Reason, Resolver};
use super::csharp_inventory::Index;
use super::csharp_roles::{role_of, Role, COLLECTION_IDS};

/// What happened at one composition edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum EdgeState {
    Resolved,
    Partial,
    /// An external abstraction with no in-solution implementation and no
    /// registration naming it: satisfied by the library itself (CG-R-68).
    Boundary,
    /// Registered by a framework call the resolver does not parse (the
    /// call's name): a reader gap, not a boundary (CG-R-75).
    RegistrationNotRead(&'static str),
    Unresolved(Reason),
    Excluded(Role),
}

/// Property-injection attributes, by resolved symbol id: a property the
/// framework satisfies from the container at composition time is a
/// composition edge by the criterion (CG-R-68). Method injection needs the
/// parameter's attributes, which the reader does not emit — a stated gap.
pub const PROPERTY_INJECTION_ATTRIBUTES: &[&str] = &[
    "T:Microsoft.AspNetCore.Components.InjectAttribute",
    "T:Microsoft.AspNetCore.Mvc.FromServicesAttribute",
];

/// How the dependency arrives: one implementation, or every registration of
/// the type argument of a collection parameter (P-6 — its own classification,
/// CG-R-77).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Injection {
    Single,
    Collection,
}

/// One composition edge, from the referencing type to the abstraction.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct CompositionEdge {
    pub from: String,
    pub target: String,
    pub role: Role,
    pub state: EdgeState,
    pub injection: Injection,
}

/// What one walk reached, and what it decided at every composition edge.
#[derive(Debug, Default)]
pub struct Closure<'a> {
    pub members: BTreeSet<&'a str>,
    pub types: BTreeSet<&'a str>,
    /// Types the container constructs: roots and resolved implementations.
    pub container_types: BTreeSet<&'a str>,
    pub edges: BTreeSet<CompositionEdge>,
}

impl Closure<'_> {
    pub fn count(&self, pred: impl Fn(&EdgeState) -> bool) -> usize {
        self.edges.iter().filter(|e| pred(&e.state)).count()
    }

    /// Targets of unresolved (or partial) edges — the abstractions whose
    /// implementors are *unresolved* rather than unreached.
    pub fn targets_where(&self, pred: impl Fn(&EdgeState) -> bool) -> BTreeSet<&str> {
        self.edges.iter().filter(|e| pred(&e.state)).map(|e| e.target.as_str()).collect()
    }
}

/// Walk to a fixpoint over the site-reached set.
pub fn closure<'a>(ix: &Index<'a>, resolver: &Resolver, start: &BTreeSet<&'a str>) -> Closure<'a> {
    closure_from(ix, resolver, start, &BTreeSet::new(), true)
}

/// Walk to a fixpoint with registration sites the walk itself need not reach
/// (`base_sites`, e.g. a host's own closure), optionally without O-17's pull
/// — the per-candidate path of Gate 1b. With no base sites and O-17 on, this
/// is [`closure`] exactly.
pub fn closure_from<'a>(ix: &Index<'a>, resolver: &Resolver, start: &BTreeSet<&'a str>, base_sites: &BTreeSet<&'a str>, o17: bool) -> Closure<'a> {
    let mut previous: BTreeSet<&'a str> = BTreeSet::new();
    for _ in 0..8 {
        let c = walk(ix, resolver, start, &Sites { own: &previous, base: base_sites }, o17);
        if c.members == previous {
            return c;
        }
        previous = c.members;
    }
    walk(ix, resolver, start, &Sites { own: &previous, base: base_sites }, o17)
}

/// The registration sites a walk may read: its own previous pass plus a base set.
struct Sites<'w, 'a> {
    own: &'w BTreeSet<&'a str>,
    base: &'w BTreeSet<&'a str>,
}

impl Sites<'_, '_> {
    fn reached(&self, s: &str) -> bool {
        self.own.contains(s) || self.base.contains(s)
    }
}

struct Walk<'w, 'a> {
    ix: &'w Index<'a>,
    resolver: &'w Resolver,
    sites: &'w Sites<'w, 'a>,
    queue: VecDeque<&'a str>,
    c: Closure<'a>,
}

fn walk<'a>(ix: &Index<'a>, resolver: &Resolver, start: &BTreeSet<&'a str>, sites: &Sites<'_, 'a>, o17: bool) -> Closure<'a> {
    let mut w = Walk { ix, resolver, sites, queue: start.iter().copied().collect(), c: Closure::default() };
    for id in start {
        let type_id = ix.members.get(id).map(|m| m.declaring_type.as_str()).unwrap_or(id);
        w.c.container_types.insert(type_id);
        w.queue.extend(ix.members_of.get(id).into_iter().flatten().copied());
    }
    // O-17: every implementation a reached registration names is constructed
    // by the container, whether or not an edge asks for it.
    let pulled = if o17 { resolver.implementations_at(&|s: &str| sites.reached(s)) } else { BTreeSet::new() };
    for implementation in pulled {
        if let Some(t) = ix.types.get(implementation).map(|t| t.id.as_str()) {
            w.c.container_types.insert(t);
            w.queue.push_back(t);
            w.queue.extend(ix.members_of.get(t).into_iter().flatten().copied());
        }
    }
    while let Some(id) = w.queue.pop_front() {
        w.visit(id);
    }
    w.c
}

impl<'a> Walk<'_, 'a> {
    fn visit(&mut self, id: &'a str) {
        let Some(m) = self.ix.members.get(id) else {
            if self.ix.types.contains_key(id) {
                self.c.types.insert(id);
            }
            return;
        };
        if !self.c.members.insert(id) {
            return;
        }
        let from_type = m.declaring_type.as_str();
        self.c.types.insert(from_type);
        let composed = self.c.container_types.contains(from_type);
        if m.kind == "constructor" && composed {
            for p in &m.parameters {
                self.dependency(from_type, &p.parameter_type, &p.type_arguments);
            }
        }
        let is_injected = m.kind == "property" && composed && m.attributes.iter().any(|a| PROPERTY_INJECTION_ATTRIBUTES.contains(&a.attribute_type.as_str()));
        if is_injected {
            if let Some(t) = m.return_type.as_deref() {
                self.dependency(from_type, t, &m.return_type_arguments);
            }
        }
        for r in self.ix.out.get(id).into_iter().flatten() {
            let to = r.to.as_str();
            match r.kind.as_str() {
                "resolve" => self.composition(from_type, to, Injection::Single),
                "call" | "construct" | "access" | "type-reference" => self.direct(id, to, &r.kind),
                _ => {}
            }
        }
    }

    /// A declared dependency: a collection parameter asks for every
    /// registration of its type argument (P-6); anything else for one.
    fn dependency(&mut self, from_type: &'a str, declared: &'a str, type_arguments: &'a [String]) {
        let collection = COLLECTION_IDS.contains(&declared) && !self.resolver.is_registered(declared);
        match (collection, type_arguments.first()) {
            (true, Some(t)) => self.composition(from_type, t.as_str(), Injection::Collection),
            _ => self.composition(from_type, declared, Injection::Single),
        }
    }

    /// A direct edge inside the solution: followed unless it is the
    /// registration call itself (`AddScoped<IFoo, Foo>()` does not use `Foo`).
    fn direct(&mut self, site: &'a str, to: &'a str, kind: &str) {
        let target_type = self.ix.members.get(to).map(|t| t.declaring_type.as_str()).unwrap_or(to);
        if !self.ix.types.contains_key(target_type) || self.is_registration_itself(site, to, target_type, kind) {
            return;
        }
        if self.ix.abstract_types.contains(target_type) && !self.ix.members.contains_key(to) {
            self.c.types.insert(target_type); // referenced, not composed
            return;
        }
        self.queue.push_back(to);
    }

    fn is_registration_itself(&self, site: &str, to: &str, target_type: &str, kind: &str) -> bool {
        match kind {
            "type-reference" => self.resolver.mentions(site, target_type),
            "construct" => self.resolver.constructs(site, target_type),
            "call" => to.contains(".#ctor(") && self.resolver.constructs(site, target_type),
            _ => false,
        }
    }

    /// A composition edge: classify, then resolve where the role says to.
    fn composition(&mut self, from_type: &'a str, target: &'a str, injection: Injection) {
        if self.c.edges.iter().any(|e| e.from == from_type && e.target == target && e.injection == injection) {
            return;
        }
        let registered = self.resolver.is_registered(target);
        let role = role_of(self.ix, target, registered);
        let sites = self.sites;
        // Order: a provider id is the criterion's partial row wherever it is
        // declared; a value is never supplied by a library; then the library
        // (or an unparsed call) may supply it; then the role; then resolution.
        let state = if role == Role::FactoryProvider && !registered {
            EdgeState::Partial
        } else if role == Role::Value {
            EdgeState::Excluded(role)
        } else if self.is_library_provided(target, registered) {
            match self.resolver.provider_of(target, &|s: &str| sites.reached(s)) {
                Some(call) => EdgeState::RegistrationNotRead(call),
                None => EdgeState::Boundary,
            }
        } else if !role.in_denominator() {
            EdgeState::Excluded(role)
        } else {
            match self.resolver.resolve(target, &|s: &str| sites.reached(s)) {
                Ok(impls) => {
                    for i in impls {
                        if let Some(impl_id) = self.ix.types.get(i.implementation.as_str()).map(|t| t.id.as_str()) {
                            self.c.types.insert(impl_id);
                            self.c.container_types.insert(impl_id);
                            self.queue.extend(self.ix.members_of.get(impl_id).into_iter().flatten().copied());
                        }
                    }
                    if role == Role::FactoryProvider { EdgeState::Partial } else { EdgeState::Resolved }
                }
                Err(_) if role == Role::FactoryProvider => EdgeState::Partial,
                Err(reason) => EdgeState::Unresolved(reason),
            }
        };
        if self.ix.types.contains_key(target) {
            self.c.types.insert(target);
        }
        self.c.edges.insert(CompositionEdge { from: from_type.to_string(), target: target.to_string(), role, state, injection });
    }

    /// Declared outside the solution, subclassed or implemented by no
    /// production type, named by no registration the resolver read — the
    /// library (or a call the resolver cannot parse) supplies it, so there is
    /// nothing in the solution for the container to choose (CG-R-68; shape
    /// is not the test, CG-R-75). Read before the role, so the boundary set
    /// is the whole used library surface; test projects do not count
    /// (CG-R-75).
    fn is_library_provided(&self, target: &str, registered: bool) -> bool {
        self.ix.external.contains_key(target) && !registered && self.ix.production_implementors(target).next().is_none()
    }
}

/// Types not reached that implement an abstraction some edge in `targets` landed on.
pub fn implementors_of<'a>(ix: &Index<'a>, c: &Closure<'a>, targets: &BTreeSet<&str>) -> BTreeSet<&'a str> {
    targets
        .iter()
        .flat_map(|t| ix.implementors.get(t).into_iter().flatten().copied())
        .filter(|t| !c.types.contains(t))
        .collect()
}

/// Composition edges by role, for reports.
pub fn by_role(c: &Closure<'_>) -> BTreeMap<String, usize> {
    let mut out = BTreeMap::new();
    for e in &c.edges {
        *out.entry(e.role.label().to_string()).or_insert(0) += 1;
    }
    out
}
