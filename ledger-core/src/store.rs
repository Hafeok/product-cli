//! Loading `.decisions/` — the log is the source of truth (§5).
//!
//! Files in git are the log; nothing here is derived from an index, because
//! L0 has no index. Loading never hard-fails on one bad entry: each file
//! that will not parse becomes a named `SCHEMA` finding so `verify` reports
//! the whole store in one pass rather than one file per run. Same discipline
//! as the `ddd` store, for the same reason.

use std::path::{Path, PathBuf};

use crate::changeset::ChangeSet;
use crate::finding::Finding;
use crate::format;
use crate::set::DecisionSet;
use crate::STORE_DIR;

/// A log file with the path it came from, so findings can name it.
#[derive(Debug, Clone)]
pub struct LoggedChangeSet {
    pub path: PathBuf,
    pub file: ChangeSet,
}

/// The parsed store plus every schema fault met while loading it.
#[derive(Debug, Default)]
pub struct Store {
    /// The repo root holding `.decisions/` — what [`crate::blame`] needs.
    pub root: PathBuf,
    pub dir: PathBuf,
    pub sets: Vec<DecisionSet>,
    pub log: Vec<LoggedChangeSet>,
    pub schema_findings: Vec<Finding>,
}

impl Store {
    /// The set with this id, when one is declared.
    pub fn set(&self, id: &str) -> Option<&DecisionSet> {
        self.sets.iter().find(|s| s.id == id)
    }

    /// How many entries loaded cleanly, for the summary line.
    pub fn entry_count(&self) -> usize {
        self.log
            .iter()
            .map(|c| {
                c.file.decisions.len()
                    + c.file.versions.len()
                    + c.file.acceptances.len()
                    + c.file.revocations.len()
            })
            .sum::<usize>()
            + self.sets.len()
    }
}

/// Walk upward from `start` to the first directory containing `.decisions/`.
pub fn find_root(start: &Path) -> Option<PathBuf> {
    let mut cur = Some(start);
    while let Some(dir) = cur {
        if dir.join(STORE_DIR).is_dir() {
            return Some(dir.to_path_buf());
        }
        cur = dir.parent();
    }
    None
}

/// Load the store rooted at `repo_root`.
pub fn load(repo_root: &Path) -> Store {
    let dir = repo_root.join(STORE_DIR);
    let mut store = Store {
        root: repo_root.to_path_buf(),
        dir: dir.clone(),
        ..Store::default()
    };
    load_sets(&dir.join("sets"), &mut store);
    load_log(&dir.join("log"), &mut store);
    store.sets.sort_by(|a, b| a.id.cmp(&b.id));
    store.log.sort_by(|a, b| a.file.id.cmp(&b.file.id));
    store
}

fn load_sets(dir: &Path, store: &mut Store) {
    for path in yaml_files(dir) {
        match std::fs::read_to_string(&path) {
            Ok(text) => take_set(store, &file_label(&path), &stem(&path), &text),
            Err(e) => store.schema_findings.push(Finding::schema(&file_label(&path), e.to_string())),
        }
    }
}

fn load_log(dir: &Path, store: &mut Store) {
    for path in yaml_files(dir) {
        match std::fs::read_to_string(&path) {
            Ok(text) => take_log(store, path.clone(), &file_label(&path), &stem(&path), &text),
            Err(e) => store.schema_findings.push(Finding::schema(&file_label(&path), e.to_string())),
        }
    }
}

/// Parse one set file's text into the store. Shared by the disk loader and
/// the at-revision loader, so both readings apply identical rules.
pub(crate) fn take_set(store: &mut Store, label: &str, stem: &str, text: &str) {
    match serde_yaml::from_str::<DecisionSet>(text) {
        Ok(set) => {
            check_format(label, set.format, store);
            if let Err(e) = DecisionSet::validate_id(&set.id) {
                store.schema_findings.push(Finding::schema(label, e));
            }
            if stem != set.id {
                store.schema_findings.push(Finding::schema(
                    label,
                    format!("declares id `{}` but is filed as `{stem}`", set.id),
                ));
            }
            if store.sets.iter().any(|s| s.id == set.id) {
                store
                    .schema_findings
                    .push(Finding::schema(label, format!("set `{}` is declared twice", set.id)));
            }
            store.sets.push(set);
        }
        Err(e) => store.schema_findings.push(parse_fault("set", label, &e.to_string())),
    }
}

/// Parse one change-set file's text into the store. Shared like [`take_set`].
pub(crate) fn take_log(store: &mut Store, path: PathBuf, label: &str, stem: &str, text: &str) {
    match serde_yaml::from_str::<ChangeSet>(text) {
        Ok(file) => {
            check_format(label, file.format, store);
            if stem != file.id.ulid() {
                store.schema_findings.push(Finding::schema(
                    label,
                    format!("declares id `{}` but is filed as `{stem}.yml`", file.id),
                ));
            }
            if store.log.iter().any(|c| c.file.id == file.id) {
                store.schema_findings.push(Finding::schema(
                    label,
                    format!("change-set `{}` appears twice", file.id),
                ));
            }
            for fault in file.acceptances.iter().flat_map(|a| a.schema_faults()) {
                store.schema_findings.push(fault);
            }
            store.log.push(LoggedChangeSet { path, file });
        }
        Err(e) => store.schema_findings.push(parse_fault("change-set", label, &e.to_string())),
    }
}

fn check_format(label: &str, declared: u32, store: &mut Store) {
    if !format::is_supported(declared) {
        store
            .schema_findings
            .push(Finding::schema(label, format::unsupported_message(declared)));
    }
}

/// Every `.yml`/`.yaml` file in `dir`, in a stable order. The PRD writes
/// `.yml`; `.yaml` is read too so a store hand-authored either way loads.
fn yaml_files(dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut out: Vec<PathBuf> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_file())
        .filter(|p| p.extension().is_some_and(|e| e == "yml" || e == "yaml"))
        .collect();
    out.sort();
    out
}

fn stem(path: &Path) -> String {
    path.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default()
}

fn file_label(path: &Path) -> String {
    path.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default()
}

fn parse_fault(kind: &str, label: &str, err: &str) -> Finding {
    Finding::schema(label, format!("{kind} file does not parse: {err}"))
}

#[path = "store_tests.rs"]
#[cfg(test)]
mod tests;
