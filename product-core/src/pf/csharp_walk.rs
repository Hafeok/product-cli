//! The walk — one graph, composition edges resolved through the container.
//!
//! From the root symbols, `call`, `construct`, `access` and `type-reference`
//! edges are followed directly inside the solution. A **composition edge** —
//! a constructor parameter of a type the container constructs, or a
//! service-locator argument — is where the container chooses an
//! implementation: its target's role is read from [`csharp_roles`], roles
//! outside the denominator are recorded as excluded, factory/provider
//! targets are recorded as *partial*, and the rest are resolved through
//! [`Resolver`] or recorded unresolved with the reason (CG-R-62). Registrations
//! count once the member holding them is reached, so the walk repeats to a
//! fixpoint.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use serde::Serialize;

use super::csharp_di::{Reason, Resolver};
use super::csharp_inventory::Index;
use super::csharp_roles::{role_of, Role};

/// What happened at one composition edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum EdgeState {
    Resolved,
    Partial,
    Unresolved(Reason),
    Excluded(Role),
}

/// One composition edge, from the referencing type to the abstraction.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct CompositionEdge {
    pub from: String,
    pub target: String,
    pub role: Role,
    pub state: EdgeState,
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
    let mut previous: BTreeSet<&'a str> = BTreeSet::new();
    for _ in 0..8 {
        let c = walk(ix, resolver, start, &previous);
        if c.members == previous {
            return c;
        }
        previous = c.members;
    }
    walk(ix, resolver, start, &previous)
}

struct Walk<'w, 'a> {
    ix: &'w Index<'a>,
    resolver: &'w Resolver,
    sites: &'w BTreeSet<&'a str>,
    queue: VecDeque<&'a str>,
    c: Closure<'a>,
}

fn walk<'a>(ix: &Index<'a>, resolver: &Resolver, start: &BTreeSet<&'a str>, sites: &BTreeSet<&'a str>) -> Closure<'a> {
    let mut w = Walk { ix, resolver, sites, queue: start.iter().copied().collect(), c: Closure::default() };
    for id in start {
        let type_id = ix.members.get(id).map(|m| m.declaring_type.as_str()).unwrap_or(id);
        w.c.container_types.insert(type_id);
        w.queue.extend(ix.members_of.get(id).into_iter().flatten().copied());
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
        let is_ctor = m.kind == "constructor" && self.c.container_types.contains(from_type);
        for r in self.ix.out.get(id).into_iter().flatten() {
            let to = r.to.as_str();
            match r.kind.as_str() {
                "resolve" => self.composition(from_type, to),
                "parameter" if is_ctor => self.composition(from_type, to),
                "call" | "construct" | "access" | "type-reference" => self.direct(id, to, &r.kind),
                _ => {}
            }
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
    fn composition(&mut self, from_type: &'a str, target: &'a str) {
        if self.c.edges.iter().any(|e| e.from == from_type && e.target == target) {
            return;
        }
        let registered = self.resolver.is_registered(target);
        let role = role_of(self.ix, target, registered);
        let state = if !role.in_denominator() {
            EdgeState::Excluded(role)
        } else {
            let sites = self.sites;
            match self.resolver.resolve(target, &|s: &str| sites.contains(s)) {
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
        self.c.edges.insert(CompositionEdge { from: from_type.to_string(), target: target.to_string(), role, state });
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
