//! Unit tests for the generation gate.

use super::*;
use crate::registry::substitute::render;

fn tokens<'a>(value: &'a str) -> Vec<Token<'a>> {
    vec![Token { label: "--owner", token: "{{OWNER_ORG}}", value }]
}

fn gated<'a>(path: &'a str, source: &'a str, rendered: &'a str, sites: &'a [Site]) -> Gated<'a> {
    Gated { path, source, rendered, sites, substituted: true }
}

#[test]
fn a_clean_render_passes() {
    let source = "owner: {{OWNER_ORG}}\n";
    let t = tokens("Hafeok");
    let out = render(source, &t);
    let report = gate(&[gated("README.md", source, &out.text, &out.sites)], &t).expect("gate");
    assert_eq!(report.sites_per_label["--owner"], 1);
}

#[test]
fn a_mangled_value_is_caught() {
    let source = "owner: {{OWNER_ORG}}\n";
    let t = tokens("tag:emil@okkels-klein.dk,2026-08-17:ground/");
    let out = render(source, &t);
    // Simulate the G0 failure: the value in the output is not the value supplied.
    let corrupted = out.text.replace("okkels-klein.dk", "okkels-klein.dkokkels-klein.dk");
    let err = gate(&[gated("README.md", source, &corrupted, &out.sites)], &t)
        .expect_err("a corrupted value must abort generation");
    assert!(err.to_string().contains("did not round-trip"), "{err}");
}

#[test]
fn a_surviving_placeholder_is_caught() {
    let source = "owner: {{OWNER_ORG}} ratifier: {{RATIFIER}}\n";
    let t = tokens("Hafeok");
    let out = render(source, &t);
    let err = gate(&[gated("README.md", source, &out.text, &out.sites)], &t)
        .expect_err("an unwired placeholder must abort generation");
    assert!(err.to_string().contains("{{RATIFIER}}"), "{err}");
}

#[test]
fn a_surviving_host_sentinel_is_caught() {
    let source = "prefix: {{OWNER_ORG}} <https://REGISTRY-HOST.example/agent/x>\n";
    let t = tokens("Hafeok");
    let out = render(source, &t);
    let err = gate(&[gated("x.ttl", source, &out.text, &out.sites)], &t)
        .expect_err("a surviving base-IRI host must abort generation");
    assert!(err.to_string().contains("REGISTRY-HOST.example"), "{err}");
}

#[test]
fn a_parameter_that_reaches_nothing_is_caught() {
    let source = "nothing to substitute\n";
    let t = tokens("Hafeok");
    let out = render(source, &t);
    let err = gate(&[gated("README.md", source, &out.text, &out.sites)], &t)
        .expect_err("a parameter landing nowhere cannot be verified");
    assert!(err.to_string().contains("reaches no file"), "{err}");
}

#[test]
fn an_edited_verbatim_file_is_caught() {
    let source = "travels verbatim {{OWNER_ORG}}\n";
    let t = tokens("Hafeok");
    let subst = render("owner {{OWNER_ORG}}", &t);
    let files = [
        gated("README.md", "owner {{OWNER_ORG}}", &subst.text, &subst.sites),
        Gated { path: "TEMPLATE.md", source, rendered: "edited", sites: &[], substituted: false },
    ];
    let err = gate(&files, &t).expect_err("a verbatim file must travel unchanged");
    assert!(err.to_string().contains("verbatim"), "{err}");
}

#[test]
fn a_value_containing_a_placeholder_is_data() {
    // Check B reads the template text that survived rendering, not the values
    // written into it: a display name may honestly carry `{{RATIFIER}}`.
    let source = "owner: {{OWNER_ORG}}\n";
    let t = tokens("{{RATIFIER}}");
    let out = render(source, &t);
    gate(&[gated("README.md", source, &out.text, &out.sites)], &t)
        .expect("a parameter value is data, never a further placeholder");
}

#[test]
fn a_value_carrying_the_sentinel_host_is_data() {
    let source = "@prefix reg: <https://REGISTRY-HOST.example/ns#> .\n{{OWNER_ORG}}\n";
    let t = vec![
        Token { label: "--owner", token: "{{OWNER_ORG}}", value: "Hafeok" },
        Token {
            label: "--base-iri",
            token: "https://REGISTRY-HOST.example/ns#",
            value: "https://REGISTRY-HOST.example.test/ns#",
        },
    ];
    let out = render(source, &t);
    gate(&[gated("x.ttl", source, &out.text, &out.sites)], &t)
        .expect("a supplied base IRI is the parameter, however it reads");
}

#[test]
fn placeholder_scan_ignores_non_placeholder_braces() {
    assert_eq!(first_placeholder("{{ not a placeholder }} {{lower}}"), None);
    assert_eq!(first_placeholder("x {{OWNER_ORG}} y").as_deref(), Some("{{OWNER_ORG}}"));
}
