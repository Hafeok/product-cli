//! The structural verdicts over a store of act-time records.
//!
//! The class set is closed at four. A store fails for a schema fault plus
//! `S001`–`S004`, never for a fifth thing a caller thought of: adding one is
//! a change to `docs/spec-flow-act-record-v1.md`, not a patch here. None is
//! configurable, because a project that could switch `S001` off would have a
//! tool that reports what it was told to report.

use std::fmt;

use ledger_core::identity::Identity;

use crate::closure::Closure;
use crate::digest::closure_digest;
use crate::record::ActRecord;

/// A verdict class. Closed set; see the module doc.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Class {
    /// A record carries no closure — the write did not happen.
    S001,
    /// A closure's principal resolves to a model or a CI identity.
    S002,
    /// A closure's binding does not match the record it claims to close.
    S003,
    /// A closure's kind disagrees with the determinations it carries.
    S004,
}

impl fmt::Display for Class {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let code = match self {
            Self::S001 => "S001",
            Self::S002 => "S002",
            Self::S003 => "S003",
            Self::S004 => "S004",
        };
        f.write_str(code)
    }
}

/// One finding against one record.
#[derive(Debug, Clone, PartialEq)]
pub struct Finding {
    pub class: Class,
    pub record: String,
    pub message: String,
}

impl fmt::Display for Finding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {} — {}", self.class, self.record, self.message)
    }
}

/// Why this identity may not close a record, when it may not.
///
/// Delegates to the ledger's acceptor test rather than restating it: one
/// identity law, two gates. The test is a floor and not a proof — it catches
/// what a CI system or an agent harness produces by default, which is where
/// the failure actually occurs, and it cannot catch a model configured with
/// a human-looking address.
pub fn principal_refusal(principal: &Identity) -> Option<String> {
    principal.model_or_bot_reason()
}

/// Judge one record against the closed class set.
pub fn judge(record: &ActRecord) -> Vec<Finding> {
    let Some(closure) = &record.closure else {
        return vec![finding(
            Class::S001,
            record,
            "no closure — a slice can be built unattended, it cannot be closed unattended",
        )];
    };
    judge_closure(record, closure)
}

/// Judge every record in a store, in the order given.
pub fn judge_store(records: &[ActRecord]) -> Vec<Finding> {
    records.iter().flat_map(judge).collect()
}

fn judge_closure(record: &ActRecord, closure: &Closure) -> Vec<Finding> {
    let mut findings = Vec::new();
    if let Some(reason) = principal_refusal(&closure.principal) {
        findings.push(finding(Class::S002, record, &format!("closed by {reason}")));
    }
    let expected = closure_digest(&record.computed_binds(), closure);
    if closure.binds != expected {
        findings.push(finding(
            Class::S003,
            record,
            "the closure does not bind the opening it claims to close",
        ));
    }
    if !closure.kind_matches_payload() {
        findings.push(finding(
            Class::S004,
            record,
            &format!(
                "`{}` with {} determination(s)",
                closure.kind.as_str(),
                closure.determinations.len()
            ),
        ));
    }
    findings
}

fn finding(class: Class, record: &ActRecord, message: &str) -> Finding {
    Finding {
        class,
        record: record.id.clone(),
        message: message.to_string(),
    }
}

#[path = "check_tests.rs"]
#[cfg(test)]
mod tests;
