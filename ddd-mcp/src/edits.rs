//! Edit-payload resolution — full text or LSP-style range edits.
//!
//! `ddd_apply_edit` accepts either the whole new file (`new_text`, read
//! through `raw_str` so a trailing newline survives — `DDD-arch-07`) or a
//! list of range edits applied last-to-first against the current content.

use std::path::Path;

use serde_json::Value;

use crate::state::raw_str;

pub fn read_or_empty(file: &Path) -> Result<String, String> {
    if file.exists() {
        std::fs::read_to_string(file).map_err(|e| format!("{}: {e}", file.display()))
    } else {
        Ok(String::new())
    }
}

pub fn resolve_new_text(args: &Value, old_text: &str) -> Result<String, String> {
    // `raw_str`: this value is a whole file, so trimming it loses a newline.
    if let Some(text) = raw_str(args, "new_text") {
        return Ok(text);
    }
    let edits = args
        .get("edits")
        .and_then(Value::as_array)
        .ok_or("give `new_text` (full content) or `edits` (range edits)")?;
    apply_range_edits(old_text, edits)
}

/// Apply LSP-style range edits (applied last-to-first so earlier offsets
/// stay valid).
fn apply_range_edits(text: &str, edits: &[Value]) -> Result<String, String> {
    let mut offsets: Vec<(usize, usize, String)> = Vec::new();
    for edit in edits {
        let pos = |leaf: &str| -> Result<usize, String> {
            let line = edit.pointer(&format!("/range/{leaf}/line")).and_then(Value::as_u64);
            let ch = edit.pointer(&format!("/range/{leaf}/character")).and_then(Value::as_u64);
            match (line, ch) {
                (Some(l), Some(c)) => byte_offset(text, l as usize, c as usize),
                _ => Err("edit range missing line/character".to_string()),
            }
        };
        let new = edit
            .get("new_text")
            .or_else(|| edit.get("newText"))
            .and_then(Value::as_str)
            .ok_or("edit missing new_text")?;
        offsets.push((pos("start")?, pos("end")?, new.to_string()));
    }
    offsets.sort_by_key(|e| std::cmp::Reverse(e.0));
    let mut out = text.to_string();
    for (start, end, new) in offsets {
        if start > end || end > out.len() {
            return Err("edit range out of bounds".to_string());
        }
        out.replace_range(start..end, &new);
    }
    Ok(out)
}

fn byte_offset(text: &str, line: usize, character: usize) -> Result<usize, String> {
    let mut offset = 0usize;
    for (i, l) in text.split_inclusive('\n').enumerate() {
        if i == line {
            return Ok(offset + character.min(l.len()));
        }
        offset += l.len();
    }
    if line == 0 {
        return Ok(character.min(text.len()));
    }
    Err(format!("line {line} beyond end of file"))
}
