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
