//! The enumerated, pinned selection a batched acceptance signs.
//!
//! Batching the *act* is an ergonomic affordance; batching the *signature*
//! would be a change to what acceptance means, and this module exists to
//! keep the two apart. A [`Plan`] is a list of `(decision, version hash)`
//! pairs with what the run would do to each — never a predicate carried
//! forward to write time. A selection that re-evaluates between the read
//! and the run is exactly how an unread entry gets signed, so the plan is
//! enumerated once, hashed, and the hash is what a confirming run must
//! present.
//!
//! The members come from [`crate::show`]'s screens, not from a second
//! traversal. That is structural rather than careful: `ledger show --group
//! X` and `ledger accept --group X` cannot resolve to different sets
//! because they are the same resolution, so what a principal read is
//! provably what a principal signs.

use chrono::NaiveDate;
use serde::Serialize;
use serde_json::{Map, Value};

use crate::canon::put;
use crate::hash::domain_hash;
use crate::hold;
use crate::identity::Identity;
use crate::show::{Screen, Selector};

/// Domain-separation prefix for the manifest digest. Its own namespace: a
/// manifest hash and a version hash name different kinds of thing, and one
/// must never be presentable where the other is expected.
pub const MANIFEST_FORM: &str = "ledger.acceptance-manifest.v1";

/// How long a statement head runs before it is elided.
const HEAD: usize = 96;

/// What the run would do to one member. Every member of the selection
/// carries one — there is no unlisted residue, so a reader can see both
/// what is about to be signed and what is not.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Standing {
    /// This member gets its own acceptance record, signing its own hash.
    Signable,
    /// Not for acceptance: the record itself says the ground is unsettled.
    Held,
    /// The actor already carries a live acceptance of this exact hash.
    AlreadySigned,
    /// No signable tip — the chain forked, and arbitration comes first.
    Forked,
}

impl Standing {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Signable => "signable",
            Self::Held => "held",
            Self::AlreadySigned => "already-signed",
            Self::Forked => "forked",
        }
    }

    /// Whether this standing stops the whole run. A held or forked member
    /// is a judgment nobody has made yet; letting the batch proceed around
    /// it would make that judgment silently.
    pub fn blocks(self) -> bool {
        matches!(self, Self::Held | Self::Forked)
    }
}

/// One member of the selection, pinned.
#[derive(Debug, Clone, Serialize)]
pub struct Member {
    pub decision: String,
    /// The exact hash this member's acceptance would sign.
    pub version: String,
    /// The weight class `show` derived, so the summary says what was read.
    pub group: String,
    /// The disposition state at enumeration time.
    pub state: String,
    /// Enough of the statement to recognise the entry on the screen.
    pub statement_head: String,
    pub standing: &'static str,
    /// Why, for every standing but `signable`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub because: Option<String>,
}

/// The whole selection, with the digest that pins it.
#[derive(Debug, Clone, Serialize)]
pub struct Plan {
    /// The selector's canonical spelling, as it enters the manifest.
    pub selector: String,
    /// Whose plan this is — standing is actor-relative, so the manifest is.
    pub actor: String,
    pub members: Vec<Member>,
    pub manifest: String,
}

impl Plan {
    /// The members that would each get an acceptance record.
    pub fn signable(&self) -> impl Iterator<Item = &Member> {
        self.members.iter().filter(|m| m.standing == Standing::Signable.as_str())
    }

    /// The members that stop the run, in selection order.
    pub fn blockers(&self) -> Vec<&Member> {
        self.members
            .iter()
            .filter(|m| {
                m.standing == Standing::Held.as_str() || m.standing == Standing::Forked.as_str()
            })
            .collect()
    }

    /// How many members carry each standing, for the summary line.
    pub fn tally(&self, standing: Standing) -> usize {
        self.members.iter().filter(|m| m.standing == standing.as_str()).count()
    }
}

/// Why a run did not write. Carried on the [`Outcome`] rather than raised
/// as an error so the enumeration is printed either way: a refusal a
/// principal cannot see the selection behind is a refusal they cannot act
/// on.
#[derive(Debug, Clone, Serialize)]
pub struct Refusal {
    /// `held` | `forked` | `drift` | `gate`.
    pub kind: &'static str,
    pub message: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub detail: Vec<String>,
}

/// What one invocation did — including, in the default posture, nothing.
#[derive(Debug, Clone, Serialize)]
pub struct Outcome {
    pub plan: Plan,
    /// True when nothing was written. The default: signing takes an
    /// explicit confirmation carrying the manifest this run enumerated.
    pub dry_run: bool,
    /// The decisions that received an acceptance record — one each, every
    /// one signing its own version hash.
    pub signed: Vec<String>,
    /// The single log file the batched act landed in.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filed: Option<std::path::PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refusal: Option<Refusal>,
}

impl Outcome {
    /// A run the tool said no to — exit 1, in the gate's own discipline.
    pub fn refused(&self) -> bool {
        self.refusal.is_some()
    }
}

/// Enumerate the selection and pin it.
///
/// `today` decides only whether an existing signature has gone stale — the
/// pairs themselves are date-free. A day boundary crossed mid-pass moves a
/// standing, which moves the manifest, which is the drift refusal doing its
/// job rather than a defect.
pub fn plan(
    screens: &[Screen],
    selector: &Selector,
    actor: &Identity,
    today: NaiveDate,
) -> Plan {
    let mut members: Vec<Member> = screens.iter().map(|s| member(s, actor, today)).collect();
    members.sort_by(|a, b| a.decision.cmp(&b.decision));
    let selector = selector.to_string();
    let actor = actor.to_string();
    let manifest = manifest(&selector, &actor, &members);
    Plan { selector, actor, members, manifest }
}

/// The manifest digest over a selection.
///
/// `sha256( MANIFEST_FORM || 0x0A || canonical JSON )` — the same one law
/// [`crate::hash`] states, one level up, over its own explicit field set.
/// Standing is inside the digest, not merely the pairs: an acceptance filed
/// between the read and the run changes what the run would *do* without
/// changing which decisions it covers, and that is a change a signature
/// must not absorb silently.
pub fn manifest(selector: &str, actor: &str, members: &[Member]) -> String {
    let mut m = Map::new();
    put(&mut m, "selector", Some(selector.to_string()));
    put(&mut m, "actor", Some(actor.to_string()));
    let mut rows: Vec<[String; 3]> = members
        .iter()
        .map(|x| [x.decision.clone(), x.version.clone(), x.standing.to_string()])
        .collect();
    rows.sort();
    let array = rows
        .into_iter()
        .map(|r| Value::Array(r.into_iter().map(Value::String).collect()))
        .collect();
    m.insert("members".to_string(), Value::Array(array));
    domain_hash(MANIFEST_FORM, Value::Object(m).to_string().as_bytes())
}

/// Whether a string is shaped like a manifest digest, so a mistyped
/// confirmation is a usage error rather than a drift report.
pub fn is_manifest_shaped(s: &str) -> bool {
    s.strip_prefix("sha256:")
        .is_some_and(|hex| hex.len() == 64 && hex.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()))
}

fn member(s: &Screen, actor: &Identity, today: NaiveDate) -> Member {
    let (standing, because) = standing_of(s, actor, today);
    Member {
        decision: s.decision.clone(),
        version: s.hash.clone(),
        group: s.group.to_string(),
        state: s.state.as_str().to_string(),
        statement_head: head(&s.statement),
        standing: standing.as_str(),
        because,
    }
}

/// The order matters: forked first, because a forked chain has no one hash
/// to reason about at all; then held, which is a statement the record makes
/// about itself; then the no-op.
fn standing_of(s: &Screen, actor: &Identity, today: NaiveDate) -> (Standing, Option<String>) {
    if s.forked {
        return (
            Standing::Forked,
            Some(
                "the version chain is forked — there is no one latest version to sign; `ledger merge --resolve` arbitrates first"
                    .to_string(),
            ),
        );
    }
    if let Some(held) = hold::hold(s.based_on.iter().map(|e| e.pointer.as_str())) {
        return (Standing::Held, Some(held.reason));
    }
    if already_signed(s, actor, today) {
        return (
            Standing::AlreadySigned,
            Some(format!("{actor} already carries a live acceptance of this exact hash")),
        );
    }
    (Standing::Signable, None)
}

/// A live, unexpired, unrevoked signature of the latest hash by this actor.
/// `show` already resolved "live" to mean unrevoked and signing the tip; the
/// only reading left here is expiry, judged exactly as `L003` judges it.
fn already_signed(s: &Screen, actor: &Identity, today: NaiveDate) -> bool {
    let me = actor.to_string();
    s.acceptances.iter().any(|a| {
        a.standing == "live"
            && a.actor == me
            && !a
                .expires_at
                .as_deref()
                .is_some_and(|d| d.parse::<NaiveDate>().is_ok_and(|e| e < today))
    })
}

fn head(statement: &str) -> String {
    let first = statement.lines().next().unwrap_or("").trim();
    match first.char_indices().nth(HEAD) {
        Some((cut, _)) => format!("{}…", &first[..cut]),
        None => first.to_string(),
    }
}

#[path = "batch_render.rs"]
mod render;
pub use render::render;

#[path = "batch_drift.rs"]
mod drift;
pub use drift::{changes, locate, Change};

#[path = "batch_tests.rs"]
#[cfg(test)]
mod tests;
