//! Signatures over the acts a principal owns.
//!
//! `S002` establishes only that a named principal does not *look* like a
//! machine. A signature establishes that the holder of a key bound to that
//! principal actually acted — which is the difference between an accountability
//! claim and an accountability fact, and the difference the PRD's §14.2
//! asserted before anything implemented it.
//!
//! Opt-in, and opt-in at the repo rather than per record: signing turns on when
//! a trust root exists, and turning it off means deleting trusted keys, which
//! is a visible, reviewable act rather than a flag on one write.

use chrono::{DateTime, Utc};
use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use ledger_core::identity::Identity;
use serde::{Deserialize, Serialize};

/// Domain-separation prefix for what a signature covers.
pub const SIGNATURE_FORM: &str = "spec.signature.v1";

/// One public key a principal is trusted to act under.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TrustKey {
    pub form: String,
    /// A short handle for the key, used as its filename.
    pub id: String,
    /// Whose key it is. A signature verifies only against keys bound to the
    /// principal the record names.
    pub principal: Identity,
    /// Always `ed25519` in v1. Present so a second algorithm is a format
    /// change rather than a silent reinterpretation of the bytes.
    pub algorithm: String,
    /// Hex, 32 bytes.
    pub public_key: String,
    pub added_at: DateTime<Utc>,
}

/// Domain-separation prefix for a trust key file.
pub const TRUST_KEY_FORM: &str = "spec.trust-key.v1";

impl TrustKey {
    /// The parsed verifying key, or `None` when the stored bytes are not one.
    pub fn verifying_key(&self) -> Option<VerifyingKey> {
        if self.algorithm != "ed25519" {
            return None;
        }
        let bytes: [u8; 32] = decode_hex(&self.public_key)?.try_into().ok()?;
        VerifyingKey::from_bytes(&bytes).ok()
    }
}

/// What a signature is computed over.
///
/// The subject is the record's own digest, which already covers the principal
/// it names — so a signature cannot be lifted onto another principal's record
/// without breaking, and there is no separate replay guard to get wrong.
pub fn subject(digest: &str) -> Vec<u8> {
    let mut bytes = SIGNATURE_FORM.as_bytes().to_vec();
    bytes.push(b'\n');
    bytes.extend_from_slice(digest.as_bytes());
    bytes
}

/// Sign a digest, returning hex.
pub fn sign(digest: &str, key: &SigningKey) -> String {
    encode_hex(&key.sign(&subject(digest)).to_bytes())
}

/// Whether a signature over a digest verifies under any trusted key bound to
/// the given principal.
pub fn verifies(digest: &str, signature: &str, principal: &Identity, trust: &[TrustKey]) -> bool {
    let Some(bytes) = decode_hex(signature) else {
        return false;
    };
    let Ok(sized): std::result::Result<[u8; 64], _> = bytes.try_into() else {
        return false;
    };
    let parsed = Signature::from_bytes(&sized);
    let message = subject(digest);
    trust
        .iter()
        .filter(|k| &k.principal == principal)
        .filter_map(TrustKey::verifying_key)
        .any(|k| k.verify_strict(&message, &parsed).is_ok())
}

/// Generate a fresh keypair, returning `(secret hex, public hex)`.
pub fn generate() -> (String, String) {
    let key = SigningKey::generate(&mut rand_core::OsRng);
    (encode_hex(&key.to_bytes()), encode_hex(key.verifying_key().as_bytes()))
}

/// Read a signing key from hex.
pub fn signing_key_from_hex(hex: &str) -> Option<SigningKey> {
    let bytes: [u8; 32] = decode_hex(hex.trim())?.try_into().ok()?;
    Some(SigningKey::from_bytes(&bytes))
}

/// Lowercase hex, matching the `sha256:` digests this store already carries.
pub fn encode_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Parse lowercase or uppercase hex; `None` on any non-hex byte or odd length.
pub fn decode_hex(text: &str) -> Option<Vec<u8>> {
    let trimmed = text.trim();
    if !trimmed.len().is_multiple_of(2) || trimmed.is_empty() {
        return None;
    }
    (0..trimmed.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(trimmed.get(i..i + 2)?, 16).ok())
        .collect()
}

/// Where trusted public keys live under a repo root.
pub fn trust_dir(repo_root: &std::path::Path) -> std::path::PathBuf {
    repo_root.join(".spec").join("trust")
}

/// Every trusted key, ordered by file name.
///
/// An empty trust root means signing is off. That is a repo-level state, not a
/// per-record flag: turning signing off means deleting trusted keys, which is
/// a reviewable act rather than an omission on one write.
pub fn load_trust(repo_root: &std::path::Path) -> product_core::error::Result<Vec<TrustKey>> {
    crate::ratify::load_dir(&trust_dir(repo_root))
}

/// File a trusted key.
pub fn file_trust(
    repo_root: &std::path::Path,
    key: TrustKey,
) -> product_core::error::Result<TrustKey> {
    if key.verifying_key().is_none() {
        return Err(product_core::error::ProductError::ConfigError(format!(
            "`{}` is not a usable {} public key",
            key.public_key, key.algorithm
        )));
    }
    let path = trust_dir(repo_root).join(format!("{}.yml", crate::ratify::slug(&key.id)));
    if path.exists() {
        return Err(product_core::error::ProductError::ConfigError(format!(
            "a key with id `{}` is already trusted — rotate under a new id",
            key.id
        )));
    }
    crate::ratify::write_yaml(&path, &key)?;
    Ok(key)
}

/// Sign a digest if a key was supplied, leaving it unsigned otherwise.
///
/// Unsigned is not silently fine: the gate decides whether it is, and refuses
/// the write when a trust root exists. Deciding here would put the rule in two
/// places.
pub fn sign_if_keyed(digest: &str, signer: Option<&SigningKey>) -> Option<String> {
    signer.map(|key| sign(digest, key))
}

#[path = "signing_tests.rs"]
#[cfg(test)]
pub(crate) mod tests;
