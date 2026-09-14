//! The denominator criterion — what an abstraction is *for* at a composition edge.
//!
//! Emil's criterion (2026-09-14): *an edge enters the denominator iff the
//! dependency must be satisfied by a chosen implementation at composition
//! time.* It is per edge; `csharp_walk` decides which edges are composition
//! edges (a constructor parameter of a container-constructed type, a
//! service-locator argument, an injected property). This module reads the
//! **role** of the target at such an edge through proxies that are declared
//! as proxies — each with the original predicate, its known divergence and
//! **the divergence's measured incidence on the field** (CG-R-77: a
//! divergence covering its field is a misclassification, not a proxy). Two
//! readings were retired on that ruling and are kept below with the
//! incidences that retired them: *data-contract* (181 of 181) and the
//! return-type reading of *factory-provider* (231 of 346).

use std::collections::BTreeSet;

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
    /// Composition chooses the provider; the provider chooses later — partial.
    FactoryProvider,
    /// No member to satisfy.
    Marker,
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
            Role::AbstractData => "abstract-data",
            Role::Value => "value",
        }
    }

    /// Does the edge enter the denominator at all?
    pub fn in_denominator(self) -> bool {
        matches!(self, Role::Service | Role::GenericDispatch | Role::FactoryProvider)
    }
}

/// One recognition rule, declared as the proxy it is, with the measured
/// incidence of its divergence (CG-R-77).
#[derive(Debug, Clone, Serialize)]
pub struct RoleProxy {
    pub role: Role,
    pub original_predicate: &'static str,
    pub proxy: &'static str,
    pub known_divergence: &'static str,
    /// *this misclassifies X, measured at n of m on this field, at this date* — or unmeasured.
    pub incidence: &'static str,
}

/// The proxies in force, printed on every report that uses them.
pub const PROXIES: &[RoleProxy] = &[
    RoleProxy { role: Role::Marker, original_predicate: "no member to satisfy", proxy: "an interface with no method, property or event anywhere in its interface chain", known_divergence: "an empty interface used as a DI key or a type-test target is excluded although the container may register it", incidence: "0 of 293 marker-role edges on A+B run 5 (2026-09-14); the pre-v4 reading (declared members only) misclassified 293 of 293 and was repaired (P-1)" },
    RoleProxy { role: Role::FactoryProvider, original_predicate: "composition chooses the provider; the provider chooses later", proxy: "Func<>, Lazy<>, IServiceProvider, IServiceScopeFactory, IServiceProviderFactory<> — the declared provider ids, nothing read from return types (P-5)", known_divergence: "a solution's own factory interface (an IClockFactory) reads as a service and is resolved to its registered implementation, which is what composition chose", incidence: "unmeasured against a ground truth (2026-09-14); on B run 5 the provider ids were 39 of 346 factory-provider edges, the remaining 307 services (231) and collection injection (76) — the retired reading below" },
    RoleProxy { role: Role::GenericDispatch, original_predicate: "the type parameter is the act", proxy: "an interface with generic arity above zero that is not a provider", known_divergence: "a generic service abstraction that is not dispatch (IRepository<T>) reads as dispatch; resolved by registration until a declaration supplies the edge (CG-R-61)", incidence: "48 of 48 generic-dispatch edges on A+B run 5 (2026-09-14) were generic services, none dispatch — no denominator effect, the label is informational" },
    RoleProxy { role: Role::AbstractData, original_predicate: "nothing to choose between", proxy: "an abstract class no registration names as a service", known_divergence: "an abstract class registered only through a wrapper the resolver does not read is excluded although it is a service", incidence: "0 of 9 abstract-data edges on A+B run 5 (2026-09-14); 8 were framework-provided and are read as boundary/registration-not-read before the role since v4" },
    RoleProxy { role: Role::Value, original_predicate: "a value, not an implementation", proxy: "a struct, enum, delegate, string or object", known_divergence: "a primitive the container does supply (an options-bound connection string) is excluded", incidence: "0 of 3 value edges on B run 5 (2026-09-14); A 0" },
];

/// Readings retired under CG-R-77, kept with the incidence that retired them.
pub const RETIRED_PROXIES: &[RoleProxy] = &[
    RoleProxy { role: Role::Service, original_predicate: "data-contract: a shape, not a dependency", proxy: "an interface whose chain declares properties only — retired, such an interface is a service (P-7)", known_divergence: "a service whose API is property-shaped (IOptions<T>.Value, IHttpContextAccessor.HttpContext) was excluded although composition chooses it", incidence: "181 of 181 data-contract edges on B run 5 (2026-09-14): the divergence was the field" },
    RoleProxy { role: Role::FactoryProvider, original_predicate: "factory-provider by return type: composition chooses the factory, the factory chooses later", proxy: "an interface with a method or property returning an abstraction — retired (P-5)", known_divergence: "a service that merely returns another service's result read as a factory", incidence: "231 of 346 factory-provider edges on B run 5 (2026-09-14) were services and 76 collection injection (now its own classification, P-6): the divergence was the field's majority" },
];

const FACTORY_IDS: &[&str] = &[
    "T:System.Func`1", "T:System.Func`2", "T:System.Lazy`1", "T:System.IServiceProvider",
    "T:Microsoft.Extensions.DependencyInjection.IServiceScopeFactory",
    "T:Microsoft.Extensions.DependencyInjection.IServiceProviderFactory`1",
];

/// The container's collection injection: a parameter of one of these types
/// asks for every registration of the type argument (P-6).
pub const COLLECTION_IDS: &[&str] = &[
    "T:System.Collections.Generic.IEnumerable`1", "T:System.Collections.Generic.IReadOnlyList`1",
    "T:System.Collections.Generic.IReadOnlyCollection`1", "T:System.Collections.Generic.IList`1",
    "T:System.Collections.Generic.ICollection`1",
];

/// The shape facts the proxies read, for a type in the solution or outside it.
struct Shape {
    kind: String,
    is_abstract: bool,
    arity: bool,
    methods: u32,
    properties: u32,
    events: u32,
}

/// The shape of one type alone, in the solution or outside it, with its base interfaces.
fn own_shape<'a>(ix: &Index<'a>, id: &str) -> Option<(Shape, Vec<&'a str>)> {
    if let Some(e) = ix.external.get(id) {
        let shape = Shape { kind: e.kind.clone(), is_abstract: e.is_abstract, arity: e.arity > 0, methods: e.methods, properties: e.properties, events: e.events };
        return Some((shape, e.interfaces.iter().map(String::as_str).collect()));
    }
    let t = ix.types.get(id)?;
    let members: Vec<_> = ix.members_of.get(id).into_iter().flatten().filter_map(|m| ix.members.get(m)).collect();
    let count = |k: &str| members.iter().filter(|m| m.kind == k).count() as u32;
    let shape = Shape { kind: t.kind.clone(), is_abstract: t.is_abstract, arity: id.contains('`'), methods: count("method"), properties: count("property"), events: count("event") };
    Some((shape, t.interfaces.iter().map(String::as_str).collect()))
}

/// The shape the proxies read: the type's own kind, with members summed over
/// its whole interface chain (P-1) — an interface that inherits its members
/// has members to satisfy.
fn shape_of(ix: &Index<'_>, id: &str) -> Option<Shape> {
    let (mut shape, bases) = own_shape(ix, id)?;
    let mut seen: BTreeSet<&str> = BTreeSet::new();
    let mut stack = bases;
    while let Some(base) = stack.pop() {
        if !seen.insert(base) {
            continue;
        }
        if let Some((b, more)) = own_shape(ix, base) {
            shape.methods += b.methods;
            shape.properties += b.properties;
            shape.events += b.events;
            stack.extend(more);
        }
    }
    Some(shape)
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
        "interface" if shape.methods == 0 && shape.properties == 0 && shape.events == 0 => Role::Marker,
        "interface" if shape.arity => Role::GenericDispatch,
        "interface" => Role::Service,
        _ if shape.is_abstract && !registered => Role::AbstractData,
        _ if matches!(target, "T:System.String" | "T:System.Object") => Role::Value,
        _ => Role::Service,
    }
}
