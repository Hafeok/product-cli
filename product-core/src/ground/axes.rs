//! The two independent grades a proposed assertion carries (§5.2, second axis).
//!
//! G0 measured that the table's single `assurance` column was two properties
//! wearing one name, independent rather than correlated: 31 assertions at the
//! top grade, every mechanical check passing, the instrument having read the
//! code perfectly, every one of them wrong to ratify. They were private
//! serialisation state on infrastructure types — high confidence, no domain
//! relevance.
//!
//! So the mark splits. [`DerivationConfidence`] says how reliably the
//! instrument read; [`DomainRelevance`] says whether the result is the kind of
//! thing the registry is for. The two are not the same *kind* of scale, and
//! nothing here pretends they are: confidence is ordinal, relevance is a
//! verdict that may be undetermined, and [`RelevanceBasis`] records which.

use serde::Serialize;

/// How reliably the instrument read the code — §5.2's first axis.
///
/// The value set is unchanged from the column this replaces, deliberately:
/// that column was already measuring derivation and nothing else, so the split
/// re-names it rather than re-marking it, and every assertion ratified under
/// the earlier generation keeps the mark it carries.
///
/// Ordinal, declaration order best-first, so sorting ascending reads
/// confidence descending.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DerivationConfidence {
    /// Declared. A probed operation returned the thing itself; the language's
    /// own semantics say what it is.
    High,
    /// Structural, or inferred from usage. The server answered, and a mapping
    /// rule took a step the language does not itself take.
    Mid,
    /// Heuristic. A lexical rule over identifier text, with no server
    /// semantics behind it.
    Low,
    /// No instrument on this surface, at any grade.
    NotDerivable,
}

impl DerivationConfidence {
    pub fn as_str(&self) -> &'static str {
        match self {
            DerivationConfidence::High => "high",
            DerivationConfidence::Mid => "mid",
            DerivationConfidence::Low => "low",
            DerivationConfidence::NotDerivable => "not-derivable",
        }
    }
}

/// Whether the result is the kind of thing the registry is for — §5.2's
/// second axis.
///
/// Three values, not four. No middle mark is defined because no rule produces
/// one, and the weakest thing that satisfies a stated output is usually a sign
/// the output was mis-specified.
///
/// `Unknown` is **not** between `High` and `Low`. It is the absence of a
/// verdict, which is why this enum derives no ordering.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DomainRelevance {
    /// The assertion states something about the domain.
    High,
    /// Read correctly; describes a mechanism of the software rather than of
    /// the domain.
    Low,
    /// Not computed. The extractor has no rule that decides it from what it
    /// read, and says so rather than defaulting to a plausible mark.
    #[default]
    Unknown,
}

impl DomainRelevance {
    pub fn as_str(&self) -> &'static str {
        match self {
            DomainRelevance::High => "high",
            DomainRelevance::Low => "low",
            DomainRelevance::Unknown => "unknown",
        }
    }

    /// Sort key, so batches order deterministically over a set that carries no
    /// natural ordering.
    pub(crate) fn rank(&self) -> u8 {
        match self {
            DomainRelevance::High => 0,
            DomainRelevance::Low => 1,
            DomainRelevance::Unknown => 2,
        }
    }
}

/// Where a [`DomainRelevance`] mark came from.
///
/// A mark without its basis is unreadable: `unknown` because nothing was
/// computed and `unknown` because the row proposes nothing are different
/// findings, and collapsing them is the failure this whole shape avoids.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RelevanceBasis {
    /// A named rule decided it from facts the extractor read.
    Computed,
    /// The row's own construction fixes it, independent of any codebase.
    Defaulted,
    /// The extractor has no rule for it on this surface.
    #[default]
    NotComputed,
    /// The row proposes nothing, so no assertion carries a mark.
    NotApplicable,
}

impl RelevanceBasis {
    pub fn as_str(&self) -> &'static str {
        match self {
            RelevanceBasis::Computed => "computed",
            RelevanceBasis::Defaulted => "defaulted",
            RelevanceBasis::NotComputed => "not-computed",
            RelevanceBasis::NotApplicable => "not-applicable",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The whole point of the split: the pair is free to disagree. A schema
    /// migration read perfectly is top confidence with no relevance.
    #[test]
    fn the_two_axes_vary_independently() {
        let read_perfectly_but_irrelevant =
            (DerivationConfidence::High, DomainRelevance::Low, RelevanceBasis::Computed);
        let guessed_but_on_topic =
            (DerivationConfidence::Low, DomainRelevance::High, RelevanceBasis::Computed);
        assert_ne!(read_perfectly_but_irrelevant.0, guessed_but_on_topic.0);
        assert_ne!(read_perfectly_but_irrelevant.1, guessed_but_on_topic.1);
    }

    /// Confidence is ordinal and sorts best-first; relevance is not ordinal
    /// and therefore carries an explicit rank instead of an `Ord`.
    #[test]
    fn confidence_is_ordinal_where_relevance_is_not() {
        let mut marks = [
            DerivationConfidence::Low,
            DerivationConfidence::NotDerivable,
            DerivationConfidence::High,
            DerivationConfidence::Mid,
        ];
        marks.sort();
        assert_eq!(marks[0], DerivationConfidence::High);
        assert_eq!(marks[3], DerivationConfidence::NotDerivable);
        // Unknown is the absence of a verdict, not a middle one.
        assert!(DomainRelevance::Unknown.rank() > DomainRelevance::Low.rank());
    }

    /// Every mark renders distinctly, so a report can never say two things
    /// with one word.
    #[test]
    fn every_mark_reads_differently_from_every_other() {
        let mut all: Vec<&str> = vec![
            DerivationConfidence::High.as_str(),
            DerivationConfidence::Mid.as_str(),
            DerivationConfidence::Low.as_str(),
            DerivationConfidence::NotDerivable.as_str(),
            RelevanceBasis::Computed.as_str(),
            RelevanceBasis::Defaulted.as_str(),
            RelevanceBasis::NotComputed.as_str(),
            RelevanceBasis::NotApplicable.as_str(),
        ];
        let before = all.len();
        all.sort_unstable();
        all.dedup();
        assert_eq!(all.len(), before);
    }

    /// An unset relevance is `unknown`, never a plausible default — the
    /// presumed-discharge shape, refused at the type level.
    #[test]
    fn the_default_relevance_is_unknown_not_high() {
        assert_eq!(DomainRelevance::default(), DomainRelevance::Unknown);
        assert_eq!(RelevanceBasis::default(), RelevanceBasis::NotComputed);
    }
}
