//! Domain (What-capture) session locations, plus its facilitation prompt.
//!
//! The launch path was generalized onto the What-capped workflow session
//! (`product author domain` scaffolds a [`super::workflow`] session capped
//! at What); what remains here is the on-disk session location shared by
//! the domain MCP server (`product author domain <product> --serve`, hosted
//! in the `product-mcp` crate) and the finalize step, and the §2 prompt.

use std::path::{Path, PathBuf};

/// Where the active session for a product is persisted: the product's home,
/// `.product/products/<product>/` (alongside its How/Delivery artifacts). A
/// graph captured under the legacy `.product/author-domain/<product>/` keeps
/// resolving there until the home carries one, so unmigrated repos read and
/// write one consistent location.
pub fn session_dir(root: &Path, product: &str) -> PathBuf {
    let home = crate::pf::paths::product_home(root, product);
    if has_graph(&home, product) {
        return home;
    }
    let legacy = root.join(".product").join("author-domain").join(product);
    if has_graph(&legacy, product) {
        return legacy;
    }
    home
}

/// Whether `dir` holds a captured What graph (the working cache or the spec).
/// Whether a What graph has been captured in `dir` — either the working
/// `session.json` cache or the committed `<product>.ttl`.
pub fn has_graph(dir: &Path, product: &str) -> bool {
    dir.join("session.json").exists() || dir.join(format!("{product}.ttl")).exists()
}

/// The facilitation system prompt — the §2 choreography turned into guidance
/// for the model holding the MCP server as scribe.
pub fn render_prompt(product: &str) -> String {
    include_str!("domain_prompt.md").replace("{{PRODUCT}}", product)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prompt_mentions_product_and_tools() {
        let p = render_prompt("acme");
        assert!(p.contains("acme"));
        assert!(p.contains("session_start"));
        assert!(p.contains("session_finalize"));
        assert!(p.contains("open_questions"));
    }

    #[test]
    fn session_dir_is_the_product_home() {
        let d = session_dir(Path::new("/repo"), "acme");
        assert!(d.ends_with(".product/products/acme"));
    }

    #[test]
    fn session_dir_falls_back_to_a_legacy_author_domain_graph() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let legacy = tmp.path().join(".product/author-domain/acme");
        std::fs::create_dir_all(&legacy).expect("mkdir");
        std::fs::write(legacy.join("acme.ttl"), "# ttl\n").expect("ttl");
        assert!(session_dir(tmp.path(), "acme").ends_with("author-domain/acme"));
        // Once the home carries a graph, it wins.
        let home = tmp.path().join(".product/products/acme");
        std::fs::create_dir_all(&home).expect("mkdir");
        std::fs::write(home.join("acme.ttl"), "# ttl\n").expect("ttl");
        assert!(session_dir(tmp.path(), "acme").ends_with(".product/products/acme"));
    }
}
