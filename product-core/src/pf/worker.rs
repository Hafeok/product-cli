//! First-party worker — a native SPMC executor (§5).
//!
//! Unlike the `claude` subprocess or a raw `litellm` completion, the first-party
//! worker is graph-aware: it asks the model for *structured file output* against
//! the work unit's schema, then applies those files. Offline (no model
//! configured) it emits a deterministic stub artifact, so the runner is testable
//! without a live model. This module is the pure part: the request envelope, the
//! response→files parse, the stub, and the safe apply.

use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use crate::error::{ProductError, Result};

/// One file the worker produces (created or overwritten whole).
#[derive(Debug, Clone, PartialEq)]
pub struct ArtifactFile {
    pub path: String,
    pub content: String,
}

/// A targeted edit to an existing file — replace the unique span `find` with
/// `replace`. This is the wiring primitive (§5): a `wire-*` work unit inserts a
/// declaration into a file it does not own without rewriting it whole.
#[derive(Debug, Clone, PartialEq)]
pub struct EditOp {
    pub path: String,
    pub find: String,
    pub replace: String,
}

/// The system instruction that fixes the worker's output contract.
pub const SYSTEM_PROMPT: &str = "You are a code-writing worker. Implement the work described, applying the How by pointer. Respond with ONLY a JSON object. To create or overwrite whole files use {\"files\":[{\"path\":\"<relative path>\",\"content\":\"<file contents>\"}]}. To modify an existing file whose current content is shown to you, return a precise edit instead: {\"edits\":[{\"path\":\"<relative path>\",\"find\":\"<a unique snippet of the current file>\",\"replace\":\"<its replacement>\"}]}. Prefer `edits` over `files` whenever a file's current content is provided.";

/// The files-only variant (`response_mode: files`): whole-file rewrites, never
/// targeted edits — for models that cannot reproduce exact unique find-spans.
pub const SYSTEM_PROMPT_FILES_ONLY: &str = "You are a code-writing worker. Implement the work described, applying the How by pointer. Respond with ONLY a JSON object of the form {\"files\":[{\"path\":\"<relative path>\",\"content\":\"<file contents>\"}]}. Always write each file COMPLETE from the first line to the last — even when the file's current content is shown to you, respond with the full rewritten file, never a fragment and never an edit object.";

/// The system prompt for a capability's declared response mode.
pub fn system_prompt(response_mode: Option<&str>) -> &'static str {
    match response_mode {
        Some("files") => SYSTEM_PROMPT_FILES_ONLY,
        _ => SYSTEM_PROMPT,
    }
}

/// Build the litellm chat-completion request body for structured file output.
/// `invocation` is the capability's optional parameter object (max_tokens,
/// temperature, chat_template_kwargs, …) merged verbatim into the body —
/// `model` and `messages` are reserved and cannot be overridden.
/// `response_mode` selects the system prompt (`files` = whole-file only).
pub fn build_request(model: &str, user: &str, invocation: Option<&Value>, response_mode: Option<&str>) -> Value {
    let mut body = json!({
        "model": model,
        "response_format": { "type": "json_object" },
        "messages": [
            { "role": "system", "content": system_prompt(response_mode) },
            { "role": "user", "content": user },
        ],
    });
    if let Some(Value::Object(inv)) = invocation {
        let obj = body.as_object_mut().expect("body is an object");
        for (k, v) in inv {
            if k != "model" && k != "messages" {
                obj.insert(k.clone(), v.clone());
            }
        }
    }
    body
}

/// Parse a structured worker response object `{ "files": [{path, content}, …] }`.
pub fn parse_files(obj: &Value) -> Result<Vec<ArtifactFile>> {
    let arr = obj
        .get("files")
        .and_then(|f| f.as_array())
        .ok_or_else(|| ProductError::ConfigError("worker response has no 'files' array".to_string()))?;
    let mut out = Vec::new();
    for f in arr {
        let path = f.get("path").and_then(|p| p.as_str())
            .ok_or_else(|| ProductError::ConfigError("a file entry is missing 'path'".to_string()))?;
        let content = f.get("content").and_then(|c| c.as_str()).unwrap_or("");
        out.push(ArtifactFile { path: path.to_string(), content: content.to_string() });
    }
    Ok(out)
}

/// Parse the `edits` array of a worker response (absent → none).
pub fn parse_edits(obj: &Value) -> Result<Vec<EditOp>> {
    let Some(arr) = obj.get("edits").and_then(|e| e.as_array()) else {
        return Ok(Vec::new());
    };
    let mut out = Vec::new();
    for e in arr {
        let path = e.get("path").and_then(|p| p.as_str())
            .ok_or_else(|| ProductError::ConfigError("an edit entry is missing 'path'".to_string()))?;
        let find = e.get("find").and_then(|f| f.as_str())
            .ok_or_else(|| ProductError::ConfigError("an edit entry is missing 'find'".to_string()))?;
        let replace = e.get("replace").and_then(|r| r.as_str()).unwrap_or("");
        out.push(EditOp { path: path.to_string(), find: find.to_string(), replace: replace.to_string() });
    }
    Ok(out)
}

/// Extract a JSON object from a model response that may be fenced or
/// prose-wrapped. Open models routinely ignore `response_format` and emit
/// ```` ```json … ``` ```` or a sentence before the object; we try a raw parse,
/// then a fenced block, then the first balanced `{…}` span.
pub fn extract_json(content: &str) -> Result<Value> {
    let trimmed = content.trim();
    if let Ok(v) = serde_json::from_str::<Value>(trimmed) {
        return Ok(v);
    }
    if let Some(body) = fenced_block(trimmed) {
        if let Ok(v) = serde_json::from_str::<Value>(body.trim()) {
            return Ok(v);
        }
    }
    if let Some(span) = balanced_object(trimmed) {
        if let Ok(v) = serde_json::from_str::<Value>(span) {
            return Ok(v);
        }
    }
    Err(ProductError::ConfigError("worker response contained no parseable JSON object".to_string()))
}

/// The body of the first ```` ``` ```` fence, dropping an optional language tag.
fn fenced_block(s: &str) -> Option<&str> {
    let open = s.find("```")?;
    let rest = &s[open + 3..];
    let end = rest.find("```")?;
    let block = &rest[..end];
    match block.find('\n') {
        Some(nl) if !block[..nl].contains('{') => Some(&block[nl + 1..]),
        _ => Some(block),
    }
}

/// The first balanced `{…}` span, respecting strings + escapes.
fn balanced_object(s: &str) -> Option<&str> {
    let start = s.find('{')?;
    let mut depth = 0i32;
    let mut in_str = false;
    let mut escaped = false;
    for (i, &b) in s.as_bytes().iter().enumerate().skip(start) {
        if in_str {
            match b {
                _ if escaped => escaped = false,
                b'\\' => escaped = true,
                b'"' => in_str = false,
                _ => {}
            }
        } else {
            match b {
                b'"' => in_str = true,
                b'{' => depth += 1,
                b'}' => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(&s[start..=i]);
                    }
                }
                _ => {}
            }
        }
    }
    None
}

/// Parse a worker response that may carry whole-file writes, targeted edits, or
/// both. Errors only if neither is present.
pub fn parse_output(obj: &Value) -> Result<(Vec<ArtifactFile>, Vec<EditOp>)> {
    let files = if obj.get("files").is_some() { parse_files(obj)? } else { Vec::new() };
    let edits = parse_edits(obj)?;
    if files.is_empty() && edits.is_empty() {
        return Err(ProductError::ConfigError("worker response has neither 'files' nor 'edits'".to_string()));
    }
    Ok((files, edits))
}

/// A deterministic offline stub artifact for a task (no model call).
pub fn stub_files(prompt: &str) -> Vec<ArtifactFile> {
    vec![ArtifactFile {
        path: format!(".product/build/artifacts/STUB-{}.md", short_hash(prompt)),
        content: format!("# Stub artifact (offline worker)\n\nNo model configured (set LITELLM_BASE_URL to dispatch live).\nThe frozen SPMC context was:\n\n{prompt}\n"),
    }]
}

/// Apply files under `root`, refusing absolute paths or `..` escapes. Returns
/// the written paths.
pub fn apply_files(files: &[ArtifactFile], root: &Path) -> Result<Vec<PathBuf>> {
    let mut written = Vec::new();
    for f in files {
        let rel = Path::new(&f.path);
        if rel.is_absolute() || rel.components().any(|c| c.as_os_str() == "..") {
            return Err(ProductError::ConfigError(format!("unsafe artifact path '{}'", f.path)));
        }
        let dest = root.join(rel);
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| ProductError::IoError(format!("{}: {}", parent.display(), e)))?;
        }
        std::fs::write(&dest, &f.content)
            .map_err(|e| ProductError::IoError(format!("{}: {}", dest.display(), e)))?;
        written.push(dest);
    }
    Ok(written)
}

/// Apply targeted edits under `root`. Each `find` must match a unique span in
/// its file (zero → error, many → ambiguous error), mirroring the Edit tool's
/// safety rule so wiring never lands in the wrong place. Returns edited paths.
pub fn apply_edits(edits: &[EditOp], root: &Path) -> Result<Vec<PathBuf>> {
    let mut written = Vec::new();
    for e in edits {
        let rel = Path::new(&e.path);
        if rel.is_absolute() || rel.components().any(|c| c.as_os_str() == "..") {
            return Err(ProductError::ConfigError(format!("unsafe edit path '{}'", e.path)));
        }
        let dest = root.join(rel);
        let current = std::fs::read_to_string(&dest)
            .map_err(|err| ProductError::IoError(format!("{}: {}", dest.display(), err)))?;
        match current.matches(&e.find).count() {
            0 => return Err(ProductError::ConfigError(format!("edit target not found in '{}'", e.path))),
            1 => {}
            n => return Err(ProductError::ConfigError(format!("edit target is ambiguous in '{}' ({n} matches)", e.path))),
        }
        let updated = current.replacen(&e.find, &e.replace, 1);
        std::fs::write(&dest, &updated)
            .map_err(|err| ProductError::IoError(format!("{}: {}", dest.display(), err)))?;
        written.push(dest);
    }
    Ok(written)
}

/// Force the worker's output to land at one declared path (the harness owns
/// placement, so a hallucinated path cannot misplace the file). Keeps a single
/// produced file (§5: one unit, one artifact) and points every edit at `target`.
pub fn retarget(mut files: Vec<ArtifactFile>, mut edits: Vec<EditOp>, target: &str) -> (Vec<ArtifactFile>, Vec<EditOp>) {
    files.truncate(1);
    for f in &mut files {
        f.path = target.to_string();
    }
    for e in &mut edits {
        e.path = target.to_string();
    }
    (files, edits)
}

/// A short stable hash for stub filenames (djb2).
fn short_hash(s: &str) -> String {
    let mut h: u64 = 5381;
    for b in s.bytes() {
        h = h.wrapping_mul(33).wrapping_add(b as u64);
    }
    format!("{:08x}", h & 0xffff_ffff)
}

#[cfg(test)]
#[path = "worker_tests.rs"]
mod tests;
