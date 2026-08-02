//! `ddd why` — render the governance chain behind one id.

use std::path::PathBuf;

use product_core::error::{ProductError, Result};

pub fn run(root: Option<PathBuf>, id: &str) -> Result<()> {
    let repo_root = super::resolve_root(root)?;
    let store = ddd_core::store::load(&repo_root.join(ddd_core::store::STORE_DIR));
    match ddd_core::why::render_why(&store, id) {
        Some(text) => {
            print!("{text}");
            Ok(())
        }
        None => Err(ProductError::NotFound(format!(
            "no decision, claim, diagnostic, pattern, or seam with id '{id}' in the .ddd/ graph"
        ))),
    }
}
