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

/// Build the litellm chat-completion request body for structured file output.
pub fn build_request(model: &str, user: &str) -> Value {
    json!({
        "model": model,
        "response_format": { "type": "json_object" },
        "messages": [
            { "role": "system", "content": SYSTEM_PROMPT },
            { "role": "user", "content": user },
        ],
    })
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
