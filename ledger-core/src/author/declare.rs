//! `ledger declare` — bringing a decision set into existence.
//!
//! A set fixes the tolerance floor, the ground state, and the owner; it
//! never lists members (§3.5 of the format). Declaring one writes a fresh
//! `sets/<id>.yml` through the same schema type the loader parses, so a
//! declared set cannot disagree with what `verify` will read back.

use crate::set::{DecisionSet, Ground};
use crate::tier::Tier;

use super::{Applied, Author, AuthorError};

/// What a declaration states.
pub struct DeclareArgs {
    pub id: String,
    pub title: String,
    pub tolerance_floor: Tier,
    pub ground: Ground,
    /// Defaults to the author's own identity when absent.
    pub owner: Option<crate::identity::Identity>,
    pub notes: Option<String>,
}

impl Author {
    /// Declare a new set. Refuses an invalid or already-declared id with
    /// the same wording the parse gate uses.
    pub fn declare(&mut self, args: DeclareArgs) -> Result<Applied, AuthorError> {
        DecisionSet::validate_id(&args.id).map_err(AuthorError::Usage)?;
        let store = self.load();
        if store.set(&args.id).is_some() {
            return Err(AuthorError::Usage(format!(
                "set `{}` is already declared under sets/ — a floor change is a set-level governed act, not a redeclaration",
                args.id
            )));
        }
        let set = DecisionSet {
            format: crate::format::CURRENT_FORMAT,
            id: args.id.clone(),
            title: args.title,
            tolerance_floor: args.tolerance_floor,
            ground: args.ground,
            owner: args.owner.unwrap_or_else(|| self.who.clone()),
            created_at: self.today(),
            notes: args.notes,
        };
        let path = self.root.join(crate::STORE_DIR).join("sets").join(format!("{}.yml", set.id));
        if path.exists() {
            return Err(AuthorError::Io(format!("{} already exists", path.display())));
        }
        let text = serde_yaml::to_string(&set)
            .map_err(|e| AuthorError::Io(format!("could not serialise the set: {e}")))?;
        product_core::fileops::write_file_atomic(&path, &text)
            .map_err(|e| AuthorError::Io(e.to_string()))?;
        Ok(Applied {
            path,
            lines: vec![format!(
                "declared set `{}` — floor {}, owner {}",
                set.id, set.tolerance_floor, set.owner
            )],
        })
    }
}
