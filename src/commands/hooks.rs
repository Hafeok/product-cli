//! Git hook installation, MCP scaffolding.

use product_lib::{config::ProductConfig, fileops, mcp};

use super::{acquire_write_lock, BoxResult};

pub(crate) fn handle_install_hooks() -> BoxResult {
    let _lock = acquire_write_lock()?;
    let (_config, root) = ProductConfig::discover()?;

    // Write pre-commit hook
    let hooks_dir = root.join(".git").join("hooks");
    std::fs::create_dir_all(&hooks_dir)?;

    let hook_path = hooks_dir.join("pre-commit");
    let hook_content = "#!/bin/sh\n\
        # Installed by `product install-hooks`\n\
        exec product adr review --staged\n";
    fileops::write_file_atomic(&hook_path, hook_content)?;

    // Make executable (Unix)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&hook_path, std::fs::Permissions::from_mode(0o755))?;
    }

    println!("Installed pre-commit hook: {}", hook_path.display());

    // Write .mcp.json
    mcp::scaffold_mcp_json(&root)?;
    println!("Wrote .mcp.json: {}", root.join(".mcp.json").display());

    Ok(())
}
