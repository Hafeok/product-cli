//! Entry format versions this tool validates against.
//!
//! Every `.decisions/` file declares `format: N`; validation is always
//! against the declared version, never against the newest one. A schema
//! change is a version bump with a migration note in
//! `docs/ledger-format-migrations.md`, so existing entries never break
//! silently. Inherited from the `ddd` store's discipline (`ddd` PRD §6).
//!
//! Note the second, independent version: [`crate::canon::CANONICAL_FORM`]
//! versions the *hash* form. A `format` bump that changes what a hashed
//! field means must bump that one too, because acceptances signed under the
//! old reading would otherwise silently re-point.

/// The format version written by this tool.
pub const CURRENT_FORMAT: u32 = 1;

/// Every format version this tool can validate an entry against.
pub const SUPPORTED_FORMATS: &[u32] = &[1];

/// Whether an entry declaring `format: n` can be validated here.
pub fn is_supported(n: u32) -> bool {
    SUPPORTED_FORMATS.contains(&n)
}

/// The message a file declaring an unknown format fails with.
pub fn unsupported_message(n: u32) -> String {
    let known: Vec<String> = SUPPORTED_FORMATS.iter().map(u32::to_string).collect();
    format!("declares format {n}; this tool validates format(s) {}", known.join(", "))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_current_format_is_supported() {
        assert!(is_supported(CURRENT_FORMAT));
    }

    #[test]
    fn an_unknown_format_names_what_is_known() {
        assert!(!is_supported(9));
        assert_eq!(unsupported_message(9), "declares format 9; this tool validates format(s) 1");
    }
}
