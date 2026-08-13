//! M4 governance handlers — why, graph_query, declare, accept_risk.
//!
//! All of these run over the `.ddd/` store (PRD §8): `why` mirrors the
//! CLI's resolution as structured content; `declare_seam` files the seam
//! (warning when no verdict knowledge is declared); `declare_pattern`
//! fetches the catalog predicate's obligation list — unanswered
//! obligations are rejected; `accept_risk` files the record a
//! suppression must cite (the M2 `UNCITED_SUPPRESSION` check).
//!
//! Every declare tool takes `amend: true` to revise a filed entry
//! (`dec/ddd/amend-explicit-evidence-frozen`): judgement fields amend and
//! omitted ones are preserved, while identity and LSP-derived evidence
//! carry forward untouched. The flag is explicit in both directions, so a
//! create can never silently overwrite and an amend can never silently
//! create.

use std::collections::BTreeMap;
use std::path::Path;

use ddd_core::store::{load, STORE_DIR};
use serde_json::{json, Value};

use crate::intercept::slug;
use crate::state::{opt_str, req_str, ServeState};

type ToolResult = Result<Value, String>;

/// Resolve the create-vs-amend intent for `path`, loading the filed entry
/// when amending. `amend: true` requires an entry to exist; its absence
/// requires that none does — neither stands in for the other.
fn amend_target<T: serde::de::DeserializeOwned>(
    state: &ServeState,
    path: &Path,
    id: &str,
    args: &Value,
) -> Result<Option<T>, String> {
    let amend = args.get("amend").and_then(Value::as_bool).unwrap_or(false);
    match (amend, path.exists()) {
        (false, true) => Err(format!(
            "{id} is already filed at {} — pass `amend: true` to revise it",
            state.rel_display(path)
        )),
        (true, false) => Err(format!("nothing is filed at {id} — drop `amend` to file it")),
        (true, true) => {
            let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
            serde_yaml::from_str::<T>(&text)
                .map(Some)
                .map_err(|e| format!("{id} is filed at {} but unreadable: {e}", state.rel_display(path)))
        }
        (false, false) => Ok(None),
    }
}

/// `"amended"` or `"filed"`, for the tool's status field.
fn outcome(existing: bool) -> &'static str {
    if existing {
        "amended"
    } else {
        "filed"
    }
}

/// `ddd_why`: the governance chain behind an id.
pub fn why(state: &ServeState, args: &Value) -> ToolResult {
    let id = req_str(args, "id")?;
    let store = load(&state.root.join(STORE_DIR));
    // Detection re-reads the config sources per call; the same three-way
    // resolution as the CLI, per PRD §8.
    let detected = ddd_core::detect::detect(&state.root, store.config.as_ref(), &[]);
    match ddd_core::why::resolve_why(&store, Some(&detected), &id) {
        ddd_core::why::WhyResolution::Resolved(chain) => {
            Ok(json!({"status": "ok", "id": id, "chain": chain}))
        }
        ddd_core::why::WhyResolution::DetectedUnfiled { namespace, rule_id, sources } => {
            Ok(json!({
                "status": "detected-unfiled", "id": id,
                "namespace": namespace, "rule_id": rule_id, "detected": sources,
                "hint": "the rule is detected but no manifest entry maps it — file a decision plus a manifest entry, then why resolves it",
            }))
        }
        ddd_core::why::WhyResolution::NotFound => Ok(json!({
            "status": "not-found", "id": id,
            "hint": "no decision, claim, manifest rule, pattern, or seam carries this id, and detection does not know it either",
        })),
    }
}

/// `ddd_graph_query`: read one entry family with simple filters.
pub fn graph_query(state: &ServeState, args: &Value) -> ToolResult {
    let select = req_str(args, "select")?;
    let id = opt_str(args, "id");
    let status = opt_str(args, "status");
    let predicate = opt_str(args, "predicate");
    let store = load(&state.root.join(STORE_DIR));
    let keep = |candidate: &str| id.as_deref().map(|w| w == candidate).unwrap_or(true);
    let entries: Vec<Value> = match select.as_str() {
        "predicates" => to_rows(store.predicates.iter().filter(|p| keep(&p.id))),
        "claims" => to_rows(store.claims.iter().filter(|c| {
            keep(&c.id)
                && status.as_deref().map(|s| c.status.as_str() == s).unwrap_or(true)
                && predicate
                    .as_deref()
                    .map(|p| c.predicate.as_deref() == Some(p))
                    .unwrap_or(true)
        })),
        "decisions" => to_rows(store.decisions.iter().filter(|d| keep(&d.id))),
        "patterns" => to_rows(store.patterns.iter().filter(|p| keep(&p.id))),
        "seams" => to_rows(store.seams.iter().filter(|s| keep(&s.id))),
        "seam-events" => to_rows(store.seam_events.iter().filter(|e| keep(&e.id))),
        "manifests" => store
            .manifests
            .iter()
            .map(|m| json!({"namespace": m.name, "rules": m.file.rules}))
            .collect(),
        other => {
            return Err(format!(
                "unknown selector `{other}` — one of predicates | claims | decisions | manifests | patterns | seams | seam-events"
            ))
        }
    };
    Ok(json!({"status": "ok", "select": select, "count": entries.len(), "entries": entries}))
}

fn to_rows<'a, T: serde::Serialize + 'a>(items: impl Iterator<Item = &'a T>) -> Vec<Value> {
    items.filter_map(|i| serde_json::to_value(i).ok()).collect()
}

/// `ddd_declare_seam`: file the declaration, warn when the boundary
/// absorbs no demand. A `binding` block signs the exact transition the
/// declaration discharges (M8): its parent state must be committed —
/// a dirty file refuses to bind, naming the constraint.
pub fn declare_seam(state: &ServeState, args: &Value) -> ToolResult {
    let id = req_str(args, "id")?;
    if !id.starts_with("seam/") {
        return Err("seam ids are `seam/<area>/<name>`".to_string());
    }
    let path = state.root.join(STORE_DIR).join("seams").join(format!("{}.yaml", slug(&id)));
    let existing: Option<ddd_core::seam::SeamDeclaration> =
        amend_target(state, &path, &id, args)?;
    let amending = existing.is_some();
    let mut seam = seam_entry(args, &id, existing)?;
    let bound = match args.get("binding") {
        Some(spec) => Some(attach_binding(state, &mut seam, spec)?),
        None => None,
    };
    write_yaml(&path, &seam)?;
    let warning = seam.verdict_knowledge.trim().is_empty().then_some(
        "verdict_knowledge is empty — this boundary declares seam cost with no demand absorbed (PRD §8); state what the boundary encodes about the verdict; pass `amend: true` to fill it in",
    );
    Ok(json!({
        "status": outcome(amending), "id": id, "path": state.rel_display(&path),
        "binding": bound, "warning": warning,
    }))
}

/// Verify and append a signed binding. Edit-time form (no
/// `base_revision`): the file must be clean against HEAD and `before`
/// must equal the committed parent's hash. Post-hoc form (explicit
/// `base_revision`, the CI-remedy path): `before` is verified against
/// that committed revision instead.
fn attach_binding(
    state: &ServeState,
    seam: &mut ddd_core::seam::SeamDeclaration,
    spec: &Value,
) -> Result<Value, String> {
    let get = |k: &str| {
        spec.get(k)
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .ok_or_else(|| format!("binding.{k} is required"))
    };
    let (symbol, file, before, after) = (get("symbol")?, get("file")?, get("before")?, get("after")?);
    let base_revision = match spec.get("base_revision").and_then(Value::as_str) {
        Some(rev) if !rev.trim().is_empty() => {
            let rev = ddd_core::gitrev::rev_parse(&state.root, rev.trim())?;
            verify_before(state, &rev, &file, &before)?;
            rev
        }
        _ => {
            let head = ddd_core::gitrev::head(&state.root).map_err(|_| {
                "the repository has no HEAD commit — a declaration binds against committed \
                 parent state, and there is none to bind against (M8 ruling 2)"
                    .to_string()
            })?;
            let disk = std::fs::read(state.root.join(&file)).ok();
            let at_head = ddd_core::gitrev::bytes_at(&state.root, &head, &file)?;
            if disk != at_head {
                return Err(format!(
                    "{file} is dirty against HEAD ({}) — a declaration never binds uncommitted \
                     parent state; commit the current state first (M8 ruling 2, the ledger's \
                     construction-limit precedent)",
                    &head[..12.min(head.len())]
                ));
            }
            verify_before(state, &head, &file, &before)?;
            head
        }
    };
    let binding =
        ddd_core::binding::SeamBinding::seal(&symbol, &file, &before, &after, &base_revision);
    if !seam.bindings.iter().any(|b| b.hash == binding.hash) {
        seam.bindings.push(binding.clone());
    }
    seam.format = seam.format.max(2);
    serde_json::to_value(&binding).map_err(|e| e.to_string())
}

/// The binding's `before` must equal the parent state actually committed
/// at `rev` — a declaration signs a reproducible transition or nothing.
fn verify_before(state: &ServeState, rev: &str, file: &str, before: &str) -> Result<(), String> {
    let committed = ddd_core::gitrev::bytes_at(&state.root, rev, file)?
        .map(|b| ddd_core::contracts::content_hash(&b))
        .unwrap_or_else(|| ddd_core::contracts::ABSENT.to_string());
    if committed != before {
        return Err(format!(
            "binding.before is {before} but {file} at {} is {committed} — the declaration \
             would not sign the transition it claims to discharge",
            &rev[..12.min(rev.len())]
        ));
    }
    Ok(())
}

/// Build the entry to write. On amend, judgement fields take the supplied
/// value and fall back to the filed one; `contract_location` plus
/// `metadata` carry forward untouched, because the correspondence rows are
/// only evidence while those stay machine-authored (`DDD-arch-04`).
fn seam_entry(
    args: &Value,
    id: &str,
    existing: Option<ddd_core::seam::SeamDeclaration>,
) -> Result<ddd_core::seam::SeamDeclaration, String> {
    let Some(prev) = existing else {
        let mut metadata: BTreeMap<String, serde_yaml::Value> = BTreeMap::new();
        if let Some(s) = opt_str(args, "symbol") {
            metadata.insert("symbol".to_string(), s.into());
        }
        return Ok(ddd_core::seam::SeamDeclaration {
            format: 1,
            id: id.to_string(),
            boundary: req_str(args, "boundary")?,
            verdict_knowledge: opt_str(args, "verdict_knowledge").unwrap_or_default(),
            contract_location: req_str(args, "contract_location")?,
            obligations: string_list(args, "obligations"),
            metadata,
            bindings: Vec::new(),
            notes: opt_str(args, "notes"),
        });
    };
    Ok(ddd_core::seam::SeamDeclaration {
        format: prev.format,
        id: id.to_string(),
        boundary: opt_str(args, "boundary").unwrap_or(prev.boundary),
        verdict_knowledge: opt_str(args, "verdict_knowledge").unwrap_or(prev.verdict_knowledge),
        contract_location: prev.contract_location,
        obligations: match args.get("obligations") {
            Some(_) => string_list(args, "obligations"),
            None => prev.obligations,
        },
        metadata: prev.metadata,
        bindings: prev.bindings,
        notes: opt_str(args, "notes").or(prev.notes),
    })
}

/// `ddd_declare_pattern`: obligations come from the catalog predicate;
/// unanswered ones reject the declaration.
pub fn declare_pattern(state: &ServeState, args: &Value) -> ToolResult {
    let id = req_str(args, "id")?;
    if !id.starts_with("pat/") {
        return Err("pattern-instance ids are `pat/<pattern>/<instance>`".to_string());
    }
    let path = state.root.join(STORE_DIR).join("patterns").join(format!("{}.yaml", slug(&id)));
    let existing: Option<ddd_core::pattern::PatternInstance> =
        amend_target(state, &path, &id, args)?;
    let pattern = pattern_predicate_for(args, existing.as_ref())?;
    let store = load(&state.root.join(STORE_DIR));
    let predicate = store
        .predicates
        .iter()
        .find(|p| p.id == pattern)
        .ok_or_else(|| format!("pattern predicate `{pattern}` is not in the catalog (ontology rule 5)"))?;
    // Amending merges onto the filed answers, so one obligation can be
    // refined without restating the rest; completeness is still checked.
    let base = existing.as_ref().map(|e| e.obligations.clone()).unwrap_or_default();
    let answers = checked_answers(args, base, &predicate.obligations, &pattern)?;
    let instance = opt_str(args, "instance")
        .or_else(|| existing.as_ref().map(|e| e.instance.clone()))
        .ok_or("instance is required when filing")?;
    let entry = ddd_core::pattern::PatternInstance {
        format: existing.as_ref().map(|e| e.format).unwrap_or(1),
        id: id.clone(),
        pattern_predicate: pattern,
        instance: instance.clone(),
        obligations: answers,
        decision: existing.as_ref().and_then(|e| e.decision.clone()),
        notes: opt_str(args, "notes").or_else(|| existing.as_ref().and_then(|e| e.notes.clone())),
    };
    write_yaml(&path, &entry)?;
    Ok(json!({
        "status": outcome(existing.is_some()), "id": id, "path": state.rel_display(&path),
    }))
}

/// The predicate a filed instance instantiates is identity, not judgement:
/// changing it would change which obligations apply, so it is a new id.
fn pattern_predicate_for(
    args: &Value,
    existing: Option<&ddd_core::pattern::PatternInstance>,
) -> Result<String, String> {
    let Some(prev) = existing else {
        return req_str(args, "pattern");
    };
    match opt_str(args, "pattern") {
        Some(p) if p != prev.pattern_predicate => Err(format!(
            "a filed instance cannot change its predicate ({} -> {p}) — file it under a new id",
            prev.pattern_predicate
        )),
        _ => Ok(prev.pattern_predicate.clone()),
    }
}

/// The obligation answers, rejected unless every catalog obligation has
/// a non-blank answer (PRD §8).
fn checked_answers(
    args: &Value,
    base: BTreeMap<String, String>,
    obligations: &[String],
    pattern: &str,
) -> Result<BTreeMap<String, String>, String> {
    let mut answers = base;
    if let Some(supplied) = args.get("obligation_answers").and_then(Value::as_object) {
        for (k, v) in supplied {
            if let Some(s) = v.as_str() {
                answers.insert(k.clone(), s.to_string());
            }
        }
    }
    let unanswered: Vec<&str> = obligations
        .iter()
        .filter(|o| answers.get(*o).map(|a| a.trim().is_empty()).unwrap_or(true))
        .map(String::as_str)
        .collect();
    if unanswered.is_empty() {
        Ok(answers)
    } else {
        Err(format!(
            "unanswered obligations for {pattern}: {} — answer every catalog obligation to declare the instance",
            unanswered.join(", ")
        ))
    }
}

/// `ddd_accept_risk`: the record a suppression cites (ontology rule 4).
pub fn accept_risk(state: &ServeState, args: &Value) -> ToolResult {
    let diagnostic_id = req_str(args, "diagnostic_id")?;
    let id = format!("risk/rule/{}", slug(&diagnostic_id));
    let path = state.root.join(STORE_DIR).join("decisions").join(format!("{}.yaml", slug(&id)));
    let existing: Option<ddd_core::decision::Decision> = amend_target(state, &path, &id, args)?;
    let amending = existing.is_some();
    let record = ddd_core::decision::Decision {
        format: existing.as_ref().map(|e| e.format).unwrap_or(1),
        id: id.clone(),
        kind: ddd_core::decision::DecisionKind::RiskAcceptance,
        title: format!("Risk accepted: {diagnostic_id} suppressed"),
        rationale: field(args, "rationale", existing.as_ref().map(|e| e.rationale.clone()))?,
        principal: field(args, "principal", existing.as_ref().map(|e| e.principal.clone()))?,
        based_on: existing.as_ref().map(|e| e.based_on.clone()).unwrap_or_default(),
        revisit_if: existing.as_ref().map(|e| e.revisit_if.clone()).unwrap_or_default(),
        date: Some(chrono::Utc::now().format("%Y-%m-%d").to_string()),
        notes: opt_str(args, "notes").or_else(|| existing.as_ref().and_then(|e| e.notes.clone())),
    };
    write_yaml(&path, &record)?;
    Ok(json!({
        "status": outcome(amending), "id": id, "path": state.rel_display(&path),
        "next": format!("cite it from the manifest entry: suppression.risk_acceptance: {id}"),
    }))
}

/// A field that is required when filing and optional when amending.
fn field(args: &Value, key: &str, fallback: Option<String>) -> Result<String, String> {
    opt_str(args, key)
        .or(fallback)
        .ok_or_else(|| format!("{key} is required when filing"))
}

fn string_list(args: &Value, key: &str) -> Vec<String> {
    args.get(key)
        .and_then(Value::as_array)
        .map(|a| a.iter().filter_map(Value::as_str).map(str::to_string).collect())
        .unwrap_or_default()
}

fn write_yaml<T: serde::Serialize>(path: &std::path::Path, value: &T) -> Result<(), String> {
    let yaml = serde_yaml::to_string(value).map_err(|e| e.to_string())?;
    product_core::fileops::write_file_atomic(path, &yaml).map_err(|e| e.to_string())
}
