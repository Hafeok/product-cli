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

/// One file the worker produces.
#[derive(Debug, Clone, PartialEq)]
pub struct ArtifactFile {
    pub path: String,
    pub content: String,
}

/// The system instruction that fixes the worker's output contract.
pub const SYSTEM_PROMPT: &str = "You are a code-writing worker. Implement the work described, applying the How by pointer. Respond with ONLY a JSON object: {\"files\":[{\"path\":\"<relative path>\",\"content\":\"<file contents>\"}]}.";

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
