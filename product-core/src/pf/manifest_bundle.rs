//! Design-bundle check — the implementation half of the §11.3 manifest.
//!
//! `validate_ds` (manifest.rs) checks the *declaration* half: catalog,
//! coverage, tokens, WCAG guarantees. This module checks the *implementation*
//! half a design bundle adds: every catalog CIO has an implementation for each
//! declared target (and its source exists beside the manifest), every token
//! has a resolved value in every declared theme, and every template composes
//! only catalog components. A declaration-only manifest (no `targets`, no
//! `themes`) passes untouched — the split stays explicit.

use std::path::Path;

use super::manifest::DsManifest;

/// The bundle findings for a manifest rooted at `base` (the manifest's own
/// directory — implementation `source`/`preview` paths resolve against it).
pub fn validate_bundle(m: &DsManifest, base: &Path) -> Vec<String> {
    let ds = &m.design_system;
    let mut findings = Vec::new();
    for target in &ds.targets {
        for c in &ds.components {
            match c.implementation.get(target) {
                None => findings.push(format!(
                    "component '{}' has no implementation for declared target '{target}'",
                    c.id
                )),
                Some(imp) => {
                    if !safe_exists(base, &imp.source) {
                        findings.push(format!(
                            "component '{}' ({target}): source '{}' not found beside the manifest",
                            c.id, imp.source
                        ));
                    }
                }
            }
        }
    }
    for theme in &ds.themes {
        for t in &ds.tokens {
            if !t.values.contains_key(theme) {
                findings.push(format!(
                    "token '{}' has no value for declared theme '{theme}'",
                    t.id
                ));
            }
        }
    }
    for tpl in &ds.templates {
        for cio in &tpl.composes {
            if !ds.components.iter().any(|c| &c.id == cio) {
                findings.push(format!(
                    "template '{}' composes '{cio}', absent from components (closed vocabulary)",
                    tpl.id
                ));
            }
        }
    }
    findings
}

/// True when `rel` is a safe relative path that exists under `base`.
fn safe_exists(base: &Path, rel: &str) -> bool {
    if rel.starts_with('/') || rel.split('/').any(|seg| seg == "..") {
        return false;
    }
    base.join(rel).exists()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pf::manifest::parse_ds;

    fn bundle_yaml(source: &str, theme_values: &str) -> String {
        format!(
            "design_system:\n  id: ds\n  version: \"1.0\"\n  targets: [web]\n  themes: [light]\n\
             \x20 components:\n    - id: primary-button\n      tokens: [color.accent]\n\
             \x20     implementation:\n        web: {{ source: {source} }}\n\
             \x20 tokens:\n    - {{ id: color.accent, type: color{theme_values} }}\n"
        )
    }

    #[test]
    fn whole_bundle_passes() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join("Button.jsx"), "export const Button = 1;").expect("write");
        let m = parse_ds(&bundle_yaml("Button.jsx", ", values: { light: \"#3366ff\" }")).expect("parse");
        assert!(validate_bundle(&m, dir.path()).is_empty());
    }

    #[test]
    fn missing_source_missing_theme_value_and_target_gap_are_findings() {
        let dir = tempfile::tempdir().expect("tempdir");
        let m = parse_ds(&bundle_yaml("Ghost.jsx", "")).expect("parse");
        let f = validate_bundle(&m, dir.path());
        assert!(f.iter().any(|x| x.contains("Ghost.jsx") && x.contains("not found")), "{f:?}");
        assert!(f.iter().any(|x| x.contains("color.accent") && x.contains("light")), "{f:?}");
    }

    #[test]
    fn component_without_target_implementation_is_a_finding() {
        let yaml = "design_system:\n  id: ds\n  targets: [web]\n  components:\n    - id: rail\n";
        let m = parse_ds(yaml).expect("parse");
        let f = validate_bundle(&m, Path::new("/nonexistent"));
        assert!(f.iter().any(|x| x.contains("rail") && x.contains("no implementation")), "{f:?}");
    }

    #[test]
    fn template_composing_off_catalog_component_is_a_finding() {
        let yaml = "design_system:\n  id: ds\n  components:\n    - id: rail\n  templates:\n    - { id: shell, composes: [rail, ghost] }\n";
        let m = parse_ds(yaml).expect("parse");
        let f = validate_bundle(&m, Path::new("/nonexistent"));
        assert_eq!(f.len(), 1);
        assert!(f[0].contains("ghost") && f[0].contains("closed vocabulary"), "{f:?}");
    }

    #[test]
    fn declaration_only_manifest_passes_untouched() {
        let yaml = "design_system:\n  id: ds\n  components:\n    - id: rail\n  tokens:\n    - { id: color.fg, type: color }\n";
        let m = parse_ds(yaml).expect("parse");
        assert!(validate_bundle(&m, Path::new("/nonexistent")).is_empty());
    }

    #[test]
    fn escaping_source_path_is_a_finding() {
        let dir = tempfile::tempdir().expect("tempdir");
        let m = parse_ds(&bundle_yaml("../outside.jsx", ", values: { light: x }")).expect("parse");
        let f = validate_bundle(&m, dir.path());
        assert!(f.iter().any(|x| x.contains("outside.jsx")), "{f:?}");
    }
}
