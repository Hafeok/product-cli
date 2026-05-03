//! Compile-time-validated convention check identifier.
//!
//! `CtxId` is the canonical example of a Tier 1 rule (see CTX003): the type
//! system makes the wrong code unrepresentable. There is no public string
//! constructor. The only way to obtain a `CtxId` is `CtxId::new("CTX###")`,
//! which validates the format inside a `const` context — invalid IDs fail to
//! compile, so no diagnostic is needed at runtime.

use std::fmt;

/// A workspace convention check identifier (`CTX` followed by digits).
///
/// # Examples
///
/// ```
/// use xtask::CtxId;
///
/// const ID: CtxId = CtxId::new("CTX001");
/// assert_eq!(ID.as_str(), "CTX001");
/// ```
///
/// Lowercase, missing prefix, or non-digit suffix all fail to compile:
///
/// ```compile_fail
/// use xtask::CtxId;
/// const BAD: CtxId = CtxId::new("ctx001");
/// ```
///
/// ```compile_fail
/// use xtask::CtxId;
/// const BAD: CtxId = CtxId::new("RULE001");
/// ```
///
/// ```compile_fail
/// use xtask::CtxId;
/// const BAD: CtxId = CtxId::new("CTX");
/// ```
///
/// ```compile_fail
/// use xtask::CtxId;
/// const BAD: CtxId = CtxId::new("CTX-DEPS");
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct CtxId(&'static str);

impl CtxId {
    /// Construct a `CtxId` at compile time. The argument must match the
    /// regex `^CTX[0-9]+$`. Misuse panics in `const` evaluation, which the
    /// compiler surfaces as an error at the call site rather than at
    /// runtime.
    pub const fn new(s: &'static str) -> Self {
        assert!(is_valid(s), "CtxId must match `CTX` followed by one or more digits, e.g. CTX001");
        Self(s)
    }

    /// Borrow the inner string.
    pub const fn as_str(&self) -> &'static str {
        self.0
    }
}

impl fmt::Display for CtxId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}

impl From<CtxId> for String {
    fn from(id: CtxId) -> Self {
        id.0.to_owned()
    }
}

const fn is_valid(s: &str) -> bool {
    let bytes = s.as_bytes();
    if bytes.len() < 4 {
        return false;
    }
    if bytes[0] != b'C' || bytes[1] != b'T' || bytes[2] != b'X' {
        return false;
    }
    let mut i = 3;
    while i < bytes.len() {
        if !bytes[i].is_ascii_digit() {
            return false;
        }
        i += 1;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_canonical_ids() {
        assert_eq!(CtxId::new("CTX001").as_str(), "CTX001");
        assert_eq!(CtxId::new("CTX042").as_str(), "CTX042");
        assert_eq!(CtxId::new("CTX9999").as_str(), "CTX9999");
    }

    #[test]
    fn display_renders_inner() {
        assert_eq!(format!("{}", CtxId::new("CTX001")), "CTX001");
    }
}
