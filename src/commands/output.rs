//! Output seam — handlers return an `Output` value; the dispatcher renders it.
//!
//! This separates business logic from presentation (SOLID/OCP). A handler that
//! returns `Output::Both { text, json }` supports `--format text` and
//! `--format json` without branching inside the handler.
//!
//! Legacy handlers that still call `println!` directly should return
//! `Output::Empty` — the renderer then prints nothing, preserving behaviour.

use product_lib::error::ProductError;
use std::io::{self, Write};

use super::BoxResult;

/// Bridge: render a migrated handler's `CmdResult` to stdout, converting the
/// typed error into the boxed error that `dispatch()` returns.
///
/// New handlers return `CmdResult` and are routed through this helper.
/// Legacy handlers return `BoxResult` and print directly — they will be
/// migrated incrementally.
pub(crate) fn render_result(result: CmdResult, format: &str) -> BoxResult {
    match result {
        Ok(out) => {
            render_stdout(out, format)?;
            Ok(())
        }
        Err(e) => Err(Box::new(e)),
    }
}

/// Result type for migrated handlers. Handlers return a renderable value
/// rather than printing directly. The dispatcher converts the value to bytes
/// on stdout according to the `--format` flag.
pub type CmdResult = Result<Output, ProductError>;

/// Renderable value produced by a command handler.
///
/// Variants let handlers express as much or as little as they want about
/// format-awareness:
/// - `Empty` — handler already printed (legacy) or produces no output.
/// - `Text` — handler only supports text rendering.
/// - `Json` — handler only produces structured data.
/// - `Both` — handler pre-computed both; dispatcher picks per `--format`.
#[allow(dead_code)] // Empty/Json/Both are consumed by subsequent slice migrations
pub enum Output {
    Empty,
    Text(String),
    Json(serde_json::Value),
    Both {
        text: String,
        json: serde_json::Value,
    },
}

impl Output {
    pub fn text(s: impl Into<String>) -> Self {
        Output::Text(s.into())
    }

    #[allow(dead_code)]
    pub fn json(v: serde_json::Value) -> Self {
        Output::Json(v)
    }

    #[allow(dead_code)]
    pub fn both(text: impl Into<String>, json: serde_json::Value) -> Self {
        Output::Both {
            text: text.into(),
            json,
        }
    }
}

/// Render an `Output` to the given writer based on the format flag.
///
/// `format` is the value of the global `--format` flag ("text" or "json").
/// Unknown formats fall through to text rendering.
pub fn render<W: Write>(out: Output, format: &str, w: &mut W) -> io::Result<()> {
    match (out, format) {
        (Output::Empty, _) => Ok(()),
        (Output::Text(s), _) => writeln!(w, "{}", s.trim_end_matches('\n')),
        (Output::Json(v), "json") => writeln!(w, "{}", v),
        (Output::Json(v), _) => writeln!(w, "{}", v),
        (Output::Both { json, .. }, "json") => writeln!(w, "{}", json),
        (Output::Both { text, .. }, _) => writeln!(w, "{}", text.trim_end_matches('\n')),
    }
}

/// Convenience: render to stdout.
pub fn render_stdout(out: Output, format: &str) -> io::Result<()> {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    render(out, format, &mut handle)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_writes_nothing() {
        let mut buf = Vec::new();
        render(Output::Empty, "text", &mut buf).expect("render");
        assert!(buf.is_empty());
    }

    #[test]
    fn text_writes_line() {
        let mut buf = Vec::new();
        render(Output::text("hello"), "text", &mut buf).expect("render");
        assert_eq!(buf, b"hello\n");
    }

    #[test]
    fn both_picks_json_when_format_is_json() {
        let mut buf = Vec::new();
        render(
            Output::both("TXT", serde_json::json!({"k": 1})),
            "json",
            &mut buf,
        )
        .expect("render");
        assert_eq!(buf, b"{\"k\":1}\n");
    }

    #[test]
    fn both_picks_text_when_format_is_text() {
        let mut buf = Vec::new();
        render(
            Output::both("TXT", serde_json::json!({"k": 1})),
            "text",
            &mut buf,
        )
        .expect("render");
        assert_eq!(buf, b"TXT\n");
    }

    #[test]
    fn json_unknown_format_still_renders_json() {
        let mut buf = Vec::new();
        render(Output::json(serde_json::json!(42)), "garbage", &mut buf).expect("render");
        assert_eq!(buf, b"42\n");
    }

    #[test]
    fn text_strips_trailing_newlines() {
        let mut buf = Vec::new();
        render(Output::text("hello\n\n"), "text", &mut buf).expect("render");
        assert_eq!(buf, b"hello\n");
    }
}
