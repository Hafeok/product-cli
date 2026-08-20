//! The structural rows: module axes, lexical synonyms.
//!
//! Neither is declared. Module containment is *structural* — the server
//! reports which project a declaration sits in, which is a fact about the
//! codebase's layout rather than about the domain, so it proposes an axis at
//! mid assurance. Synonyms are *heuristic*: a word split over an identifier,
//! by rule, with no model in the path.

use super::derivation::Derivation;
use super::facts::CorpusFacts;
use super::mint::{class_iri, module_iri};
use super::propose::{cite, Proposer};
use super::rows::{row, Outcome};
use super::triple::{ProposedAssertion, Term, Triple};
use super::vocab::{reg, RDFS_LABEL, RDF_TYPE};

use super::declared::{gate, settle};

/// Row `modules`: a declaration's project becomes an axis it belongs to.
pub fn modules(base: &str, facts: &CorpusFacts, p: &Proposer) -> (Vec<ProposedAssertion>, Outcome) {
    let Some(r) = row("modules") else { return (Vec::new(), Outcome::FoundNothing) };
    if let Some(blocked) = gate(facts, r) {
        return (Vec::new(), blocked);
    }
    let mut out = Vec::new();
    for decl in &facts.declarations {
        if decl.module.is_empty() {
            continue;
        }
        let axis = module_iri(base, &decl.module);
        let evidence = vec![cite(&decl.file, decl.start_line), decl.module.clone()];
        let d = Derivation::by("module-containment", r.operations)
            .in_container(decl.kind, &decl.file);
        out.push(p.propose(
            r,
            Triple::new(axis.clone(), RDF_TYPE, Term::iri(reg::axis_class(base))),
            evidence.clone(),
            d.clone(),
        ));
        out.push(p.propose(
            r,
            Triple::new(axis.clone(), RDFS_LABEL, Term::plain(decl.module.clone())),
            evidence.clone(),
            d.clone(),
        ));
        out.push(p.propose(
            r,
            Triple::new(class_iri(base, &decl.name), reg::in_module(base), Term::iri(axis)),
            evidence,
            d,
        ));
    }
    settle(out)
}

/// Row `synonyms`: the spaced reading of a compound identifier.
///
/// One lexical rule, applied only where it says something: a single-word
/// identifier has no spaced form to offer, and proposing the name back as
/// its own synonym would be noise a reviewer has to reject.
pub fn synonyms(base: &str, facts: &CorpusFacts, p: &Proposer) -> (Vec<ProposedAssertion>, Outcome) {
    let Some(r) = row("synonyms") else { return (Vec::new(), Outcome::FoundNothing) };
    if let Some(blocked) = gate(facts, r) {
        return (Vec::new(), blocked);
    }
    let mut out = Vec::new();
    for decl in &facts.declarations {
        let Some(spaced) = word_split(&decl.name) else { continue };
        out.push(p.propose(
            r,
            Triple::new(
                class_iri(base, &decl.name),
                super::vocab::SKOS_ALT_LABEL,
                Term::plain(spaced),
            ),
            vec![cite(&decl.file, decl.start_line), format!("identifier {}", decl.name)],
            Derivation::by("identifier-word-split", r.operations)
                .in_container(decl.kind, &decl.file),
        ));
    }
    settle(out)
}

/// Split a compound identifier into words, or `None` when there is nothing
/// to split.
///
/// Handles the three casings that reach the extractor: camel and Pascal
/// boundaries, an acronym run followed by a word (`SQLTable` → `SQL Table`),
/// and explicit separators. A leading interface `I` is a naming convention,
/// not a word, so it is dropped when a capital follows it.
pub fn word_split(identifier: &str) -> Option<String> {
    let stripped = strip_interface_prefix(identifier);
    let chars: Vec<char> = stripped.chars().collect();
    let mut words: Vec<String> = Vec::new();
    let mut current = String::new();
    for (i, ch) in chars.iter().enumerate() {
        if *ch == '_' || *ch == '-' {
            push_word(&mut words, &mut current);
            continue;
        }
        let starts_word = ch.is_uppercase()
            && i > 0
            && (chars[i - 1].is_lowercase()
                || chars[i - 1].is_ascii_digit()
                || chars.get(i + 1).is_some_and(|n| n.is_lowercase()));
        if starts_word {
            push_word(&mut words, &mut current);
        }
        current.push(*ch);
    }
    push_word(&mut words, &mut current);
    (words.len() > 1).then(|| words.join(" "))
}

fn push_word(words: &mut Vec<String>, current: &mut String) {
    if !current.is_empty() {
        words.push(std::mem::take(current));
    }
}

/// Drop a C#/Java interface `I` prefix when a capital follows it.
fn strip_interface_prefix(identifier: &str) -> &str {
    let mut chars = identifier.chars();
    match (chars.next(), chars.next()) {
        (Some('I'), Some(second)) if second.is_uppercase() => &identifier[1..],
        _ => identifier,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_the_casings_that_reach_the_extractor() {
        assert_eq!(word_split("ChargingProfileSettings").as_deref(), Some("Charging Profile Settings"));
        assert_eq!(word_split("customerId").as_deref(), Some("customer Id"));
        assert_eq!(word_split("SQLTable").as_deref(), Some("SQL Table"));
        assert_eq!(word_split("upsert_profile").as_deref(), Some("upsert profile"));
    }

    #[test]
    fn an_interface_prefix_is_a_convention_not_a_word() {
        assert_eq!(word_split("IStringIdentity").as_deref(), Some("String Identity"));
        // A real word starting with I is not a prefix.
        assert_eq!(word_split("Invoice"), None);
        assert_eq!(word_split("InvoiceLine").as_deref(), Some("Invoice Line"));
    }

    /// Proposing a single-word name back as its own synonym is noise the
    /// reviewer would have to reject, so the rule declines to fire.
    #[test]
    fn a_single_word_identifier_yields_no_synonym() {
        assert_eq!(word_split("Location"), None);
        assert_eq!(word_split("ICommand"), None, "the prefix leaves one word behind");
        assert_eq!(word_split(""), None);
        assert_eq!(word_split("X"), None);
    }
}
