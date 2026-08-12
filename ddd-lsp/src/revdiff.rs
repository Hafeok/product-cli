//! Diff-over-two-revisions classification — the shared classifier's
//! repository path (M8, spec invariant 4).
//!
//! Takes both sides of every changed file from git, produces symbol
//! snapshots exactly the way the interceptor does (LSP overlay for hosted
//! adapters, source text for hostless ones), and routes them through the
//! *same* [`crate::classify::classify_edit`] — one classifier, two
//! callers. A host that fails or text that will not read is reported as
//! skipped, never as "no findings" (spec invariant 9).

use std::path::Path;
use std::time::Duration;

use ddd_core::config::DddConfig;
use ddd_core::contracts::{
    content_hash, finding_id, ContractDiffReport, ContractEvent, FileContracts, SkippedFile,
    ABSENT,
};
use ddd_core::{detect, gitrev};
use serde_json::json;

use crate::adapter::{self, Adapter};
use crate::classify::classify_edit;
use crate::host::{Host, Readiness};
use crate::manager::HostManager;
use crate::protocol::{flatten_symbols, to_uri, RawSymbol};

/// Classify every adapter-claimed change between `base` and `head`
/// (`None` = the working tree). `wait` bounds how long a lazily started
/// host may take to become ready before its files are reported skipped.
pub fn diff_contracts(
    manager: &mut HostManager,
    base: &str,
    head: Option<&str>,
    wait: Duration,
) -> Result<ContractDiffReport, String> {
    let root = manager.root().to_path_buf();
    let base_sha = gitrev::rev_parse(&root, base)?;
    let head_sha = head.map(|h| gitrev::rev_parse(&root, h)).transpose()?;
    let config = manager.config().cloned().unwrap_or_default();
    let mut report = ContractDiffReport {
        base: base_sha.clone(),
        head: head_sha.clone().unwrap_or_else(|| "worktree".to_string()),
        ..Default::default()
    };
    for changed in gitrev::changed_files(&root, &base_sha, head_sha.as_deref())? {
        let rel = changed.path;
        let path = root.join(&rel);
        let Some(adapter) = adapter::for_path(&path) else { continue };
        if config.ignore.iter().any(|g| detect::glob_match(g, &rel)) {
            continue;
        }
        let before = gitrev::bytes_at(&root, &base_sha, &rel)?;
        let after = match &head_sha {
            Some(h) => gitrev::bytes_at(&root, h, &rel)?,
            None => std::fs::read(&path).ok(),
        };
        match classify_sides(manager, adapter, &config, &path, &before, &after, wait) {
            Ok(events) => report.files.push(file_contracts(
                adapter,
                &rel,
                before.as_deref(),
                after.as_deref(),
                events,
            )),
            Err(reason) => report.skipped.push(SkippedFile { file: rel, reason }),
        }
    }
    Ok(report)
}

fn file_contracts(
    adapter: &Adapter,
    rel: &str,
    before: Option<&[u8]>,
    after: Option<&[u8]>,
    events: Vec<ddd_core::surface::SurfaceEvent>,
) -> FileContracts {
    FileContracts {
        file: rel.to_string(),
        language: adapter.language.to_string(),
        artifact_class: adapter.artifact_class.to_string(),
        before: before.map(content_hash).unwrap_or_else(|| ABSENT.to_string()),
        after: after.map(content_hash).unwrap_or_else(|| ABSENT.to_string()),
        events: events
            .into_iter()
            .map(|event| ContractEvent { id: finding_id(adapter.language, rel, &event), event })
            .collect(),
    }
}

/// Both sides through the shared classifier. Hostless adapters classify
/// text directly; hosted ones snapshot `documentSymbol` per side through
/// an overlay, exactly as the interceptor does.
#[allow(clippy::too_many_arguments)]
fn classify_sides(
    manager: &mut HostManager,
    adapter: &'static Adapter,
    config: &DddConfig,
    path: &Path,
    before: &Option<Vec<u8>>,
    after: &Option<Vec<u8>>,
    wait: Duration,
) -> Result<Vec<ddd_core::surface::SurfaceEvent>, String> {
    let before_text = side_text(before)?;
    let after_text = side_text(after)?;
    let flags = manager.flags(adapter.language);
    let (before_syms, after_syms) = if adapter.hosted {
        let host = manager.host(adapter.language).map_err(|e| e.to_string())?;
        snapshot_sides(host, path, &before_text, &after_text, wait)?
    } else {
        (Vec::new(), Vec::new())
    };
    let mut events =
        classify_edit(adapter, &flags, &before_text, &before_syms, &after_text, &after_syms);
    if let Some(enrich) = adapter.enrich {
        enrich(manager.root(), path, config, &mut events);
    }
    Ok(events)
}

fn side_text(bytes: &Option<Vec<u8>>) -> Result<String, String> {
    match bytes {
        None => Ok(String::new()),
        Some(b) => String::from_utf8(b.clone()).map_err(|_| "not UTF-8".to_string()),
    }
}

fn snapshot_sides(
    host: &mut Host,
    path: &Path,
    before_text: &str,
    after_text: &str,
    wait: Duration,
) -> Result<(Vec<RawSymbol>, Vec<RawSymbol>), String> {
    match host.wait_ready(wait).map_err(|e| e.to_string())? {
        Readiness::Ready { .. } => {}
        loading => {
            return Err(format!(
                "host `{}` not ready within {}s (waiting on {:?})",
                host.language(),
                wait.as_secs(),
                loading
            ))
        }
    }
    host.open_with_text(path, before_text).map_err(|e| e.to_string())?;
    let before = symbols(host, path)?;
    host.overlay(path, after_text).map_err(|e| e.to_string())?;
    let after = symbols(host, path)?;
    // Put the host back on the disk state so later tools see reality.
    if let Ok(disk) = std::fs::read_to_string(path) {
        let _ = host.overlay(path, &disk);
    }
    Ok((before, after))
}

fn symbols(host: &mut Host, path: &Path) -> Result<Vec<RawSymbol>, String> {
    let result = host
        .request("textDocument/documentSymbol", json!({"textDocument": {"uri": to_uri(path)}}))
        .map_err(|e| e.to_string())?;
    Ok(flatten_symbols(&result))
}

#[cfg(test)]
#[path = "revdiff_tests.rs"]
mod tests;
