//! Report figures over the registration calls — the knowledge table's coverage of reached calls (CG-R-79), the ignored-call split, the unlearned map.

use std::collections::BTreeMap;

use serde::Serialize;

use super::csharp_di::Resolver;
use super::csharp_di_knowledge::REGISTRATION_KNOWLEDGE;

/// CG-R-79: of the external registration calls the walk reached, how many
/// the resolver parses, how many the knowledge table knows, how many neither.
#[derive(Debug, Clone, Default, Serialize)]
pub struct TableCoverage {
    pub reached: usize,
    pub parsed: usize,
    pub known: usize,
    pub unknown: usize,
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

impl Resolver {
    /// The registration-knowledge table's coverage over *reached* external
    /// calls (CG-R-79): the resolver parses some, the table knows some, the
    /// rest are unknown — the table's size is not the measure.
    pub fn table_coverage(&self, site_reached: &dyn Fn(&str) -> bool) -> TableCoverage {
        let mut c = TableCoverage::default();
        for call in self.calls.iter().filter(|c| c.external && site_reached(&c.site)) {
            c.reached += 1;
            if call.read {
                c.parsed += 1;
            } else if REGISTRATION_KNOWLEDGE.iter().any(|(m, _, _)| *m == call.method) {
                c.known += 1;
            } else {
                c.unknown += 1;
            }
        }
        c
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

}
