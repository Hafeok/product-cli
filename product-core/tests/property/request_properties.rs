//! Request-model property invariants (TC-P012, TC-P013, TC-P014 / TC-548..TC-550).
//!
//! These properties are part of FT-043's deliverable and were introduced
//! by the ADR-018 amendment. They exercise `product request apply` as a
//! black box — the test spawns the binary per case — so default case
//! counts are modest. Override with `PROPTEST_CASES=<n>` at runtime when
//! a thorough sweep is wanted (e.g. CI or release validation).
//!
//! The harness here is a minimal in-file reimplementation of the
//! session-style repo bootstrap, kept separate from `tests/sessions/` so
//! proptest can own a fresh repo per case without the session-level
//! ergonomics.

use proptest::prelude::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const CONFIG_TOML: &str = r#"name = "property-test"
schema-version = "1"
[paths]
features = "docs/features"
adrs = "docs/adrs"
tests = "docs/tests"
graph = "docs/graph"
checklist = "docs/checklist.md"
dependencies = "docs/dependencies"
[prefixes]
feature = "FT"
adr = "ADR"
test = "TC"
dependency = "DEP"
[mcp]
write = true
[domains]
api = "api"
security = "security"
networking = "networking"
storage = "storage"
"#;

fn find_binary() -> PathBuf {
    if let Some(bin) = option_env!("CARGO_BIN_EXE_product") {
        let p = PathBuf::from(bin);
        if p.exists() {
            return p;
        }
    }
    if let Ok(exe) = std::env::current_exe() {
        let mut p = exe.clone();
        p.pop();
        p.pop();
        p.push("product");
        if p.exists() {
            return p;
        }
    }
    PathBuf::from("target/debug/product")
}

fn fresh_repo() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(dir.path().join("product.toml"), CONFIG_TOML).expect("write toml");
    for sub in [
        "docs/features",
        "docs/adrs",
        "docs/tests",
        "docs/dependencies",
        "docs/graph",
    ] {
        std::fs::create_dir_all(dir.path().join(sub)).expect("mkdir");
    }
    dir
}

fn apply(bin: &Path, dir: &Path, yaml: &str) -> (i32, String, String) {
    let path = dir.join("r.yaml");
    std::fs::write(&path, yaml).expect("write");
    let out = Command::new(bin)
        .args(["--format", "json", "request", "apply", "r.yaml"])
        .current_dir(dir)
        .stdin(Stdio::null())
        .output()
        .expect("spawn");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

fn docs_digest(root: &Path) -> HashMap<String, String> {
    use sha2::{Digest, Sha256};
    let mut out = HashMap::new();
    let docs = root.join("docs");
    let mut stack = vec![docs.clone()];
    while let Some(dir) = stack.pop() {
        let entries = match std::fs::read_dir(&dir) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
            } else if p.is_file() {
                let rel = p.strip_prefix(&docs).unwrap_or(&p).to_string_lossy().to_string();
                let bytes = std::fs::read(&p).unwrap_or_default();
                let mut h = Sha256::new();
                h.update(&bytes);
                let digest = h.finalize();
                let hex: String = digest.iter().map(|b| format!("{:02x}", b)).collect();
                out.insert(rel, hex);
            }
        }
    }
    out
}

fn parse_id_map(stdout: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let start = stdout.find('{').unwrap_or(0);
    let slice = &stdout[start..];
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(slice) {
        if let Some(arr) = v.get("created").and_then(|x| x.as_array()) {
            for c in arr {
                let ref_name = c
                    .get("ref_name")
                    .and_then(|x| x.as_str())
                    .map(|s| s.to_string());
                let id = c
                    .get("id")
                    .and_then(|x| x.as_str())
                    .unwrap_or("")
                    .to_string();
                if let Some(n) = ref_name {
                    map.insert(n, id);
                }
            }
        }
    }
    map
}

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

fn arb_title() -> impl Strategy<Value = String> {
    "[A-Za-z][a-z ]{2,20}".prop_map(|s| s.trim().to_string())
}

#[derive(Debug, Clone)]
enum InvalidKind {
    EmptyReason,
    UnknownDomain,
    UnresolvedRef,
    DepWithoutAdr,
}

fn arb_invalid_request() -> impl Strategy<Value = (String, InvalidKind)> {
    prop_oneof![
        arb_title().prop_map(|t| (format!(
            "type: create\nschema-version: 1\nreason: \"\"\nartifacts:\n  - type: feature\n    title: {}\n    phase: 1\n    domains: [api]\n",
            t
        ), InvalidKind::EmptyReason)),
        arb_title().prop_map(|t| (format!(
            "type: create\nschema-version: 1\nreason: \"invalid domain\"\nartifacts:\n  - type: feature\n    title: {}\n    phase: 1\n    domains: [not-a-real-domain]\n",
            t
        ), InvalidKind::UnknownDomain)),
        arb_title().prop_map(|t| (format!(
            "type: create\nschema-version: 1\nreason: \"unresolved ref\"\nartifacts:\n  - type: feature\n    ref: ft-a\n    title: {}\n    phase: 1\n    domains: [api]\n    adrs: [ref:does-not-exist]\n",
            t
        ), InvalidKind::UnresolvedRef)),
        arb_title().prop_map(|t| (format!(
            "type: create\nschema-version: 1\nreason: \"dep without ADR\"\nartifacts:\n  - type: dep\n    title: {}\n    dep-type: library\n    version: \">=1\"\n",
            t
        ), InvalidKind::DepWithoutAdr)),
    ]
}

fn arb_domain_list() -> impl Strategy<Value = Vec<String>> {
    let d = prop_oneof![
        Just("api".to_string()),
        Just("security".to_string()),
        Just("networking".to_string()),
        Just("storage".to_string()),
    ];
    prop::collection::vec(d, 1..3)
}

fn arb_create_request() -> impl Strategy<Value = String> {
    (arb_title(), arb_title(), arb_domain_list()).prop_map(|(ft_title, adr_title, domains)| {
        let d = domains.join(", ");
        format!(
            r#"type: create
schema-version: 1
reason: "property-generated create"
artifacts:
  - type: adr
    ref: adr-g
    title: {}
    domains: [{}]
    scope: domain
  - type: feature
    ref: ft-x
    title: {}
    phase: 1
    domains: [{}]
    adrs: [ref:adr-g]
"#,
            adr_title.replace(':', ""),
            d,
            ft_title.replace(':', ""),
            d,
        )
    })
}

// ---------------------------------------------------------------------------
// TC-P012 / TC-548 — failed apply leaves zero files changed
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig {
        cases: std::env::var("PROPTEST_CASES").ok().and_then(|s| s.parse().ok()).unwrap_or(16),
        .. ProptestConfig::default()
    })]

    #[test]
    fn tc_p012_failed_apply_leaves_zero_files_changed((yaml, _kind) in arb_invalid_request()) {
        let bin = find_binary();
        let dir = fresh_repo();

        // Optionally seed some content so the "unchanged" check is meaningful.
        let seed = r#"type: create
schema-version: 1
reason: "seed"
artifacts:
  - type: feature
    title: Seed
    phase: 1
    domains: [api]
"#;
        let (seed_code, _, _) = apply(&bin, dir.path(), seed);
        prop_assert_eq!(seed_code, 0);

        let before = docs_digest(dir.path());

        let (code, _stdout, _stderr) = apply(&bin, dir.path(), &yaml);
        prop_assert_ne!(code, 0, "invalid request should fail to apply");

        let after = docs_digest(dir.path());
        prop_assert_eq!(before, after, "failed apply must leave docs/ unchanged");
    }
}

// ---------------------------------------------------------------------------
// TC-P013 / TC-549 — append is idempotent
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig {
        cases: std::env::var("PROPTEST_CASES").ok().and_then(|s| s.parse().ok()).unwrap_or(16),
        .. ProptestConfig::default()
    })]

    #[test]
    fn tc_p013_append_is_idempotent(extra in prop_oneof![Just("security"), Just("networking"), Just("storage")]) {
        let bin = find_binary();
        let dir = fresh_repo();

        let seed = r#"type: create
schema-version: 1
reason: "seed"
artifacts:
  - type: feature
    title: Target
    phase: 1
    domains: [api]
"#;
        let (seed_code, _, _) = apply(&bin, dir.path(), seed);
        prop_assert_eq!(seed_code, 0);

        let change = format!(
            "type: change\nschema-version: 1\nreason: \"append domain\"\nchanges:\n  - target: FT-001\n    mutations:\n      - op: append\n        field: domains\n        value: {}\n",
            extra,
        );
        let (c1, _, _) = apply(&bin, dir.path(), &change);
        prop_assert_eq!(c1, 0);
        let after_first = docs_digest(dir.path());

        let (c2, _, _) = apply(&bin, dir.path(), &change);
        prop_assert_eq!(c2, 0);
        let after_second = docs_digest(dir.path());

        prop_assert_eq!(after_first, after_second, "append should be idempotent");
    }
}

// ---------------------------------------------------------------------------
// TC-P014 / TC-550 — forward-ref resolution is deterministic
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig {
        cases: std::env::var("PROPTEST_CASES").ok().and_then(|s| s.parse().ok()).unwrap_or(12),
        .. ProptestConfig::default()
    })]

    #[test]
    fn tc_p014_forward_ref_resolution_is_deterministic(yaml in arb_create_request()) {
        let bin = find_binary();

        let dir_a = fresh_repo();
        let dir_b = fresh_repo();

        let (ca, stdout_a, _) = apply(&bin, dir_a.path(), &yaml);
        let (cb, stdout_b, _) = apply(&bin, dir_b.path(), &yaml);

        prop_assert_eq!(ca, 0, "apply A must succeed; stdout={}", stdout_a);
        prop_assert_eq!(cb, 0, "apply B must succeed; stdout={}", stdout_b);

        let map_a = parse_id_map(&stdout_a);
        let map_b = parse_id_map(&stdout_b);
        prop_assert_eq!(&map_a, &map_b, "ref→id mapping must be deterministic across fresh repos");

        // And the assigned IDs are in the proper namespace and topologically valid.
        for (ref_name, id) in map_a.iter() {
            match ref_name.as_str() {
                n if n.starts_with("adr-") => prop_assert!(id.starts_with("ADR-"), "adr ref must map to ADR id"),
                n if n.starts_with("ft-")  => prop_assert!(id.starts_with("FT-"),  "ft ref must map to FT id"),
                n if n.starts_with("tc-")  => prop_assert!(id.starts_with("TC-"),  "tc ref must map to TC id"),
                n if n.starts_with("dep-") => prop_assert!(id.starts_with("DEP-"), "dep ref must map to DEP id"),
                _ => {}
            }
        }
    }
}
