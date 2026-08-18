//! Fixtures for the ground-registry generator: generation, the gate, teardown.
//!
//! These are the reason the generator exists as a subcommand rather than a
//! script. Each fixture generates into its own temporary directory, which is
//! the whole of the generation's footprint.

#![allow(clippy::unwrap_used)]

mod registry_support;

use registry_support::{bin, g0_args, generate, read};

#[test]
fn parameters_round_trip_byte_identically() {
    let inst = generate(&g0_args());
    let generation = inst.read("GENERATION.ttl");
    for value in [
        "Hafeok",
        "ground-registry-g0",
        "emil@okkels-klein.dk",
        "Ground Registry (G0 validation)",
        "tag:emil@okkels-klein.dk,2026-08-17:ground/",
        "2026-08-17",
    ] {
        assert!(generation.contains(value), "{value:?} did not reach the birth provenance");
    }
}

/// The G0 handoff, pinned: the instance the mechanism exists to mint, with the
/// exact five parameters and the mint date that do not move when it regenerates.
#[test]
fn the_g0_handoff_parameters_mint_the_handed_over_instance() {
    let inst = generate(&g0_args());
    let generation = inst.read("GENERATION.ttl");
    assert!(
        generation.contains("@prefix reg:  <tag:emil@okkels-klein.dk,2026-08-17:ground/> ."),
        "the base IRI must be byte-for-byte the handed-over value"
    );
    assert!(generation.contains("reg:baseIri           \"tag:emil@okkels-klein.dk,2026-08-17:ground/\" ;"));
    assert!(generation.contains("reg:mintDate          \"2026-08-17\"^^xsd:date ;"));
    for file in ["shapes/reading.ttl", "shapes/structural.ttl", "graphs/canonical/_exemplar.ttl"] {
        assert!(
            inst.read(file).contains("tag:emil@okkels-klein.dk,2026-08-17:ground/"),
            "{file} did not receive the base IRI"
        );
    }
}

/// The G0 bug's exact case: the ratifier's address carried a sigil the shell
/// one-liner re-read as code, and the corrupted base IRI reached every file.
#[test]
fn g0_regression_a_sigil_in_the_ratifiers_address_survives() {
    let inst = generate(&g0_args());
    assert!(inst.read("README.md").contains("emil@okkels-klein.dk"));
    assert!(!inst.read("README.md").contains("{{RATIFIER}}"));
    let base = "tag:emil@okkels-klein.dk,2026-08-17:ground/";
    for file in inst.files() {
        let text = inst.read(&file);
        if file != "TEMPLATE.md" && text.contains("tag:") {
            assert!(text.contains(base), "{file} carries a mangled base IRI");
        }
    }
}

#[test]
fn hostile_parameter_values_are_data() {
    let hostile = "R&D \\1 $0 | / \\ {{OWNER_ORG}} `id` $(whoami)";
    let mut args = g0_args();
    args.display_name = hostile.to_string();
    let inst = generate(&args);
    assert!(inst.read("GENERATION.ttl").contains(hostile), "a hostile value must be carried verbatim");
    assert!(inst.read("README.md").contains(hostile));
}

#[test]
fn no_placeholder_survives_the_generated_tree() {
    let inst = generate(&g0_args());
    for file in inst.files() {
        if file == "TEMPLATE.md" {
            continue;
        }
        let text = inst.read(&file);
        assert!(!text.contains("REGISTRY-HOST.example"), "{file} kept the placeholder host");
        assert!(
            !text.contains("{{") || !text.contains("}}"),
            "{file} kept a placeholder"
        );
    }
}

#[test]
fn template_md_travels_verbatim() {
    let inst = generate(&g0_args());
    let shipped = inst.read("TEMPLATE.md");
    let source = read("docs/g-track/registry-template/TEMPLATE.md");
    assert_eq!(shipped, source, "TEMPLATE.md is the instance's record of its own placeholders");
    assert!(shipped.contains("{{OWNER_ORG}}"));
}

#[test]
fn the_birth_provenance_is_complete_as_rdf() {
    let inst = generate(&g0_args());
    let ttl = inst.read("GENERATION.ttl");
    let base = "tag:emil@okkels-klein.dk,2026-08-17:ground/";
    let rows = product_core::pf::sparql_rules::select(
        &ttl,
        &format!(
            "SELECT ?owner ?repo ?ratifier ?name ?date ?tv ?gv ?agent WHERE {{
               ?i <{base}owner> ?owner ; <{base}repository> ?repo ; <{base}ratifier> ?ratifier ;
                  <{base}displayName> ?name ; <{base}mintDate> ?date ;
                  <{base}templateVersion> ?tv ; <{base}generatorVersion> ?gv ;
                  <http://www.w3.org/ns/prov#wasAttributedTo> ?a .
               ?a <{base}name> ?agent . }}"
        ),
    )
    .expect("the birth provenance parses as Turtle");
    assert_eq!(rows.len(), 1, "one registry node, fully described: {rows:?}");
    let row = &rows[0];
    assert_eq!(row["owner"], "\"Hafeok\"");
    assert_eq!(row["tv"], "\"0.1.0\"");
    assert!(row["gv"].starts_with('"'), "the generator's own version is recorded");
    assert!(row["agent"].contains("fixture"), "generated-by is recorded: {}", row["agent"]);
}

#[test]
fn a_fresh_instance_passes_its_own_rules() {
    let inst = generate(&g0_args());
    let out = inst.check();
    assert_eq!(out.exit_code, 0, "a freshly generated instance must be conformant:\n{}{}", out.stdout, out.stderr);
    assert!(out.stdout.contains("conformant"), "{}", out.stdout);
}

#[test]
fn generation_is_deterministic() {
    let first = generate(&g0_args());
    let second = generate(&g0_args());
    assert_eq!(first.files(), second.files());
    for file in first.files() {
        assert_eq!(first.read(&file), second.read(&file), "{file} differs between generations");
    }
    assert_eq!(first.commit(), second.commit(), "the birth commit depends on the parameters alone");
}

#[test]
fn two_generations_in_one_run_do_not_interfere() {
    let first = generate(&g0_args());
    let mut other = g0_args();
    other.owner = "Other".into();
    other.repo = "other-registry".into();
    other.base_iri = "https://other.test/ns#".into();
    let second = generate(&other);
    assert!(first.read("GENERATION.ttl").contains("\"Hafeok\""));
    assert!(second.read("GENERATION.ttl").contains("\"Other\""));
    assert!(!second.read("GENERATION.ttl").contains("Hafeok"));
    assert_ne!(first.commit(), second.commit());
}

#[test]
fn generation_leaves_no_residue_outside_its_target() {
    let inst = generate(&g0_args());
    let parent: Vec<String> = std::fs::read_dir(inst.parent())
        .unwrap()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();
    assert_eq!(parent, vec!["instance".to_string()], "the target is the whole footprint");
    let path = inst.path().to_path_buf();
    drop(inst);
    assert!(!path.exists(), "teardown must leave nothing behind");
}

#[test]
fn a_refused_parameter_writes_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("instance");
    let mut args = g0_args();
    args.base_iri = "https://example.test/ns".into(); // no terminator
    let result = args.run(&out);
    assert_ne!(result.exit_code, 0, "a bad parameter must refuse");
    assert!(result.stderr.contains("--base-iri"), "{}", result.stderr);
    assert!(!out.exists(), "the gate refuses before anything reaches disk");
}

#[test]
fn generation_refuses_a_non_empty_target() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("instance");
    std::fs::create_dir_all(&out).unwrap();
    std::fs::write(out.join("existing.txt"), "prior content").unwrap();
    let result = g0_args().run(&out);
    assert_ne!(result.exit_code, 0);
    assert!(result.stderr.contains("not empty"), "{}", result.stderr);
    assert_eq!(std::fs::read_to_string(out.join("existing.txt")).unwrap(), "prior content");
}

#[test]
fn generation_configures_no_remote() {
    let inst = generate(&g0_args());
    let remotes = inst.git(&["remote", "-v"]);
    assert!(remotes.trim().is_empty(), "publishing is a separate act: {remotes}");
    assert!(inst.stdout().contains("nothing has been published"));
    assert!(inst.stdout().contains("git -C"), "the publish commands are printed, never run");
}

#[test]
fn the_binary_is_where_the_fixtures_expect_it() {
    assert!(bin().exists(), "the fixtures drive the built `product` binary");
}

#[test]
fn a_broken_file_rule_fails_the_check_in_its_own_section() {
    let inst = generate(&g0_args());
    std::fs::write(
        inst.path().join("graphs/canonical/two-assertions.ttl"),
        "@prefix reg: <tag:emil@okkels-klein.dk,2026-08-17:ground/> .\n\
         reg:a-1 a reg:Assertion .\nreg:a-2 a reg:Assertion .\n",
    )
    .unwrap();
    let out = inst.check();
    assert_ne!(out.exit_code, 0);
    let text = format!("{}{}", out.stdout, out.stderr);
    assert!(text.contains("file rule:"), "{text}");
    assert!(text.contains("carries 2 assertion/decision typings"), "{text}");
}

/// A shape the reader cannot read fails the check, and says so distinctly from
/// data that violates a shape — an unreadable shape is not a confusing red.
#[test]
fn an_unreadable_shape_fails_closed_with_its_own_heading() {
    let inst = generate(&g0_args());
    std::fs::write(
        inst.path().join("shapes/exotic.ttl"),
        "@prefix sh: <http://www.w3.org/ns/shacl#> .\n\
         @prefix reg: <tag:emil@okkels-klein.dk,2026-08-17:ground/> .\n\
         reg:ExoticShape a sh:NodeShape ; sh:targetClass reg:Reading ;\n\
           sh:property [ sh:path reg:value ; sh:qualifiedValueShape reg:Other ] .\n",
    )
    .unwrap();
    let out = inst.check();
    assert_ne!(out.exit_code, 0, "an unreadable shape must fail the check");
    let text = format!("{}{}", out.stdout, out.stderr);
    assert!(text.contains("unreadable shape"), "{text}");
    assert!(text.contains("no data was judged against these"), "{text}");
    assert!(text.contains("sh:qualifiedValueShape"), "{text}");
    assert!(!text.contains("shape violations:"), "no data violated anything here:\n{text}");
}
