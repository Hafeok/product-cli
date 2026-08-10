//! The `apply_edit` interceptor — PRD §8 outcome semantics, exactly.
//!
//! Non-surface applies. Surface with a matching same-session declaration
//! applies plus links. Surface without one rejects with a structured
//! demand: LSP-derived facts pre-filled, judgment fields blank
//! (`dec/ddd/rejection-facts-prefilled`). Mode per artifact class:
//! `enforce` rejects, `warn` applies with a warning, `off` skips
//! classification. Every classified surface outcome logs one row under
//! `.ddd/seams/events/` — the correspondence dataset.

use std::path::Path;

use ddd_lsp::adapter::Adapter;
use ddd_lsp::classify::classify_edit;
use ddd_lsp::host::Host;
use ddd_lsp::protocol::{flatten_symbols, position_params, to_uri};
use ddd_lsp::surface::{ChangeKind, SurfaceEvent};
use serde_json::{json, Value};

use crate::state::{opt_str, raw_str, req_str, ServeState};
use crate::state::Declared;

pub fn apply_edit(state: &ServeState, args: &Value) -> Result<Value, String> {
    let file = state.resolve_file(&req_str(args, "file")?);
    let old_text = read_or_empty(&file)?;
    let new_text = resolve_new_text(args, &old_text)?;
    let config = ddd_core::config::load(&state.root).unwrap_or_default();
    let (adapter, flags) = {
        let manager = state.manager.lock().map_err(|_| "host manager poisoned".to_string())?;
        let adapter = manager
            .resolve_adapter(opt_str(args, "language").as_deref(), Some(&file))
            .map_err(|e| e.to_string())?;
        (adapter, manager.flags(adapter.language))
    };
    let mode = config.intercept_for(adapter.artifact_class).to_string();
    if mode == "off" {
        product_core::fileops::write_file_atomic(&file, &new_text).map_err(|e| e.to_string())?;
        if let Ok(mut manager) = state.manager.lock() {
            if let Some(host) = manager.existing_host(adapter.language) {
                let _ = host.overlay(&file, &new_text);
            }
        }
        return Ok(json!({"status": "applied", "intercepted": false, "mode": "off"}));
    }
    crate::lang_tools::with_ready_host(state, args, Some(&file), |host| {
        intercept(state, host, adapter, &flags, &mode, &file, &old_text, &new_text)
    })
}

/// The classified path: diff the symbol surfaces, decide per PRD §8.
/// The caller holds the manager lock through `host` — nothing here may
/// lock the manager again.
#[allow(clippy::too_many_arguments)]
fn intercept(
    state: &ServeState,
    host: &mut Host,
    adapter: &'static Adapter,
    flags: &ddd_lsp::adapter::AdapterFlags,
    mode: &str,
    file: &Path,
    old_text: &str,
    new_text: &str,
) -> Result<Value, String> {
    host.ensure_open(file).map_err(|e| e.to_string())?;
    let before = symbols(host, file)?;
    host.overlay(file, new_text).map_err(|e| e.to_string())?;
    let after = symbols(host, file)?;
    let events = classify_edit(adapter, flags, old_text, &before, new_text, &after);
    let surface: Vec<SurfaceEvent> = events.into_iter().filter(|e| e.surface).collect();
    if surface.is_empty() {
        write_disk(file, new_text)?;
        return Ok(json!({"status": "applied", "intercepted": true, "mode": mode,
                          "surface_events": 0}));
    }
    let counts = reference_counts(host, file, old_text, new_text, &surface);
    let warning = (adapter.posture_warning)(file, flags);
    let rel = state.rel_display(file);
    if mode == "warn" {
        // Advisory matching keeps the file arm; the rows carry the generous
        // link, but declaration metadata stays symbol-exact (enforce-only)
        // so a broad match can never overwrite another symbol's facts.
        let matches = match_declarations(state, &rel, &surface, false);
        let rows =
            log_events(state, adapter, mode, &rel, &surface, &counts, "applied-warned", &matches)?;
        write_disk(file, new_text)?;
        return Ok(json!({
            "status": "applied", "intercepted": true, "mode": "warn",
            "warning": "contract-surface change applied unverified — declare the seam (ddd_declare_seam) or switch this class to enforce",
            "surface": demands(&surface, &counts, adapter, &rel),
            "events_logged": rows, "posture_warning": warning,
        }));
    }
    let matches = match_declarations(state, &rel, &surface, true);
    if matches.iter().all(Option::is_some) {
        let linked: Vec<String> = matches.iter().flatten().map(|d| d.id.clone()).collect();
        let rows =
            log_events(state, adapter, mode, &rel, &surface, &counts, "applied-linked", &matches)?;
        link_seam_metadata(state, &surface, &counts, &matches);
        write_disk(file, new_text)?;
        return Ok(json!({
            "status": "applied", "intercepted": true, "mode": "enforce",
            "linked": linked, "events_logged": rows, "posture_warning": warning,
        }));
    }
    // Reject: put the host back on the disk state, demand declarations.
    host.overlay(file, old_text).map_err(|e| e.to_string())?;
    let unmatched: Vec<usize> =
        matches.iter().enumerate().filter(|(_, m)| m.is_none()).map(|(i, _)| i).collect();
    let only_unmatched: Vec<SurfaceEvent> =
        unmatched.iter().map(|&i| surface[i].clone()).collect();
    let unmatched_counts: Vec<Option<u64>> = unmatched.iter().map(|&i| counts[i]).collect();
    let rows =
        log_events(state, adapter, mode, &rel, &only_unmatched, &unmatched_counts, "rejected", &[])?;
    Ok(json!({
        "status": "rejected", "mode": "enforce",
        "reason": "contract-surface change without a matching same-session declaration (PRD §8)",
        "demands": demands(&only_unmatched, &unmatched_counts, adapter, &rel),
        "next": "file ddd_declare_seam (or ddd_declare_pattern) covering each demanded surface, then re-apply this edit",
        "events_logged": rows, "posture_warning": warning,
    }))
}

fn write_disk(file: &Path, new_text: &str) -> Result<(), String> {
    product_core::fileops::write_file_atomic(file, new_text).map_err(|e| e.to_string())
}

fn read_or_empty(file: &Path) -> Result<String, String> {
    if file.exists() {
        std::fs::read_to_string(file).map_err(|e| format!("{}: {e}", file.display()))
    } else {
        Ok(String::new())
    }
}

fn resolve_new_text(args: &Value, old_text: &str) -> Result<String, String> {
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

fn symbols(host: &mut Host, file: &Path) -> Result<Vec<ddd_lsp::protocol::RawSymbol>, String> {
    let result = host
        .request("textDocument/documentSymbol", json!({"textDocument": {"uri": to_uri(file)}}))
        .map_err(|e| e.to_string())?;
    Ok(flatten_symbols(&result))
}

/// References per event, queried on the side the symbol exists on: the
/// after-overlay for additions/changes, the before-state for removals.
fn reference_counts(
    host: &mut Host,
    file: &Path,
    old_text: &str,
    new_text: &str,
    events: &[SurfaceEvent],
) -> Vec<Option<u64>> {
    let removed: Vec<usize> = events
        .iter()
        .enumerate()
        .filter(|(_, e)| e.change == ChangeKind::Removed)
        .map(|(i, _)| i)
        .collect();
    let mut counts: Vec<Option<u64>> = events
        .iter()
        .map(|e| {
            if e.change == ChangeKind::Removed {
                None
            } else {
                count_at(host, file, e.facts.sel_line, e.facts.sel_char)
            }
        })
        .collect();
    if !removed.is_empty() && host.overlay(file, old_text).is_ok() {
        for i in removed {
            counts[i] = count_at(host, file, events[i].facts.sel_line, events[i].facts.sel_char);
        }
        let _ = host.overlay(file, new_text);
    }
    counts
}

fn count_at(host: &mut Host, file: &Path, line: u64, character: u64) -> Option<u64> {
    let mut params = position_params(file, line, character);
    params["context"] = json!({"includeDeclaration": false});
    host.request("textDocument/references", params)
        .ok()
        .map(|r| ddd_lsp::protocol::normalize_locations(&r).len() as u64)
}

/// PRD §8 + settled question 3: facts pre-filled, judgment blank.
fn demands(
    events: &[SurfaceEvent],
    counts: &[Option<u64>],
    adapter: &Adapter,
    rel_file: &str,
) -> Vec<Value> {
    events
        .iter()
        .zip(counts)
        .map(|(e, count)| {
            json!({
                "surface": {
                    "file": rel_file,
                    "symbol": e.facts.name,
                    "container": e.facts.container,
                    "kind": e.facts.kind,
                    "change": e.change,
                    "signature": e.facts.signature,
                    "visibility": e.facts.visibility,
                    "reference_count": count,
                    "language": adapter.language,
                    "artifact_class": adapter.artifact_class,
                    "rule": e.rule,
                    "rule_claim": e.rule_claim,
                    "extra": e.facts.extra,
                },
                "required": "a seam declaration (ddd_declare_seam) or pattern instance (ddd_declare_pattern)",
                "template": {
                    "id": format!("seam/{}/{}", adapter.language, slug(&e.facts.name)),
                    "boundary": format!("{} {} in {}", e.facts.kind, e.facts.name, rel_file),
                    "contract_location": format!("{rel_file}#{}", e.facts.name),
                    "symbol": e.facts.name,
                    "verdict_knowledge": "",
                    "obligations": [],
                },
            })
        })
        .collect()
}

/// Same-session matching (`dec/ddd/enforce-matching-tightens-to-symbol`).
///
/// In enforce mode a declaration covers an event only when it names the
/// event's symbol: the file arm admitted unexamined symbols and corrupted
/// the correspondence record (`DDD-arch-05`, seam-event/4). In warn mode
/// the declaration is advisory and the edit applies either way, so the
/// broad file arm stays — a generous link on a row written regardless.
fn match_declarations(
    state: &ServeState,
    rel_file: &str,
    events: &[SurfaceEvent],
    symbol_only: bool,
) -> Vec<Option<Declared>> {
    let declared = state.session.lock().map(|l| l.declared.clone()).unwrap_or_default();
    events
        .iter()
        .map(|e| {
            declared
                .iter()
                .find(|d| {
                    d.symbol.as_deref() == Some(e.facts.name.as_str())
                        || (!symbol_only && d.contract_location.contains(rel_file))
                })
                .cloned()
        })
        .collect()
}

/// One correspondence row per surface event (PRD §8).
#[allow(clippy::too_many_arguments)]
fn log_events(
    state: &ServeState,
    adapter: &Adapter,
    mode: &str,
    rel_file: &str,
    events: &[SurfaceEvent],
    counts: &[Option<u64>],
    outcome: &str,
    matches: &[Option<Declared>],
) -> Result<Vec<String>, String> {
    let ddd_dir = state.root.join(ddd_core::store::STORE_DIR);
    let mut ids = Vec::new();
    for (i, event) in events.iter().enumerate() {
        let seq = ddd_core::seam_event::next_seq(&ddd_dir);
        let row = ddd_core::seam_event::SeamEvent {
            format: 1,
            id: format!("seam-event/{seq}"),
            at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            language: adapter.language.to_string(),
            artifact_class: adapter.artifact_class.to_string(),
            mode: mode.to_string(),
            file: rel_file.to_string(),
            symbol: event.facts.name.clone(),
            container: event.facts.container.clone(),
            kind: event.facts.kind.clone(),
            change: change_name(event.change).to_string(),
            visibility: event.facts.visibility.clone(),
            signature: event.facts.signature.clone(),
            reference_count: counts.get(i).copied().flatten(),
            rule: event.rule.map(str::to_string),
            rule_claim: event.rule_claim.map(str::to_string),
            outcome: outcome.to_string(),
            linked_declaration: matches.get(i).and_then(|m| m.as_ref()).map(|d| d.id.clone()),
            extra: event.facts.extra.clone(),
        };
        ddd_core::seam_event::save(&ddd_dir, &row).map_err(|e| e.to_string())?;
        ids.push(row.id);
    }
    Ok(ids)
}

fn change_name(change: ChangeKind) -> &'static str {
    match change {
        ChangeKind::Added => "added",
        ChangeKind::Removed => "removed",
        ChangeKind::SignatureChanged => "signature-changed",
        ChangeKind::DecoratorChanged => "decorator-changed",
        ChangeKind::VisibilityChanged => "visibility-changed",
    }
}

/// Update matched seam declarations with the LSP-derived metadata, so the
/// declaration carries the correspondence facts (PRD §8).
fn link_seam_metadata(
    state: &ServeState,
    events: &[SurfaceEvent],
    counts: &[Option<u64>],
    matches: &[Option<Declared>],
) {
    for (i, m) in matches.iter().enumerate() {
        let Some(decl) = m else { continue };
        if !decl.id.starts_with("seam/") {
            continue;
        }
        let Some(event) = events.get(i) else { continue };
        let path = state
            .root
            .join(ddd_core::store::STORE_DIR)
            .join("seams")
            .join(format!("{}.yaml", slug(&decl.id)));
        let Ok(text) = std::fs::read_to_string(&path) else { continue };
        let Ok(mut seam) = serde_yaml::from_str::<ddd_core::seam::SeamDeclaration>(&text) else {
            continue;
        };
        let meta = &mut seam.metadata;
        let mut set = |k: &str, v: serde_yaml::Value| {
            meta.insert(k.to_string(), v);
        };
        set("symbol", event.facts.name.clone().into());
        set("kind", event.facts.kind.clone().into());
        set("signature", event.facts.signature.clone().into());
        set("visibility", event.facts.visibility.clone().into());
        if let Some(count) = counts.get(i).copied().flatten() {
            set("reference_count", count.into());
        }
        for (k, v) in &event.facts.extra {
            set(k, v.clone().into());
        }
        if let Ok(yaml) = serde_yaml::to_string(&seam) {
            let _ = product_core::fileops::write_file_atomic(&path, &yaml);
        }
    }
}

/// `seam/api/foo` → `seam-api-foo` (the on-disk filename form).
pub fn slug(id: &str) -> String {
    id.chars()
        .map(|c| if c.is_ascii_alphanumeric() { c.to_ascii_lowercase() } else { '-' })
        .collect()
}
