//! Pair contract check coverage: both directions, thresholds, enrichment.

use super::*;
use ddd_core::config::{DddConfig, PairUnit};

fn write(root: &Path, rel: &str, content: &str) {
    let path = root.join(rel);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("mkdir");
    }
    std::fs::write(path, content).expect("write");
}

fn config(ignore_classes: &[&str]) -> DddConfig {
    DddConfig {
        format: 4,
        pair: Some(PairConfig {
            units: vec![PairUnit {
                html: vec!["site/*.html".into()],
                css: vec!["site/style.css".into()],
            }],
            ignore_classes: ignore_classes.iter().map(|s| s.to_string()).collect(),
        }),
        ..DddConfig::default()
    }
}

const PAGE: &str = "<html><head><link rel=\"stylesheet\" href=\"style.css\"></head>\n<body id=\"top\" class=\"page\">\n<div class=\"card\"><button class=\"btn js-toggle\">go</button></div>\n</body></html>\n";
const SHEET: &str = ".page { margin: 0; }\n.card { padding: 1rem; }\n.btn { color: red; }\n";

#[test]
fn a_conformant_pair_is_clean_and_states_its_coverage() {
    let tmp = tempfile::tempdir().expect("tmp");
    write(tmp.path(), "site/page.html", PAGE);
    write(tmp.path(), "site/style.css", SHEET);
    let report = check_pairs(tmp.path(), &config(&["js-*"]));
    assert!(report.is_clean(), "{report:?}");
    assert_eq!(report.coverage.units, 1);
    assert_eq!(report.coverage.html_files, 1);
    assert_eq!(report.coverage.css_files, 1);
    assert!(report.coverage.selectors_checked >= 3, "{report:?}");
}

#[test]
fn an_orphan_class_is_found_unless_a_decorative_glob_exempts_it() {
    let tmp = tempfile::tempdir().expect("tmp");
    write(tmp.path(), "site/page.html", PAGE);
    write(tmp.path(), "site/style.css", ".page { margin: 0; }\n.card { padding: 1rem; }\n");
    let report = check_pairs(tmp.path(), &config(&["js-*"]));
    let orphans: Vec<&str> = report
        .findings
        .iter()
        .filter(|f| f.kind == "orphan-class")
        .map(|f| f.name.as_str())
        .collect();
    assert_eq!(orphans, vec!["btn"], "{report:?}");
    // Without the decorative threshold the js-hook joins them.
    let report = check_pairs(tmp.path(), &config(&[]));
    let orphans: Vec<&str> = report
        .findings
        .iter()
        .filter(|f| f.kind == "orphan-class")
        .map(|f| f.name.as_str())
        .collect();
    assert_eq!(orphans, vec!["btn", "js-toggle"], "{report:?}");
}

#[test]
fn a_dead_selector_is_found_in_class_element_and_id_form() {
    let tmp = tempfile::tempdir().expect("tmp");
    write(tmp.path(), "site/page.html", PAGE);
    write(
        tmp.path(),
        "site/style.css",
        format!("{SHEET}.ghost {{ color: blue; }}\ntable {{ margin: 0; }}\n#nowhere {{ color: red; }}\ndiv.card {{ border: 0; }}\n").as_str(),
    );
    let report = check_pairs(tmp.path(), &config(&["js-*"]));
    let dead: Vec<&str> = report
        .findings
        .iter()
        .filter(|f| f.kind == "dead-selector")
        .map(|f| f.name.as_str())
        .collect();
    assert_eq!(dead, vec!["#nowhere", ".ghost", "table"], "{report:?}");
}

#[test]
fn unmodelled_selectors_are_reported_as_skipped_not_passed() {
    let tmp = tempfile::tempdir().expect("tmp");
    write(tmp.path(), "site/page.html", PAGE);
    write(tmp.path(), "site/style.css", format!("{SHEET}[data-x] {{ color: red; }}\n*:hover {{ color: blue; }}\n").as_str());
    let report = check_pairs(tmp.path(), &config(&["js-*"]));
    assert!(report.is_clean(), "{report:?}");
    assert_eq!(report.coverage.selectors_skipped.len(), 2, "{report:?}");
}

#[test]
fn pseudo_classes_are_stripped_and_compounds_check_each_part() {
    let tmp = tempfile::tempdir().expect("tmp");
    write(tmp.path(), "site/page.html", PAGE);
    write(tmp.path(), "site/style.css", format!("{SHEET}.btn:hover {{ color: blue; }}\n.card > .btn {{ margin: 0; }}\n").as_str());
    let report = check_pairs(tmp.path(), &config(&["js-*"]));
    assert!(report.is_clean(), "{report:?}");
}

#[test]
fn enrichment_stamps_direction_and_counterpart_and_drops_decorative_classes() {
    let tmp = tempfile::tempdir().expect("tmp");
    write(tmp.path(), "site/page.html", PAGE);
    write(tmp.path(), "site/style.css", SHEET);
    let cfg = config(&["js-*"]);
    let before = (super::super::htmlcss::ADAPTER.facts)(SHEET, &[], &Default::default());
    let after_text = format!("{SHEET}.js-only {{ color: red; }}\n.real {{ color: red; }}\n");
    let after = (super::super::htmlcss::ADAPTER.facts)(&after_text, &[], &Default::default());
    let mut events = crate::classify::diff_facts(
        &(super::super::htmlcss::ADAPTER.policy)(&Default::default()),
        super::super::htmlcss::ADAPTER.visibility_rank,
        before,
        after,
    );
    enrich(tmp.path(), &tmp.path().join("site/style.css"), &cfg, &mut events);
    let names: Vec<&str> = events.iter().map(|e| e.facts.name.as_str()).collect();
    assert_eq!(names, vec!["real"], "decorative dropped: {events:?}");
    let extra = &events[0].facts.extra;
    assert_eq!(extra.get("pair_direction").map(String::as_str), Some("definition"));
    assert_eq!(extra.get("pair_counterpart").map(String::as_str), Some("site/page.html"));
}
