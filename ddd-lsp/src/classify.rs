//! Snapshot diffing — before/after symbol facts into surface events.
//!
//! Language-neutral: facts extraction plus the policy table come in from
//! the adapter; this module only pairs symbols across the edit, names the
//! change, then lets the table decide. Symbols pair by
//! (container, kind, name); same-key overloads pair positionally after
//! sorting by signature, so an unchanged overload set yields no events.

use std::collections::BTreeMap;

use crate::adapter::{Adapter, AdapterFlags};
use crate::protocol::RawSymbol;
use crate::surface::{decide, ChangeKind, PolicyRow, SurfaceEvent, SymbolFacts};

/// Classify one edit: the adapter derives facts on both sides, the diff
/// names each change, the policy table rules on it.
pub fn classify_edit(
    adapter: &Adapter,
    flags: &AdapterFlags,
    before_text: &str,
    before_symbols: &[RawSymbol],
    after_text: &str,
    after_symbols: &[RawSymbol],
) -> Vec<SurfaceEvent> {
    let before = (adapter.facts)(before_text, before_symbols, flags);
    let after = (adapter.facts)(after_text, after_symbols, flags);
    let rows = (adapter.policy)(flags);
    diff_facts(&rows, adapter.visibility_rank, before, after)
}

/// Pair the two fact sets, emit one event per changed symbol.
pub fn diff_facts(
    rows: &[PolicyRow],
    visibility_rank: fn(&str) -> u8,
    before: Vec<SymbolFacts>,
    after: Vec<SymbolFacts>,
) -> Vec<SurfaceEvent> {
    let mut events = Vec::new();
    let mut before_by_key = group_by_key(before);
    for (key, mut afters) in group_by_key(after) {
        let mut befores = before_by_key.remove(&key).unwrap_or_default();
        afters.sort_by(|a, b| a.signature.cmp(&b.signature));
        befores.sort_by(|a, b| a.signature.cmp(&b.signature));
        pair_off(rows, visibility_rank, befores, afters, &mut events);
    }
    for (_, befores) in before_by_key {
        for facts in befores {
            push_event(rows, ChangeKind::Removed, facts.visibility.clone(), facts, None, &mut events);
        }
    }
    events
}

fn group_by_key(
    list: Vec<SymbolFacts>,
) -> BTreeMap<(String, String, String), Vec<SymbolFacts>> {
    let mut map: BTreeMap<_, Vec<SymbolFacts>> = BTreeMap::new();
    for facts in list {
        map.entry(facts.key()).or_default().push(facts);
    }
    map
}

/// Same-key symbols: pair positionally, surplus is added/removed.
fn pair_off(
    rows: &[PolicyRow],
    visibility_rank: fn(&str) -> u8,
    befores: Vec<SymbolFacts>,
    afters: Vec<SymbolFacts>,
    events: &mut Vec<SurfaceEvent>,
) {
    let paired = befores.len().min(afters.len());
    let mut afters = afters;
    let surplus_after = afters.split_off(paired);
    for (b, a) in befores.iter().take(paired).zip(afters) {
        if let Some(change) = change_of(b, &a) {
            let vis = exposed_visibility(visibility_rank, b, &a);
            push_event(rows, change, vis, a, Some(b.clone()), events);
        }
    }
    for facts in surplus_after {
        push_event(rows, ChangeKind::Added, facts.visibility.clone(), facts, None, events);
    }
    for facts in befores.into_iter().skip(paired) {
        push_event(rows, ChangeKind::Removed, facts.visibility.clone(), facts, None, events);
    }
}

/// Name the change between two paired states; `None` means untouched.
fn change_of(before: &SymbolFacts, after: &SymbolFacts) -> Option<ChangeKind> {
    if before.signature != after.signature {
        Some(ChangeKind::SignatureChanged)
    } else if before.decorators != after.decorators {
        Some(ChangeKind::DecoratorChanged)
    } else if before.visibility != after.visibility {
        Some(ChangeKind::VisibilityChanged)
    } else {
        None
    }
}

/// The visibility a paired change is judged at: the more exposed of the
/// two sides — promoting to public, demoting from public, or changing a
/// signature while demoting are all boundary events.
fn exposed_visibility(rank: fn(&str) -> u8, before: &SymbolFacts, after: &SymbolFacts) -> String {
    if rank(&before.visibility) > rank(&after.visibility) {
        before.visibility.clone()
    } else {
        after.visibility.clone()
    }
}

fn push_event(
    rows: &[PolicyRow],
    change: ChangeKind,
    judged_visibility: String,
    facts: SymbolFacts,
    before: Option<SymbolFacts>,
    events: &mut Vec<SurfaceEvent>,
) {
    let (surface, rule, rule_claim) = decide(rows, change, &facts, &judged_visibility);
    events.push(SurfaceEvent { change, facts, before, surface, rule, rule_claim });
}

#[cfg(test)]
#[path = "classify_tests.rs"]
mod tests;
