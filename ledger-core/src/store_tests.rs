//! Store-loading cases: whole-store reporting, filing faults, format gates.

use crate::testkit;

use super::*;

struct Repo {
    dir: tempfile::TempDir,
}

impl Repo {
    fn new() -> Self {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(dir.path().join(".decisions/sets")).expect("mkdir");
        std::fs::create_dir_all(dir.path().join(".decisions/log")).expect("mkdir");
        Self { dir }
    }

    fn path(&self) -> &Path {
        self.dir.path()
    }

    fn write(&self, relative: &str, body: &str) {
        let path = self.dir.path().join(".decisions").join(relative);
        std::fs::write(path, body).expect("write");
    }

    fn write_set(&self) {
        let text = serde_yaml::to_string(&testkit::set()).expect("serialize");
        self.write("sets/ledger-design.yml", &text);
    }

    fn write_log(&self) {
        let cs = testkit::changeset(vec![testkit::sealed(testkit::version())], Vec::new());
        let text = serde_yaml::to_string(&cs).expect("serialize");
        self.write(&format!("log/{}.yml", testkit::CS_ULID), &text);
    }

    fn load(&self) -> Store {
        load(self.path())
    }
}

#[test]
fn a_well_formed_store_loads_with_no_faults() {
    let repo = Repo::new();
    repo.write_set();
    repo.write_log();
    let store = repo.load();
    assert!(store.schema_findings.is_empty(), "{:?}", store.schema_findings);
    assert_eq!(store.sets.len(), 1);
    assert_eq!(store.log.len(), 1);
    assert_eq!(store.entry_count(), 3, "one set, one decision, one version");
    assert!(store.set("ledger-design").is_some());
}

#[test]
fn an_absent_store_loads_empty_rather_than_failing() {
    let dir = tempfile::tempdir().expect("tempdir");
    let store = load(dir.path());
    assert!(store.sets.is_empty() && store.log.is_empty());
    assert!(store.schema_findings.is_empty());
}

#[test]
fn one_unparseable_file_does_not_stop_the_others() {
    let repo = Repo::new();
    repo.write_set();
    repo.write_log();
    repo.write("log/01K2C4YQJ3F8M0PT5W7NZ9RDXZ.yml", "format: 1\nid: not-a-change-set-id\n");
    let store = repo.load();
    assert_eq!(store.log.len(), 1, "the good file still loaded");
    assert_eq!(store.schema_findings.len(), 1);
    assert!(store.schema_findings[0].message.contains("does not parse"), "{:?}", store.schema_findings);
}

#[test]
fn a_file_whose_name_disagrees_with_its_id_is_a_fault() {
    let repo = Repo::new();
    repo.write_set();
    let cs = testkit::changeset(Vec::new(), Vec::new());
    let text = serde_yaml::to_string(&cs).expect("serialize");
    repo.write("log/01K2C4YQJ3F8M0PT5W7NZ9RDXZ.yml", &text);
    let store = repo.load();
    assert!(
        store.schema_findings.iter().any(|f| f.message.contains("but is filed as")),
        "{:?}",
        store.schema_findings
    );
}

#[test]
fn a_set_filed_under_the_wrong_name_is_a_fault() {
    let repo = Repo::new();
    let text = serde_yaml::to_string(&testkit::set()).expect("serialize");
    repo.write("sets/other-name.yml", &text);
    let store = repo.load();
    assert!(store.schema_findings.iter().any(|f| f.message.contains("but is filed as")));
}

#[test]
fn an_unknown_format_is_named_rather_than_assumed() {
    let repo = Repo::new();
    let mut set = testkit::set();
    set.format = 7;
    let text = serde_yaml::to_string(&set).expect("serialize");
    repo.write("sets/ledger-design.yml", &text);
    let store = repo.load();
    assert!(
        store.schema_findings.iter().any(|f| f.message.contains("declares format 7")),
        "{:?}",
        store.schema_findings
    );
}

#[test]
fn a_reserved_signature_in_a_log_file_is_caught_at_load() {
    let repo = Repo::new();
    repo.write_set();
    let sealed = testkit::sealed(testkit::version());
    let mut acceptance = testkit::acceptance(&sealed);
    acceptance.signature = "sig".into();
    let cs = testkit::changeset(vec![sealed], vec![acceptance]);
    let text = serde_yaml::to_string(&cs).expect("serialize");
    repo.write(&format!("log/{}.yml", testkit::CS_ULID), &text);
    let store = repo.load();
    assert!(store.schema_findings.iter().any(|f| f.message.contains("reserved")));
}

#[test]
fn both_yaml_extensions_are_read() {
    let repo = Repo::new();
    let text = serde_yaml::to_string(&testkit::set()).expect("serialize");
    repo.write("sets/ledger-design.yaml", &text);
    assert_eq!(repo.load().sets.len(), 1);
}

#[test]
fn the_root_is_found_by_walking_up() {
    let repo = Repo::new();
    let nested = repo.path().join("a/b/c");
    std::fs::create_dir_all(&nested).expect("mkdir");
    assert_eq!(find_root(&nested).as_deref(), Some(repo.path()));
}
