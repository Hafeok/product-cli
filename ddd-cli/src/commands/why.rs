//! `ddd why` — render the governance chain behind one id.

use std::path::PathBuf;

use ddd_core::why::WhyResolution;
use product_core::error::{ProductError, Result};

pub fn run(root: Option<PathBuf>, id: &str, sarif: Vec<PathBuf>) -> Result<()> {
    let repo_root = super::resolve_root(root)?;
    let store = ddd_core::store::load(&repo_root.join(ddd_core::store::STORE_DIR));
    let detected = ddd_core::detect::detect(&repo_root, store.config.as_ref(), &sarif);
    match ddd_core::why::resolve_why(&store, Some(&detected), id) {
        WhyResolution::Resolved(text) => {
            print!("{text}");
            Ok(())
        }
        // Detected but unfiled is a governance escape, not a lookup miss:
        // the id is real, the mapping is what is absent.
        WhyResolution::DetectedUnfiled { namespace, rule_id, sources } => {
            println!("diagnostic {namespace}/{rule_id} — detected, not in the manifest");
            println!("  {sources}");
            println!("  UNGOVERNED — file a decision plus a manifest entry (see `ddd diff`)");
            Err(ProductError::ConfigError(format!(
                "{namespace}/{rule_id} resolves to no decision"
            )))
        }
        WhyResolution::NotFound => Err(ProductError::NotFound(format!(
            "no decision, claim, diagnostic, pattern, or seam with id '{id}' in the .ddd/ graph"
        ))),
    }
}
