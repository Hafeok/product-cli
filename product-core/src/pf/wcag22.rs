//! The WCAG 2.2 success-criterion identifiers — reference data for validating a
//! design-system manifest's `satisfies` claims (§11.3, FT-141).
//!
//! The full set of 86 success criteria across levels A/AA/AAA (4.1.1 Parsing is
//! obsolete in 2.2 and excluded). A `satisfies` entry citing an identifier
//! outside this set is not a real WCAG 2.2 entity.

/// Every WCAG 2.2 success-criterion number.
pub const WCAG_22_CRITERIA: &[&str] = &[
    "1.1.1",
    "1.2.1", "1.2.2", "1.2.3", "1.2.4", "1.2.5", "1.2.6", "1.2.7", "1.2.8", "1.2.9",
    "1.3.1", "1.3.2", "1.3.3", "1.3.4", "1.3.5", "1.3.6",
    "1.4.1", "1.4.2", "1.4.3", "1.4.4", "1.4.5", "1.4.6", "1.4.7", "1.4.8", "1.4.9",
    "1.4.10", "1.4.11", "1.4.12", "1.4.13",
    "2.1.1", "2.1.2", "2.1.3", "2.1.4",
    "2.2.1", "2.2.2", "2.2.3", "2.2.4", "2.2.5", "2.2.6",
    "2.3.1", "2.3.2", "2.3.3",
    "2.4.1", "2.4.2", "2.4.3", "2.4.4", "2.4.5", "2.4.6", "2.4.7", "2.4.8", "2.4.9",
    "2.4.10", "2.4.11", "2.4.12", "2.4.13",
    "2.5.1", "2.5.2", "2.5.3", "2.5.4", "2.5.5", "2.5.6", "2.5.7", "2.5.8",
    "3.1.1", "3.1.2", "3.1.3", "3.1.4", "3.1.5", "3.1.6",
    "3.2.1", "3.2.2", "3.2.3", "3.2.4", "3.2.5", "3.2.6",
    "3.3.1", "3.3.2", "3.3.3", "3.3.4", "3.3.5", "3.3.6", "3.3.7", "3.3.8", "3.3.9",
    "4.1.2", "4.1.3",
];

/// Whether `id` is a real WCAG 2.2 success-criterion number.
pub fn is_wcag_22(id: &str) -> bool {
    WCAG_22_CRITERIA.contains(&id)
}
