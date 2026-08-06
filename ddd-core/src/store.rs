//! Store loading — walk `.ddd/`, parse every entry, collect schema faults.
//!
//! Loading never hard-fails on a bad entry: each file that does not parse
//! becomes a named schema [`Violation`] so `ddd validate` can report the whole
//! store in one pass. The predicate `status` rejection (ontology rule 1) is
//! surfaced here with its rule message, because the Turtle projection can
//! never see a field the schema already refused.

use std::path::{Path, PathBuf};

use product_core::pf::validate::Violation;

use crate::claim::Claim;
use crate::config::DddConfig;
use crate::decision::Decision;
use crate::manifest::{ManifestFile, ManifestSet};
use crate::pattern::PatternInstance;
use crate::predicate::PredicateFile;
use crate::seam::SeamDeclaration;
use crate::seam_event::{SeamEvent, EVENTS_DIR};

/// The store directory name, sibling to `.product/` where both are present.
pub const STORE_DIR: &str = ".ddd";

/// The parsed `.ddd/` store plus every schema fault met while loading it.
#[derive(Default)]
pub struct DddStore {
    /// The `.ddd/` directory this store was loaded from.
    pub dir: PathBuf,
    pub predicates: Vec<crate::predicate::Predicate>,
    pub claims: Vec<Claim>,
    pub decisions: Vec<Decision>,
    pub manifests: Vec<ManifestSet>,
    pub patterns: Vec<PatternInstance>,
    pub seams: Vec<SeamDeclaration>,
    /// Interception outcome rows (`seams/events/`, PRD §8).
    pub seam_events: Vec<SeamEvent>,
    pub config: Option<DddConfig>,
    /// Parse-level faults, one per unreadable entry, named by file.
    pub schema_violations: Vec<Violation>,
}

impl DddStore {
    /// How many entries loaded cleanly (manifest rules count individually).
    pub fn entry_count(&self) -> usize {
        self.predicates.len()
            + self.claims.len()
            + self.decisions.len()
            + self.manifests.iter().map(|m| m.file.rules.len()).sum::<usize>()
            + self.patterns.len()
            + self.seams.len()
    }
}

/// Walk upward from `start` to the first directory containing `.ddd/`.
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

/// Load every entry under the given `.ddd/` directory.
pub fn load(ddd_dir: &Path) -> DddStore {
    let mut store = DddStore { dir: ddd_dir.to_path_buf(), ..DddStore::default() };
    let v = &mut store.schema_violations;
    parse_dir::<PredicateFile>(&ddd_dir.join("predicates"), "predicate", v, |f| {
        store.predicates.push(f.predicate);
    });
    parse_dir::<Claim>(&ddd_dir.join("claims"), "claim", v, |c| store.claims.push(c));
    parse_dir::<Decision>(&ddd_dir.join("decisions"), "decision", v, |d| store.decisions.push(d));
    parse_dir::<PatternInstance>(&ddd_dir.join("patterns"), "pattern", v, |p| {
        store.patterns.push(p);
    });
    parse_dir::<SeamDeclaration>(&ddd_dir.join("seams"), "seam", v, |s| store.seams.push(s));
    parse_dir::<SeamEvent>(&ddd_dir.join(EVENTS_DIR), "seam-event", v, |e| {
        store.seam_events.push(e);
    });
    load_manifests(ddd_dir, &mut store.manifests, v);
    load_config(ddd_dir, &mut store.config, v);
    store
}

/// Parse every YAML file in `dir` as `T`, faulting unparseable ones by name.
fn parse_dir<T: serde::de::DeserializeOwned>(
    dir: &Path,
    kind: &str,
    violations: &mut Vec<Violation>,
    mut on_ok: impl FnMut(T),
) {
    for path in yaml_files(dir) {
        match read_yaml::<T>(&path) {
            Ok(v) => on_ok(v),
            Err(msg) => violations.push(parse_fault(kind, &path, &msg)),
        }
    }
}

fn load_manifests(ddd_dir: &Path, out: &mut Vec<ManifestSet>, violations: &mut Vec<Violation>) {
    for path in yaml_files(&ddd_dir.join("manifest")) {
        let name = path.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
        match read_yaml::<ManifestFile>(&path) {
            Ok(file) => out.push(ManifestSet { name, file }),
            Err(msg) => violations.push(parse_fault("manifest", &path, &msg)),
        }
    }
}

fn load_config(ddd_dir: &Path, out: &mut Option<DddConfig>, violations: &mut Vec<Violation>) {
    let path = ddd_dir.join("config.yaml");
    if !path.is_file() {
        return;
    }
    match read_yaml::<DddConfig>(&path) {
        Ok(c) => *out = Some(c),
        Err(msg) => violations.push(parse_fault("config", &path, &msg)),
    }
}

fn read_yaml<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, String> {
    let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_yaml::from_str(&text).map_err(|e| e.to_string())
}

/// Render a parse failure as a named violation; the predicate `status`
/// rejection gets its ontology-rule message (rule 1) instead of the raw
/// serde text.
fn parse_fault(kind: &str, path: &Path, err: &str) -> Violation {
    let file = path.file_name().map(|f| f.to_string_lossy().into_owned()).unwrap_or_default();
    let (path_label, message) = if kind == "predicate" && err.contains("unknown field `status`") {
        (
            "status".to_string(),
            "predicates carry no status — closure lives only in claims; file a claim instead (ontology rule 1)".to_string(),
        )
    } else {
        ("schema".to_string(), format!("{kind} entry does not parse: {err}"))
    };
    Violation { focus: file, path: path_label, message, severity: "violation".to_string() }
}

fn yaml_files(dir: &Path) -> Vec<PathBuf> {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut files: Vec<PathBuf> = rd
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.is_file()
                && p.extension().map(|e| e == "yaml" || e == "yml").unwrap_or(false)
        })
        .collect();
    files.sort();
    files
}

#[cfg(test)]
#[path = "store_tests.rs"]
mod tests;
