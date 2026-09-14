//! DI resolution over registration facts — the one graph (CG-R-60), with unresolved as a category (CG-R-62).
//!
//! In a solution built with DI the registrations are the call graph. This
//! module reads the reader's `registrations` (every call on an
//! `IServiceCollection`, as written) and answers, for a service type, which
//! implementation types the container would supply — or **why it cannot
//! say**. The idioms it understands are listed in [`Resolver::build`]; each
//! is a documented behaviour of Microsoft.Extensions.DependencyInjection,
//! not a guess about the solution. Of CG-R-62's six reasons, `open-generic`
//! is not distinguished (2026-09-13): an open-generic `typeof` pair resolves,
//! and one this resolver cannot read falls under `factory` or
//! `no-registration`. `factory` is a seventh reason, added because an opaque
//! factory lambda is neither scanning nor conditional.

use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;

use super::csharp_inventory::{Inventory, Registration};
pub use super::csharp_di_knowledge::REGISTRATION_KNOWLEDGE;

/// How a resolution was read off a registration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Via {
    /// `Add*<TService, TImpl>()`.
    Generic,
    /// `Add*(typeof(TService), typeof(TImpl))` — closed or open generic.
    TypeofPair,
    /// `Add*<TService>(new TImpl(…))`.
    Instance,
    /// `Add*<TService>(sp => new TImpl(…))` — the lambda constructs one type.
    FactoryConstruct,
    /// `Add*<T>()` — the service is its own implementation.
    SelfRegistration,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Resolved {
    pub service: String,
    pub implementation: String,
    pub via: Via,
    pub site: String,
}

/// Why an interface-mediated edge could not be followed (CG-R-62).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Reason {
    AssemblyScanning,
    KeyedService,
    DecoratorChain,
    ConditionalRegistration,
    ModuleConfiguration,
    Factory,
    NoRegistration,
}

impl Reason {
    pub fn label(self) -> &'static str {
        match self {
            Reason::AssemblyScanning => "assembly-scanning",
            Reason::KeyedService => "keyed-service",
            Reason::DecoratorChain => "decorator-chain",
            Reason::ConditionalRegistration => "conditional-registration",
            Reason::ModuleConfiguration => "module-configuration",
            Reason::Factory => "factory",
            Reason::NoRegistration => "no-registration",
        }
    }
}

#[derive(Debug, Clone)]
struct Entry {
    resolved: Option<Resolved>,
    conditional: bool,
    site: String,
}

/// One registration call as written, whatever the resolver made of it.
#[derive(Debug, Clone, Serialize)]
pub struct Call {
    pub site: String,
    /// The resolved method id, up to its parameter list.
    pub method: String,
    pub name: String,
    /// Declared outside the solution (its body cannot be walked).
    pub external: bool,
    pub read: bool,
}

/// The registration facts, indexed by service type id.
#[derive(Debug, Default)]
pub struct Resolver {
    entries: BTreeMap<String, Vec<Entry>>,
    /// Site → every type a registration call there names as a type argument.
    mentions: BTreeMap<String, BTreeSet<String>>,
    /// Site → every type a registration call there constructs in its arguments.
    constructed: BTreeMap<String, BTreeSet<String>>,
    keyed: BTreeSet<String>,
    decorated: BTreeSet<String>,
    /// Calls that register by scanning an assembly (`Scan`, `AddMediatR`, …).
    pub scanning_sites: usize,
    /// Projects a scanning call names through a `typeof` marker — the
    /// assemblies it scans. A service is attributed to scanning only when
    /// one of its implementors lives in such a project.
    scanned_projects: BTreeSet<String>,
    /// Type id → project id, for that attribution.
    project_of: BTreeMap<String, String>,
    /// Interface/base → implementing/deriving type ids.
    implementors: BTreeMap<String, Vec<String>>,
    /// Calls on `IServiceCollection` the resolver read as registrations.
    pub registrations_read: usize,
    /// Calls on `IServiceCollection` it did not (AddControllers, Build…, …).
    pub calls_ignored: usize,
    /// Those ignored calls by method name — the map of what the resolver
    /// would have to learn next (a solution's own registration wrappers show
    /// up here).
    pub ignored_by_method: BTreeMap<String, usize>,
    /// Every call, read or not, for the registration-knowledge lookup
    /// (CG-R-75) and the map of unlearned calls.
    pub calls: Vec<Call>,
    /// Registration calls at sites in test projects, left unread (CG-R-75).
    pub test_sites_skipped: usize,
}

/// Lifetime registrations, plus the `ServiceDescriptor` forms (`Add`,
/// `TryAdd`, `TryAddEnumerable`, `Replace`) whose service/implementation pair
/// arrives as `typeof` arguments inside the descriptor — the shape source
/// generators such as Mediator emit.
const LIFETIME: &[&str] = &[
    "AddScoped", "AddTransient", "AddSingleton", "TryAddScoped", "TryAddTransient", "TryAddSingleton",
    "Add", "TryAdd", "TryAddEnumerable", "Replace", "AddHostedService", "AddDbContext", "AddDbContextPool",
    "AddCheck",
];
const SCANNING: &[&str] = &[
    "Scan", "AddMediatR", "RegisterServicesFromAssembly", "RegisterServicesFromAssemblies",
    "RegisterServicesFromAssemblyContaining", "AddValidatorsFromAssembly", "AddValidatorsFromAssemblyContaining",
    "AddAutoMapper", "AddClassesMatching",
];

impl Resolver {
    /// Read every registration fact once.
    pub fn build(inv: &Inventory) -> Resolver {
        let mut implementors: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for e in inv.references.iter().filter(|e| e.kind == "implement" || e.kind == "inherit") {
            implementors.entry(e.to.clone()).or_default().push(e.from.clone());
        }
        let mut r = Resolver {
            project_of: inv.types.iter().map(|t| (t.id.clone(), t.project.clone())).collect(),
            implementors,
            ..Default::default()
        };
        let test_projects: BTreeSet<&str> = inv.projects.iter().filter(|p| p.is_test()).map(|p| p.id.as_str()).collect();
        let declaring: BTreeMap<&str, &str> = inv.members.iter().map(|m| (m.id.as_str(), m.declaring_type.as_str())).collect();
        for reg in &inv.registrations {
            let in_test = declaring.get(reg.site.as_str()).and_then(|t| r.project_of.get(*t)).is_some_and(|p| test_projects.contains(p.as_str()));
            if in_test {
                r.test_sites_skipped += 1;
                continue;
            }
            let read = r.read(reg);
            let method = reg.method.split('(').next().unwrap_or(&reg.method).to_string();
            let declaring_type = method.rsplit_once('.').map(|(t, _)| t.replacen("M:", "T:", 1)).unwrap_or_default();
            r.calls.push(Call { site: reg.site.clone(), method, name: reg.method_name.clone(), external: !r.project_of.contains_key(&declaring_type), read });
        }
        r
    }

    /// Every implementation a registration at a reached, unconditional site
    /// names: the container constructs each of them, asked for or not (O-17,
    /// CG-R-68's operational form — the registration list decides).
    pub fn implementations_at(&self, site_reached: &dyn Fn(&str) -> bool) -> BTreeSet<&str> {
        self.entries
            .values()
            .flatten()
            .filter(|e| !e.conditional && site_reached(&e.site))
            .filter_map(|e| e.resolved.as_ref().map(|r| r.implementation.as_str()))
            .collect()
    }

    /// A reached registration call the resolver does not parse but the
    /// knowledge table says registers `type_id`: the call's name (CG-R-75).
    pub fn provider_of(&self, type_id: &str, site_reached: &dyn Fn(&str) -> bool) -> Option<&'static str> {
        self.calls.iter().filter(|c| site_reached(&c.site)).find_map(|c| {
            REGISTRATION_KNOWLEDGE.iter().find(|(m, _, types)| *m == c.method && types.contains(&type_id)).map(|(_, name, _)| *name)
        })
    }

    /// Reached external calls the resolver neither parsed nor knows — the
    /// map of what the registration reader still has to learn.
    pub fn unlearned(&self, site_reached: &dyn Fn(&str) -> bool) -> BTreeMap<String, usize> {
        let mut out = BTreeMap::new();
        for c in self.calls.iter().filter(|c| c.external && !c.read && site_reached(&c.site)) {
            if !REGISTRATION_KNOWLEDGE.iter().any(|(m, _, _)| *m == c.method) {
                *out.entry(c.method.clone()).or_insert(0) += 1;
            }
        }
        out
    }

    /// Ignored calls split by where the method lives: in the solution (its
    /// body is walked, nothing is lost) or outside it (a gap).
    pub fn ignored_split(&self) -> (BTreeMap<String, usize>, BTreeMap<String, usize>) {
        let (mut inside, mut outside) = (BTreeMap::new(), BTreeMap::new());
        for c in self.calls.iter().filter(|c| !c.read) {
            *if c.external { &mut outside } else { &mut inside }.entry(c.name.clone()).or_insert(0) += 1;
        }
        (inside, outside)
    }

    /// Does an implementor of `service` live in a project some scanning
    /// call names? Scanning registers the handlers it finds, so the scanned
    /// assembly is the implementors', not the interface's.
    fn scanned(&self, service: &str) -> bool {
        self.implementors
            .get(service)
            .into_iter()
            .flatten()
            .any(|t| self.project_of.get(t).is_some_and(|p| self.scanned_projects.contains(p)))
    }

    fn read(&mut self, reg: &Registration) -> bool {
        let mentioned = self.mentions.entry(reg.site.clone()).or_default();
        mentioned.extend(reg.type_arguments.iter().cloned());
        mentioned.extend(reg.typeof_arguments.iter().cloned());
        self.constructed.entry(reg.site.clone()).or_default().extend(reg.constructs.iter().cloned());
        let name = reg.method_name.as_str();
        if SCANNING.contains(&name) {
            self.scanning_sites += 1;
            self.registrations_read += 1;
            for marker in &reg.typeof_arguments {
                if let Some(p) = self.project_of.get(marker) {
                    self.scanned_projects.insert(p.clone());
                }
            }
            true
        } else if name.contains("Keyed") {
            if let Some(service) = reg.type_arguments.first().or(reg.typeof_arguments.first()) {
                self.keyed.insert(service.clone());
            }
            self.registrations_read += 1;
            true
        } else if name == "Decorate" {
            if let Some(service) = reg.type_arguments.first().or(reg.typeof_arguments.first()) {
                self.decorated.insert(service.clone());
            }
            self.registrations_read += 1;
            true
        } else if LIFETIME.contains(&name) {
            match Self::pair(reg) {
                Some((service, resolved)) => {
                    self.entries.entry(service).or_default().push(Entry { resolved, conditional: reg.conditional, site: reg.site.clone() });
                    self.registrations_read += 1;
                    true
                }
                None => self.ignore(name),
            }
        } else {
            self.ignore(name)
        }
    }

    fn ignore(&mut self, name: &str) -> bool {
        self.calls_ignored += 1;
        *self.ignored_by_method.entry(name.to_string()).or_insert(0) += 1;
        false
    }

    /// The (service, implementation) a lifetime registration states, if readable.
    fn pair(reg: &Registration) -> Option<(String, Option<Resolved>)> {
        let ta = &reg.type_arguments;
        let to = &reg.typeof_arguments;
        let mk = |service: &str, implementation: &str, via: Via| {
            Some(Resolved { service: service.to_string(), implementation: implementation.to_string(), via, site: reg.site.clone() })
        };
        // A constructed ServiceDescriptor is the registration's carrier, not an implementation.
        let constructs: Vec<&String> = reg
            .constructs
            .iter()
            .filter(|c| !c.ends_with(".ServiceDescriptor"))
            .collect();
        match (ta.len(), to.len()) {
            (2, _) => Some((ta[0].clone(), mk(&ta[0], &ta[1], Via::Generic))),
            (1, _) if !constructs.is_empty() => {
                let via = if reg.has_lambda { Via::FactoryConstruct } else { Via::Instance };
                Some((ta[0].clone(), mk(&ta[0], constructs[0], via)))
            }
            (1, _) if reg.has_lambda => Some((ta[0].clone(), None)),
            (1, _) => Some((ta[0].clone(), mk(&ta[0], &ta[0], Via::SelfRegistration))),
            (0, n) if n >= 2 => Some((to[0].clone(), mk(&to[0], &to[1], Via::TypeofPair))),
            (0, 1) if !constructs.is_empty() => Some((to[0].clone(), mk(&to[0], constructs[0], Via::Instance))),
            (0, 1) if reg.has_lambda => Some((to[0].clone(), None)),
            (0, 1) => Some((to[0].clone(), mk(&to[0], &to[0], Via::SelfRegistration))),
            _ => None,
        }
    }

    /// Does a registration at `site` name `type_id` as a type argument or
    /// `typeof`? Such a mention is the registration itself, not a use: the
    /// walk must not read `AddScoped<IFoo, Foo>()` as reaching `Foo` (CG-R-60
    /// — the edge exists only when the registration resolves).
    pub fn mentions(&self, site: &str, type_id: &str) -> bool {
        self.mentions.get(site).is_some_and(|m| m.contains(type_id))
    }

    /// Does a registration at `site` construct `type_id` in an instance or
    /// factory argument? That construction runs when the container resolves
    /// the service, not when the site runs.
    pub fn constructs(&self, site: &str, type_id: &str) -> bool {
        self.constructed.get(site).is_some_and(|m| m.contains(type_id))
    }

    /// Is `type_id` something the container would supply — registered at all?
    pub fn is_registered(&self, type_id: &str) -> bool {
        self.entries.contains_key(type_id) || self.keyed.contains(type_id) || self.decorated.contains(type_id)
    }

    /// The implementations the container supplies for `service`, given which
    /// registration sites the walk has reached — or the reason it cannot say.
    pub fn resolve(&self, service: &str, site_reached: &dyn Fn(&str) -> bool) -> Result<Vec<&Resolved>, Reason> {
        if self.keyed.contains(service) {
            return Err(Reason::KeyedService);
        }
        if self.decorated.contains(service) {
            return Err(Reason::DecoratorChain);
        }
        let Some(entries) = self.entries.get(service) else {
            return Err(if self.scanned(service) { Reason::AssemblyScanning } else { Reason::NoRegistration });
        };
        let live: Vec<&Resolved> = entries
            .iter()
            .filter(|e| !e.conditional && site_reached(&e.site))
            .filter_map(|e| e.resolved.as_ref())
            .collect();
        if !live.is_empty() {
            return Ok(live);
        }
        if entries.iter().any(|e| e.resolved.is_some() && !e.conditional) {
            Err(Reason::ModuleConfiguration)
        } else if entries.iter().any(|e| e.conditional) {
            Err(Reason::ConditionalRegistration)
        } else {
            Err(Reason::Factory)
        }
    }

    /// Every registered service, with its resolution count — for reports.
    pub fn services(&self) -> Vec<(&str, usize)> {
        self.entries.iter().map(|(s, e)| (s.as_str(), e.iter().filter(|x| x.resolved.is_some()).count())).collect()
    }
}

#[cfg(test)]
#[path = "csharp_di_tests.rs"]
mod tests;
