//! The denominator criterion — what an abstraction is *for* at a composition edge.
//!
//! Emil's criterion (2026-09-14): *an edge enters the denominator iff the
//! dependency must be satisfied by a chosen implementation at composition
//! time.* It is per edge; `csharp_walk` decides which edges are composition
//! edges (a constructor parameter of a container-constructed type, or a
//! service-locator argument). This module reads the **role** of the target
//! at such an edge from its shape, through proxies that are declared as
//! proxies — each with the original predicate and its known divergence —
//! so that an undefined denominator is not replaced by a confidently wrong one.

use serde::Serialize;

use super::csharp_inventory::Index;

/// The roles the criterion yields. Only `Service` and `GenericDispatch`
/// enter the ratio; `FactoryProvider` is its own *partial* state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Role {
    /// An implementation is chosen at composition.
    Service,
    /// A generic contract whose type parameter is the act (CG-R-61).
    GenericDispatch,
    /// Composition chooses the factory; the factory chooses later — partial.
    FactoryProvider,
    /// No member to satisfy.
    Marker,
    /// A shape, not a dependency.
    DataContract,
    /// An abstract class no registration names — nothing to choose between (CG-R-64).
    AbstractData,
    /// A struct, enum, delegate or primitive: a value, not an implementation.
    Value,
}

impl Role {
    pub fn label(self) -> &'static str {
        match self {
            Role::Service => "service",
            Role::GenericDispatch => "generic-dispatch",
            Role::FactoryProvider => "factory-provider",
            Role::Marker => "marker",
            Role::DataContract => "data-contract",
            Role::AbstractData => "abstract-data",
            Role::Value => "value",
        }
    }

    /// Does the edge enter the denominator at all?
    pub fn in_denominator(self) -> bool {
        matches!(self, Role::Service | Role::GenericDispatch | Role::FactoryProvider)
    }
}

/// One recognition rule, declared as the proxy it is.
#[derive(Debug, Clone, Serialize)]
pub struct RoleProxy {
    pub role: Role,
    pub original_predicate: &'static str,
    pub proxy: &'static str,
    pub known_divergence: &'static str,
}

/// The proxies, printed on every report that uses them.
pub const PROXIES: &[RoleProxy] = &[
    RoleProxy { role: Role::Marker, original_predicate: "no member to satisfy", proxy: "an interface declaring no methods, properties or events", known_divergence: "an empty interface used as a DI key or a type-test target is excluded although the container may register it" },
    RoleProxy { role: Role::DataContract, original_predicate: "a shape, not a dependency", proxy: "an interface declaring properties only", known_divergence: "a service whose API is property-shaped (an options accessor, IOptions<T>.Value) is excluded although composition chooses it" },
    RoleProxy { role: Role::FactoryProvider, original_predicate: "composition chooses the factory; the factory chooses later", proxy: "Func<>, Lazy<>, IServiceProvider, IServiceScopeFactory, or an interface with a method or property returning an abstraction", known_divergence: "a service that merely returns another service's result (a repository returning an IReadOnlyList) reads as a factory" },
    RoleProxy { role: Role::GenericDispatch, original_predicate: "the type parameter is the act", proxy: "an interface with generic arity above zero that is not a factory", known_divergence: "a generic service abstraction that is not dispatch (IRepository<T>) reads as dispatch; resolved by registration until a declaration supplies the edge (CG-R-61)" },
    RoleProxy { role: Role::AbstractData, original_predicate: "nothing to choose between", proxy: "an abstract class no registration names as a service", known_divergence: "an abstract class registered only through a wrapper the resolver does not read is excluded although it is a service" },
    RoleProxy { role: Role::Value, original_predicate: "a value, not an implementation", proxy: "a struct, enum, delegate, string or object", known_divergence: "a primitive the container does supply (an options-bound connection string) is excluded" },
];

const FACTORY_IDS: &[&str] = &[
    "T:System.Func`1", "T:System.Func`2", "T:System.Lazy`1", "T:System.IServiceProvider",
    "T:Microsoft.Extensions.DependencyInjection.IServiceScopeFactory",
    "T:Microsoft.Extensions.DependencyInjection.IServiceProviderFactory`1",
];

/// The shape facts the proxies read, for a type in the solution or outside it.
struct Shape {
    kind: String,
    is_abstract: bool,
    arity: bool,
    methods: u32,
    properties: u32,
    events: u32,
    abstract_returns: bool,
}

fn shape_of(ix: &Index<'_>, id: &str) -> Option<Shape> {
    if let Some(e) = ix.external.get(id) {
        return Some(Shape { kind: e.kind.clone(), is_abstract: e.is_abstract, arity: e.arity > 0, methods: e.methods, properties: e.properties, events: e.events, abstract_returns: !e.abstract_returns.is_empty() });
    }
    let t = ix.types.get(id)?;
    let members: Vec<_> = ix.members_of.get(id).into_iter().flatten().filter_map(|m| ix.members.get(m)).collect();
    let count = |k: &str| members.iter().filter(|m| m.kind == k).count() as u32;
    let abstract_returns = members.iter().any(|m| m.return_type.as_deref().is_some_and(|r| ix.abstract_types.contains(r)));
    Some(Shape { kind: t.kind.clone(), is_abstract: t.is_abstract, arity: id.contains('`'), methods: count("method"), properties: count("property"), events: count("event"), abstract_returns })
}

/// The role of `target` at a composition edge. `registered` says whether any
/// registration names it as a service.
pub fn role_of(ix: &Index<'_>, target: &str, registered: bool) -> Role {
    if FACTORY_IDS.contains(&target) {
        return Role::FactoryProvider;
    }
    let Some(shape) = shape_of(ix, target) else { return Role::Service };
    match shape.kind.as_str() {
        "struct" | "record-struct" | "enum" | "delegate" => Role::Value,
        "interface" => {
            if shape.methods == 0 && shape.properties == 0 && shape.events == 0 {
                Role::Marker
            } else if shape.methods == 0 && shape.events == 0 {
                Role::DataContract
            } else if shape.abstract_returns {
                Role::FactoryProvider
            } else if shape.arity {
                Role::GenericDispatch
            } else {
                Role::Service
            }
        }
        _ if shape.is_abstract && !registered => Role::AbstractData,
        _ if matches!(target, "T:System.String" | "T:System.Object") => Role::Value,
        _ => Role::Service,
    }
}
