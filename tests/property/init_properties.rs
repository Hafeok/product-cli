//! TC-438: Property test for init-generated TOML validity (ADR-033)

use proptest::prelude::*;
use std::path::PathBuf;
use std::process::Command;

/// Find the product binary
fn find_binary() -> PathBuf {
    let mut path = std::env::current_exe().expect("current_exe");
    path.pop(); // remove test binary name
    path.pop(); // remove deps/
    path.push("product");
    if !path.exists() {
        path = PathBuf::from("target/debug/product");
    }
    path
}

/// Generate a safe project name (alphanumeric + hyphens, 1-50 chars)
fn arb_project_name() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9-]{0,49}".prop_map(|s| s.trim_end_matches('-').to_string())
}

/// Generate a safe domain key (alphanumeric + hyphens)
fn arb_domain_key() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9-]{0,19}".prop_map(|s| s.trim_end_matches('-').to_string())
}

/// Generate a domain description (simple ASCII)
fn arb_domain_desc() -> impl Strategy<Value = String> {
    "[A-Za-z ,]{1,40}"
}

/// Generate a domain entry as "key=value"
fn arb_domain_entry() -> impl Strategy<Value = String> {
    (arb_domain_key(), arb_domain_desc()).prop_map(|(k, v)| format!("{}={}", k, v))
}

/// TC-438: init generated toml parses as valid ProductConfig
/// Property: any combination of flags produces a toml parseable by ProductConfig::load()
proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn tc_438_init_generated_toml_parses_as_valid_productconfig(
        name in arb_project_name(),
        domain_entries in proptest::collection::vec(arb_domain_entry(), 0..5),
        port in 1024u16..65535u16,
        write_tools in proptest::bool::ANY,
    ) {
        let dir = tempfile::tempdir().expect("tempdir");
        let bin = find_binary();

        let mut args: Vec<String> = vec![
            "init".to_string(),
            "--yes".to_string(),
            "--name".to_string(),
            name.clone(),
            "--port".to_string(),
            port.to_string(),
        ];

        if write_tools {
            args.push("--write-tools".to_string());
        }

        // Deduplicate by domain key to avoid duplicate TOML keys
        let mut seen_keys = std::collections::HashSet::new();
        for entry in &domain_entries {
            let key = entry.split('=').next().unwrap_or("");
            if seen_keys.insert(key.to_string()) {
                args.push("--domain".to_string());
                args.push(entry.clone());
            }
        }

        let output = Command::new(&bin)
            .args(&args)
            .current_dir(dir.path())
            .output()
            .expect("run binary");

        let exit_code = output.status.code().unwrap_or(-1);
        prop_assert_eq!(
            exit_code, 0,
            "init should succeed. stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        // 1. ProductConfig::load() succeeds
        let toml_path = dir.path().join("product.toml");
        let config = product_lib::config::ProductConfig::load(&toml_path);
        prop_assert!(
            config.is_ok(),
            "ProductConfig::load() should succeed, got: {:?}\ntoml contents:\n{}",
            config.err(),
            std::fs::read_to_string(&toml_path).unwrap_or_default()
        );
        let config = config.expect("just asserted ok");

        // 2. check_schema_version() returns Ok
        let schema_check = config.check_schema_version();
        prop_assert!(
            schema_check.is_ok(),
            "check_schema_version() should succeed, got: {:?}",
            schema_check.err()
        );

        // 3. All paths are valid relative directory strings (no absolute, no ..)
        let paths = [
            &config.paths.features,
            &config.paths.adrs,
            &config.paths.tests,
            &config.paths.graph,
            &config.paths.checklist,
        ];
        for p in &paths {
            prop_assert!(
                !p.starts_with('/'),
                "path should not be absolute: {}",
                p
            );
            prop_assert!(
                !p.contains(".."),
                "path should not contain '..': {}",
                p
            );
        }
    }
}
