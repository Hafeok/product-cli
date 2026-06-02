//! Unused unified `Artifact` enum split out to keep `types.rs` under 400 lines.

use crate::types::*;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Artifact {
    Feature(Feature),
    Adr(Adr),
    Test(TestCriterion),
    Dependency(Dependency),
}

#[allow(dead_code)]
impl Artifact {
    pub fn id(&self) -> &str {
        match self {
            Self::Feature(f) => &f.front.id,
            Self::Adr(a) => &a.front.id,
            Self::Test(t) => &t.front.id,
            Self::Dependency(d) => &d.front.id,
        }
    }

    pub fn title(&self) -> &str {
        match self {
            Self::Feature(f) => &f.front.title,
            Self::Adr(a) => &a.front.title,
            Self::Test(t) => &t.front.title,
            Self::Dependency(d) => &d.front.title,
        }
    }
}
