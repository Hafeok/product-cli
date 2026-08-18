//! File discovery, instance reading, and the already-ratified id set.

use super::*;

fn write(dir: &Path, rel: &str, body: &str) {
    let path = dir.join(rel);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("mkdir");
    }
    std::fs::write(path, body).expect("write");
}

#[test]
fn source_discovery_skips_build_output_and_sorts() {
    let tmp = tempfile::tempdir().expect("tmp");
    write(tmp.path(), "src/B.cs", "");
    write(tmp.path(), "src/A.cs", "");
    write(tmp.path(), "src/obj/Generated.cs", "");
    write(tmp.path(), "src/notes.md", "");
    let found = source_files(&[tmp.path().to_path_buf()], &["cs"]);
    let names: Vec<String> =
        found.iter().filter_map(|p| p.file_name()?.to_str().map(str::to_string)).collect();
    assert_eq!(names, ["A.cs", "B.cs"], "sorted, and no obj/ output");
}

/// The instance states its own base IRI; the reader never guesses one.
#[test]
fn the_base_iri_comes_from_the_instances_own_birth_provenance() {
    let tmp = tempfile::tempdir().expect("tmp");
    write(
        tmp.path(),
        "GENERATION.ttl",
        "# birth\n@prefix reg:  <tag:someone@example,2026-08-17:ground/> .\n",
    );
    assert_eq!(
        instance_base(tmp.path()).expect("base"),
        "tag:someone@example,2026-08-17:ground/"
    );
}

#[test]
fn a_missing_or_silent_generation_file_is_an_error_not_a_default() {
    let tmp = tempfile::tempdir().expect("tmp");
    assert!(instance_base(tmp.path()).is_err());
    write(tmp.path(), "GENERATION.ttl", "# nothing declared\n");
    assert!(instance_base(tmp.path()).is_err());
}

#[test]
fn shapes_are_concatenated_in_a_stable_order() {
    let tmp = tempfile::tempdir().expect("tmp");
    write(tmp.path(), "shapes/b.ttl", "# second\n");
    write(tmp.path(), "shapes/a.ttl", "# first\n");
    write(tmp.path(), "shapes/README.md", "# ignored\n");
    let text = shapes_of(tmp.path()).expect("shapes");
    assert!(text.find("# first").expect("first") < text.find("# second").expect("second"));
    assert!(!text.contains("ignored"));
}

/// Identity is content-derived and carried by the file name, so the ratified
/// set is read off the directory with no index to keep in step.
#[test]
fn the_ratified_set_is_read_off_the_file_names() {
    let tmp = tempfile::tempdir().expect("tmp");
    write(tmp.path(), "graphs/canonical/assertion-aabbccdd11223344.ttl", "");
    write(tmp.path(), "graphs/canonical/founding-decision.ttl", "");
    write(tmp.path(), "graphs/canonical/_exemplar.ttl", "");
    let ids = ratified_ids(&tmp.path().join("graphs/canonical"));
    assert_eq!(ids.len(), 1);
    assert!(ids.contains("aabbccdd11223344"));
}

#[test]
fn a_missing_graph_directory_is_an_empty_set_not_a_failure() {
    let tmp = tempfile::tempdir().expect("tmp");
    assert!(ratified_ids(&tmp.path().join("graphs/canonical")).is_empty());
}
