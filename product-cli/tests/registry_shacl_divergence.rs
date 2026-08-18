//! Measuring the native shapes reader against pySHACL over the same cases.
//!
//! The instance's own CI runs `pyshacl`; the fixtures run a fail-closed native
//! reader, because a gate that depends on a Python toolchain is a gate that
//! quietly becomes a habit. Two readers of one rule set is a standing finding,
//! so it is measured rather than assumed small: this fixture runs both over the
//! same cases whenever pySHACL is present, failing on any disagreement.
//!
//! When pySHACL is absent the fixture says so and measures nothing — it is the
//! divergence detector, never the gate.

#![allow(clippy::unwrap_used)]

mod registry_support;

use registry_support::{g0_args, generate};

const BASE: &str = "tag:emil@okkels-klein.dk,2026-08-17:ground/";

fn pyshacl_available() -> bool {
    std::process::Command::new("python3")
        .args(["-c", "import pyshacl"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// pySHACL's verdict: does the data conform to the shapes?
fn pyshacl_conforms(dir: &std::path::Path, data: &str, shapes: &str) -> bool {
    std::fs::write(dir.join("data.ttl"), data).unwrap();
    std::fs::write(dir.join("shapes.ttl"), shapes).unwrap();
    let out = std::process::Command::new("python3")
        .arg("-m")
        .arg("pyshacl")
        .arg("-a")
        .arg("-s")
        .arg(dir.join("shapes.ttl"))
        .arg(dir.join("data.ttl"))
        .output()
        .expect("run pyshacl");
    String::from_utf8_lossy(&out.stdout).contains("Conforms: True")
}

fn reading(extra: &str) -> String {
    format!(
        "@prefix reg: <{BASE}> .\n@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n\
         reg:a-1 a reg:Assertion , reg:Reading ;\n\
             reg:subject reg:Order ; reg:predicate reg:hasPart ; reg:object reg:OrderLine ;\n\
             reg:value \"Order hasPart OrderLine\" ;\n\
             reg:asOf \"2026-08-17T00:00:00Z\"^^xsd:dateTime ;\n\
             reg:assurance \"hand-authored\" ;\n{extra} .\n"
    )
}

#[test]
fn the_native_reader_agrees_with_pyshacl() {
    if !pyshacl_available() {
        eprintln!(
            "pySHACL is not installed — the native reader's divergence from the instance's own \
             CI was NOT measured in this run. Install `pyshacl` to measure it."
        );
        return;
    }
    let inst = generate(&g0_args());
    let shapes = format!("{}\n{}", inst.read("shapes/reading.ttl"), inst.read("shapes/structural.ttl"));
    let scratch = tempfile::tempdir().unwrap();

    let cases: Vec<(&str, String)> = vec![
        ("shipped exemplar", inst.read("graphs/canonical/_exemplar.ttl")),
        ("conforming reading", reading("    reg:provenance reg:controlled")),
        ("institutional without trust decision", reading("    reg:provenance reg:institutional")),
        (
            "institutional with trust decision",
            reading("    reg:provenance reg:institutional ;\n    reg:trust_decision reg:td-1"),
        ),
        ("provenance outside the vocabulary", reading("    reg:provenance reg:hearsay")),
        (
            "assertion without a predicate",
            format!(
                "@prefix reg: <{BASE}> .\nreg:a-2 a reg:Assertion ; reg:subject reg:Order ; reg:object reg:OrderLine .\n"
            ),
        ),
    ];

    let mut divergence = Vec::new();
    for (name, data) in &cases {
        let (findings, evaluated) = product_core::registry::evaluate(data, &shapes).expect("native reader");
        assert!(evaluated > 0, "the native reader evaluated no constraint for '{name}'");
        let native = findings.is_empty();
        let python = pyshacl_conforms(scratch.path(), data, &shapes);
        if native != python {
            divergence.push(format!(
                "  {name}: native reader says {}, pySHACL says {}\n    findings: {:?}",
                verdict(native),
                verdict(python),
                findings.iter().map(|f| f.message.clone()).collect::<Vec<_>>()
            ));
        }
    }
    assert!(
        divergence.is_empty(),
        "the two readers of one rule set disagree — the divergence is real, not assumed small:\n{}",
        divergence.join("\n")
    );
}

fn verdict(conforms: bool) -> &'static str {
    if conforms {
        "conformant"
    } else {
        "violating"
    }
}
