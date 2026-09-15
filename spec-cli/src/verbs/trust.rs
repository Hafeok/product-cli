//! Trusted keys: generating one, and saying which are trusted.
//!
//! Filing a public key is what turns signing on for a repo, and deleting every
//! trusted key is what turns it off. Both are edits to reviewable files rather
//! than flags, which is the only way an accountability guarantee survives the
//! first deadline.

use std::path::{Path, PathBuf};

use chrono::Utc;
use clap::Args;
use product_core::error::{ProductError, Result};
use serde_json::json;
use spec_core::signing::{self, TrustKey, TRUST_KEY_FORM};
use spec_core::store;

use crate::exit;
use crate::render::Report;
use crate::verbs::resolve_identity;

#[derive(Args)]
pub struct TrustGenerateArgs {
    /// A short handle for the key, used as its filename.
    #[arg(long)]
    pub id: String,
    /// Whose key it is. Defaults to the git identity.
    #[arg(long)]
    pub principal: Option<String>,
    /// Where to write the secret key. Must be outside the repo.
    #[arg(long)]
    pub out: PathBuf,
    #[arg(long)]
    pub json: bool,
}

#[derive(Args)]
pub struct TrustListArgs {
    #[arg(long)]
    pub json: bool,
}

/// Generate a keypair, file the public half, write the secret half outside the
/// repo.
pub fn generate(root: &Path, args: &TrustGenerateArgs) -> Result<Report> {
    refuse_inside_repo(root, &args.out)?;
    let principal = resolve_identity(root, args.principal.as_deref())?;
    let (secret, public) = signing::generate();

    write_secret(&args.out, &secret)?;
    let filed = signing::file_trust(
        root,
        TrustKey {
            form: TRUST_KEY_FORM.to_string(),
            id: args.id.clone(),
            principal,
            algorithm: "ed25519".to_string(),
            public_key: public,
            added_at: Utc::now(),
        },
    )?;

    let text = format!(
        "trusted {} for {}\n  secret key → {}\n\n\
         Keep the secret key out of the repo and off shared machines.\n\
         Sign with `--key-file {}` or SPEC_SIGNING_KEY.\n\
         Filing this key turns signing on for this repo: every closure, \
         ratification and refusal now needs one.",
        filed.id,
        filed.principal.as_str(),
        args.out.display(),
        args.out.display()
    );
    let body = args.json.then(|| json!({
        "key": filed.id,
        "principal": filed.principal.as_str(),
        "public_key": filed.public_key,
        "secret_key_path": args.out.display().to_string(),
    }));
    Ok(Report::text(exit::CONFORMANT, text).with_json(body))
}

/// List the trusted keys, and say what their presence means.
pub fn list(root: &Path, args: &TrustListArgs) -> Result<Report> {
    let spec = store::load_store(root)?;
    let text = if spec.trust.is_empty() {
        "no trusted keys — signing is off.\n  \
         `S002` still refuses a machine-looking principal, but nothing \
         establishes that the named human acted.\n  \
         `spec trust generate` turns signing on."
            .to_string()
    } else {
        let listed = spec
            .trust
            .iter()
            .map(|k| format!("  {:<16} {}  {}", k.id, k.principal.as_str(), k.public_key))
            .collect::<Vec<_>>()
            .join("\n");
        format!("signing is on; {} key(s) trusted:\n{listed}", spec.trust.len())
    };
    let body = args.json.then(|| json!(spec.trust.iter().map(|k| json!({
        "key": k.id,
        "principal": k.principal.as_str(),
        "public_key": k.public_key,
    })).collect::<Vec<_>>()));
    Ok(Report::text(exit::CONFORMANT, text).with_json(body))
}

/// The signing key a write should use, if any.
///
/// `--key-file` wins over the environment. Absence is not an error here — the
/// gate decides whether an unsigned write is acceptable, and deciding it twice
/// is how the two answers drift apart.
pub fn signing_key(key_file: Option<&Path>) -> Result<Option<ed25519_dalek::SigningKey>> {
    let hex = match key_file {
        Some(path) => Some(
            std::fs::read_to_string(path)
                .map_err(|e| ProductError::IoError(format!("{}: {e}", path.display())))?,
        ),
        None => std::env::var("SPEC_SIGNING_KEY").ok(),
    };
    let Some(hex) = hex else {
        return Ok(None);
    };
    signing::signing_key_from_hex(&hex)
        .map(Some)
        .ok_or_else(|| ProductError::ConfigError("the signing key is not 32 bytes of hex".into()))
}

/// A secret key inside the repo is one commit away from being public.
fn refuse_inside_repo(root: &Path, out: &Path) -> Result<()> {
    let root = std::fs::canonicalize(root).unwrap_or_else(|_| root.to_path_buf());
    let target = out.parent().map_or_else(
        || out.to_path_buf(),
        |p| std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf()),
    );
    if target.starts_with(&root) {
        return Err(ProductError::ConfigError(format!(
            "{} is inside the repo — a secret key there is one commit from being public",
            out.display()
        )));
    }
    Ok(())
}

/// Write the secret, owner-readable only where the platform allows it.
fn write_secret(path: &Path, secret: &str) -> Result<()> {
    if path.exists() {
        return Err(ProductError::ConfigError(format!(
            "{} already exists — refusing to overwrite a key",
            path.display()
        )));
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| ProductError::WriteError {
            path: parent.to_path_buf(),
            message: e.to_string(),
        })?;
    }
    std::fs::write(path, format!("{secret}\n")).map_err(|e| ProductError::WriteError {
        path: path.to_path_buf(),
        message: e.to_string(),
    })?;
    restrict(path);
    Ok(())
}

#[cfg(unix)]
fn restrict(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600));
}

#[cfg(not(unix))]
fn restrict(_path: &Path) {
    // Windows ACLs are not a chmod; the caller is told to keep the key safe.
}
