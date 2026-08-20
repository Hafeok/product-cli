//! Rendering an extraction run for a human — rows, grades, the fixpoint.
//!
//! The report's job is to make an absence legible. A row that found nothing
//! reads differently from one gated off because an operation did not answer,
//! differently again from one that is not derivable from the declaration
//! surface at all. Collapsing those three into "0" is the failure this whole
//! shape exists to prevent.

use super::plan::ExtractionPlan;

/// The full text report of one run.
pub fn render(plan: &ExtractionPlan) -> String {
    let mut out = format!(
        "extraction — {} at {}\n  {} assertion(s) proposed across {} file(s)\n",
        plan.corpus,
        plan.git_ref,
        plan.assertions.len(),
        plan.files.len()
    );
    if plan.already_ratified > 0 {
        out.push_str(&format!(
            "  {} assertion(s) already ratified in the canonical graph — not re-proposed\n",
            plan.already_ratified
        ));
    }
    out.push_str(&grades(plan));
    out.push_str(&batches(plan));
    out.push_str(&evidence(plan));
    out.push_str(&rows(plan));
    out.push_str(&fixpoint(plan));
    out.push_str(&notes(plan));
    out.push_str(&shacl(plan));
    out
}

/// Counts per `(derivation confidence, domain relevance)` pair — never per
/// mark alone. A count per confidence is the report that let 31 top-grade
/// assertions read as safe to ratify.
fn grades(plan: &ExtractionPlan) -> String {
    let mut out = String::from("\nby derivation confidence / domain relevance:\n");
    for (pair, count) in plan.by_pair() {
        out.push_str(&format!("  {pair:<22} {count}\n"));
    }
    out
}

/// The reviewer's actual working unit at volume: batches, keyed by the pair
/// with the derivation signature, in the order two runs agree on.
fn batches(plan: &ExtractionPlan) -> String {
    let mut out = format!(
        "\nreview batches ({} batch(es) over {} assertion(s)):\n",
        plan.batches.len(),
        plan.assertions.len()
    );
    for batch in &plan.batches {
        out.push_str(&format!("  {:>6}  {}\n", batch.count, batch.label()));
        for line in batch.sample.iter().take(2) {
            out.push_str(&format!("          · {line}\n"));
        }
    }
    out
}

/// What the run wrote to the evidence layer, which is not the same number as
/// what it proposed: evidence covers every assertion the instrument read,
/// including the ones already ratified.
fn evidence(plan: &ExtractionPlan) -> String {
    format!(
        "\nrun evidence: {} derivation record(s) — {} proposed, {} already ratified \
and re-read for their derivation\n",
        plan.sidecar_entries,
        plan.assertions.len(),
        plan.already_ratified
    )
}

fn rows(plan: &ExtractionPlan) -> String {
    let mut out = String::from("\nby §5.2 row:\n");
    for result in &plan.rows {
        out.push_str(&format!(
            "  {:<16} {:<6} {}\n",
            result.row.id,
            result.row.confidence.as_str(),
            result.outcome.describe()
        ));
    }
    out
}

fn fixpoint(plan: &ExtractionPlan) -> String {
    let f = &plan.fixpoint;
    let mut out = format!(
        "\nfixpoint: {} round(s), {} rule parse(s), {} execution(s), {} ms\n  \
         seed {} triples · {} proposed · {} entailed (recomputed at projection build, never stored)\n",
        f.rounds,
        f.parses,
        f.executions,
        f.elapsed_ms,
        f.seed_triples,
        f.proposed(),
        f.entailed()
    );
    for rule in &f.rules {
        out.push_str(&format!(
            "  {:<20} {:>6} triple(s) over {} productive round(s){}\n",
            rule.id,
            rule.triples,
            rule.productive_rounds,
            if rule.proposed { "" } else { "  [entailed, not proposed]" }
        ));
    }
    out
}

fn notes(plan: &ExtractionPlan) -> String {
    let n = &plan.notes;
    let mut out = String::from("\nwhat the corpus did that the table did not anticipate:\n");
    let outside: Vec<String> = n.supertypes_outside_subset.iter().cloned().collect();
    out.push_str(&format!(
        "  {} supertype(s) named by an edge but declared outside the loaded subset{}\n",
        outside.len(),
        sample(&outside)
    ));
    out.push_str(&format!(
        "  {} propert(ies) carry no range — neither a declared type nor a mapped primitive{}\n",
        n.unranged_properties.len(),
        sample(&n.unranged_properties)
    ));
    out.push_str(&format!(
        "  {} propert(ies) declared a nullability marker, read and deliberately unmapped\n",
        n.nullable_properties
    ));
    out.push_str(&format!(
        "  {} propert(ies) derived from a member the server reported as a field — \
         implementation the row cannot tell from domain structure{}\n",
        n.field_derived.len(),
        sample(&n.field_derived)
    ));
    out
}

/// A bounded sample of a list, so a report names what it counts without
/// becoming the list itself.
fn sample(items: &[String]) -> String {
    const SHOWN: usize = 6;
    if items.is_empty() {
        return String::new();
    }
    let head: Vec<&str> = items.iter().take(SHOWN).map(String::as_str).collect();
    let more = items.len().saturating_sub(head.len());
    let tail = if more > 0 { format!(", … {more} more") } else { String::new() };
    format!(" — {}{}", head.join(", "), tail)
}

fn shacl(plan: &ExtractionPlan) -> String {
    if plan.conformant() {
        return format!(
            "\nshapes: conformant — {} constraint(s) evaluated against the instance's own shapes\n",
            plan.constraints_evaluated
        );
    }
    let mut out = format!("\nshapes: {} finding(s) — nothing may be written:\n", plan.shacl.len());
    for finding in &plan.shacl {
        out.push_str(&format!("  {} — {}\n", finding.focus, finding.message));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ground::entail::FixpointReport;
    use crate::ground::plan::RowResult;
    use crate::ground::rows::{Outcome, ROWS};

    fn bare_plan(rows: Vec<RowResult>) -> ExtractionPlan {
        ExtractionPlan {
            corpus: "corpus-backend".into(),
            git_ref: "deadbeefcafe".into(),
            assertions: Vec::new(),
            rows,
            fixpoint: FixpointReport {
                rounds: 0,
                parses: 0,
                executions: 0,
                elapsed_ms: 0,
                seed_triples: 0,
                rules: Vec::new(),
            },
            files: Vec::new(),
            shacl: Vec::new(),
            constraints_evaluated: 12,
            already_ratified: 0,
            sidecar_entries: 0,
            batches: Vec::new(),
            notes: Default::default(),
        }
    }

    /// The point of the report: three kinds of zero, three different lines.
    #[test]
    fn the_three_kinds_of_absence_read_differently() {
        let rows = vec![
            RowResult { row: ROWS[0], outcome: Outcome::FoundNothing },
            RowResult {
                row: ROWS[1],
                outcome: Outcome::Unavailable { operations: vec!["typeHierarchy".into()] },
            },
            RowResult { row: ROWS[3], outcome: Outcome::NotDerivable { reason: "…" } },
        ];
        let text = render(&bare_plan(rows));
        assert!(text.contains("found nothing"), "{text}");
        assert!(text.contains("UNAVAILABLE — typeHierarchy did not answer"), "{text}");
        assert!(text.contains("not derivable from the declaration subset"), "{text}");
    }

    #[test]
    fn a_conformant_plan_reports_its_constraint_count() {
        let text = render(&bare_plan(Vec::new()));
        assert!(text.contains("conformant — 12 constraint(s) evaluated"), "{text}");
    }
}
