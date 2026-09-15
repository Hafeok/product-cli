//! Turning a verb's report into the bytes a terminal or a pipeline reads.

use serde_json::Value;

/// What a verb produced: a process exit code plus what to print.
pub struct Report {
    pub code: i32,
    pub text: String,
    pub json: Option<Value>,
}

impl Report {
    /// A report that prints text only.
    pub fn text(code: i32, text: impl Into<String>) -> Self {
        Self { code, text: text.into(), json: None }
    }

    /// Attach a machine-readable body, used when `--json` was asked for.
    pub fn with_json(mut self, json: Option<Value>) -> Self {
        self.json = json;
        self
    }
}

/// Print a report: the JSON body when one was asked for, the prose otherwise.
pub fn emit(report: &Report) {
    match &report.json {
        Some(value) => println!("{}", serde_json::to_string_pretty(value).unwrap_or_default()),
        None if report.text.is_empty() => {}
        None => println!("{}", report.text),
    }
}

/// The report a refused write prints.
///
/// Same class, same message the gate would report — the refusal *is* the gate,
/// not a second opinion about it.
pub fn refusal(
    subject: &str,
    verb: &str,
    findings: &[spec_core::check::Finding],
    as_json: bool,
) -> Report {
    let listed = findings.iter().map(ToString::to_string).collect::<Vec<_>>().join("\n  ");
    let text = format!("refusing to {verb} {subject} — the write would introduce:\n  {listed}");
    let body = as_json.then(|| {
        serde_json::json!({
            "subject": subject,
            "status": "refused",
            "findings": findings.iter().map(|f| serde_json::json!({
                "class": f.class.to_string(),
                "subject": f.record,
                "message": f.message,
            })).collect::<Vec<_>>(),
        })
    });
    Report::text(crate::exit::FINDINGS, text).with_json(body)
}
