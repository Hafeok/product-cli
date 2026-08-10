//! Minting ULIDs — the one thing L0 deliberately omitted.
//!
//! L0 validates ids and never creates them; every authoring verb needs to.
//! A ULID is 48 bits of millisecond timestamp plus 80 bits of entropy,
//! rendered as 26 characters of Crockford base32 (`crate::id` owns the
//! validation rules; everything minted here must pass them). Both inputs are
//! injectable so tests are deterministic: the source is a closure yielding
//! `(millis, entropy)`, and [`UlidMint::fixed`] is the reproducible variant.

use std::str::FromStr;

use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};

use crate::id::validate_ulid;

/// The Crockford alphabet, identical to the validator's.
const CROCKFORD: &[u8] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";

/// Render a 48-bit millisecond timestamp plus 80 bits of entropy as the
/// canonical 26-character form. The value is 128 bits under a 130-bit
/// rendering, so the top two bits are always zero and the first character
/// can never overflow the timestamp field.
pub fn encode_ulid(millis: u64, entropy: [u8; 10]) -> String {
    let value = ((u128::from(millis & 0xFFFF_FFFF_FFFF)) << 80)
        | entropy.iter().fold(0u128, |acc, b| (acc << 8) | u128::from(*b));
    (0..26)
        .map(|i| {
            let shift = 5 * (25 - i);
            let index = ((value >> shift) & 0x1F) as usize;
            CROCKFORD[index] as char
        })
        .collect()
}

/// A source of fresh ULIDs. One authoring act may mint several (a change-set
/// id plus decision and acceptance ids), so the source is stateful.
pub struct UlidMint {
    next: Box<dyn FnMut() -> (u64, [u8; 10]) + Send>,
}

impl UlidMint {
    /// A mint drawing on the given source of `(millis, entropy)` pairs.
    pub fn new(next: Box<dyn FnMut() -> (u64, [u8; 10]) + Send>) -> Self {
        Self { next }
    }

    /// The wall-clock mint. Entropy is derived by hashing a process-local
    /// counter with the nanosecond clock and the process id — not key
    /// material, but far past accidental collision.
    pub fn system() -> Self {
        let mut counter: u64 = 0;
        Self::new(Box::new(move || {
            counter += 1;
            let now: DateTime<Utc> = Utc::now();
            let millis = now.timestamp_millis().max(0) as u64;
            let mut hasher = Sha256::new();
            hasher.update(counter.to_be_bytes());
            hasher.update(now.timestamp_nanos_opt().unwrap_or_default().to_be_bytes());
            hasher.update(std::process::id().to_be_bytes());
            let digest = hasher.finalize();
            let mut entropy = [0u8; 10];
            entropy.copy_from_slice(&digest[..10]);
            (millis, entropy)
        }))
    }

    /// A deterministic mint for tests: a fixed timestamp with a counting
    /// entropy field, so successive ids are distinct and reproducible.
    pub fn fixed(millis: u64) -> Self {
        let mut counter: u64 = 0;
        Self::new(Box::new(move || {
            counter += 1;
            let mut entropy = [0u8; 10];
            entropy[2..].copy_from_slice(&counter.to_be_bytes());
            (millis, entropy)
        }))
    }

    /// Mint one ULID, guaranteed to satisfy [`crate::id::validate_ulid`].
    pub fn mint(&mut self) -> String {
        let (millis, entropy) = (self.next)();
        let ulid = encode_ulid(millis, entropy);
        debug_assert!(validate_ulid(&ulid).is_ok(), "minted `{ulid}` fails its own validator");
        ulid
    }

    /// Mint a typed id: the prefix plus a fresh ULID, parsed through the
    /// id type's own validation so a minting bug cannot ship a bad id.
    pub fn mint_id<T: FromStr>(&mut self, prefix: &str) -> Result<T, T::Err> {
        format!("{prefix}:{}", self.mint()).parse()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::id::{AcceptanceId, ChangeSetId};

    #[test]
    fn minted_ulids_validate_and_are_distinct() {
        let mut mint = UlidMint::system();
        let a = mint.mint();
        let b = mint.mint();
        assert!(validate_ulid(&a).is_ok(), "{a}");
        assert!(validate_ulid(&b).is_ok(), "{b}");
        assert_ne!(a, b);
    }

    #[test]
    fn the_fixed_mint_is_deterministic() {
        let mut first = UlidMint::fixed(1_755_000_000_000);
        let mut second = UlidMint::fixed(1_755_000_000_000);
        assert_eq!(first.mint(), second.mint());
        assert_eq!(first.mint(), second.mint());
    }

    #[test]
    fn ids_sort_by_creation_time() {
        let earlier = encode_ulid(1_000, [0xFF; 10]);
        let later = encode_ulid(2_000, [0x00; 10]);
        assert!(earlier < later, "{earlier} should sort before {later}");
    }

    #[test]
    fn the_timestamp_field_cannot_overflow_the_first_character() {
        // The largest representable millisecond value still renders `7`.
        let ulid = encode_ulid(u64::MAX, [0xFF; 10]);
        assert!(validate_ulid(&ulid).is_ok(), "{ulid}");
        assert_eq!(ulid.as_bytes()[0], b'7');
    }

    #[test]
    fn typed_ids_mint_through_their_own_parsers() {
        let mut mint = UlidMint::fixed(1_000);
        let cs: ChangeSetId = mint.mint_id("cs").expect("change-set id");
        let acc: AcceptanceId = mint.mint_id("acc").expect("acceptance id");
        assert!(cs.to_string().starts_with("cs:"));
        assert!(acc.to_string().starts_with("acc:"));
    }
}
