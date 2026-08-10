//! Tolerance tiers with the floor rule that makes tier-shopping unexpressible.
//!
//! §4.2.1 (OD-4, resolved): a set's tolerance is a **floor**, not a default.
//! A version's effective tier is the floor it was created under unless it
//! carries an up-only `tolerance_override`. A below-floor override is a
//! schema violation rejected at write — there is no representable state in
//! which a decision sits below its set's regime. Silent tier-shopping is not
//! caught; it is unconstructible, which is why [`Tolerance`]'s fields are
//! private and [`Tolerance::new`] is the only way in.
//!
//! The floor is **pinned on the version**, not read live from the set. Without
//! the pin the effective tier would not be recomputable after a floor raise,
//! so the version hash could not be stable and acceptance-binds-the-tier
//! would be unimplementable. Same discipline as the `ddd` store's format-2
//! `based_on` pins, one level up.

use std::fmt;

use serde::{Deserialize, Serialize};

/// The declared assurance level, ordered by consequence
/// (`way-of-working-decision-allocated-delivery.md` §2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Tier {
    /// Consequence trivial, lifetime short.
    T0,
    /// Normal product surface.
    T1,
    /// Money, identity, data loss, regulatory.
    T2,
}

impl fmt::Display for Tier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::T0 => "T0",
            Self::T1 => "T1",
            Self::T2 => "T2",
        };
        f.write_str(s)
    }
}

/// Why a tolerance could not be constructed. Carried through to verify class
/// `L004` when the state is met in a hand-edited file rather than at write.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BelowFloor {
    pub over: Tier,
    pub floor: Tier,
}

impl fmt::Display for BelowFloor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let relation = if self.over == self.floor { "equals" } else { "is below" };
        write!(
            f,
            "tolerance_override {} {relation} the set floor {} — an override is valid only above the floor (§4.2.1)",
            self.over, self.floor
        )
    }
}

/// A version's tolerance: the pinned floor plus an optional up-only override.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tolerance {
    floor_at_creation: Tier,
    over: Option<Tier>,
}

impl Tolerance {
    /// The only constructor. An override at or below the pinned floor is
    /// rejected here, so a below-floor `Tolerance` never exists in memory.
    ///
    /// An override *equal* to the floor is rejected too: it is not "above",
    /// and it is a no-op that would only add noise to the version hash.
    pub fn new(floor_at_creation: Tier, over: Option<Tier>) -> Result<Self, BelowFloor> {
        if let Some(o) = over {
            if o <= floor_at_creation {
                return Err(BelowFloor { over: o, floor: floor_at_creation });
            }
        }
        Ok(Self { floor_at_creation, over })
    }

    /// The set floor as it stood when this version was created.
    pub fn floor_at_creation(&self) -> Tier {
        self.floor_at_creation
    }

    /// The up-only override, when the version carries one.
    pub fn override_tier(&self) -> Option<Tier> {
        self.over
    }

    /// The tier this version is actually governed at.
    pub fn effective(&self) -> Tier {
        self.over.unwrap_or(self.floor_at_creation)
    }

    /// Whether a set floor standing at `current` has stranded this version
    /// below its regime (verify class `L005`). Re-acceptance at the new floor
    /// means a new version pinning it; entries are never grandfathered.
    pub fn is_stranded_below(&self, current: Tier) -> bool {
        self.effective() < current
    }
}

#[path = "tier_tests.rs"]
#[cfg(test)]
mod tests;
