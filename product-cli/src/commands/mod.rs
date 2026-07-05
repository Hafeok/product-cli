//! Command dispatch module — subcommand enums, run(), shared helpers.

mod blueprint;
mod author;
mod build;
mod build_guard;
mod build_seam_emit;
mod build_lsp;
mod build_session;
mod build_verify;
mod cell;
mod completions;
mod decider;
mod primitive;
mod projector;
mod deliverable;
mod deployable_unit;
mod dispatch;
mod domain;
mod domain_data;
mod domain_fields;
mod domain_fields_v16;
mod domain_rows;
mod guide;
mod hooks;
mod how;
mod how_fields;
mod init;
mod init_helpers;
mod lsp;
mod mcp_cmd;
mod output;
mod preview;
mod release;
mod seam;
mod session;
mod shared;
mod skills;
mod feature;
mod reify;
mod reify_how;
mod target;
mod verdict;
mod work_unit;
mod worker;

pub(crate) use self::output::render_result as render;
pub(crate) use self::shared::acquire_write_lock;

pub use self::blueprint::BlueprintCommands;
pub use self::author::AuthorCommands;
pub use self::cell::CellCommands;
pub use self::decider::DeciderCommands;
pub use self::primitive::PrimitiveCommands;
pub use self::projector::ProjectorCommands;
pub use self::deliverable::DeliverableCommands;
pub use self::deployable_unit::DeployableUnitCommands;
pub use self::domain::DomainCommands;
pub use self::how::HowCommands;
pub use self::lsp::LspCommands;
pub use self::preview::PreviewCommands;
pub use self::release::ReleaseCommands;
pub use self::session::SessionCommands;
pub use self::skills::SkillsCommands;
pub use self::feature::FeatureCommands;
pub use self::reify::ReifyCommands;
pub use self::target::TargetCommands;

mod root_enum;
pub use root_enum::Commands;


pub use self::work_unit::WorkUnitCommands;
pub use self::worker::WorkerCommands;

type BoxResult = Result<(), Box<dyn std::error::Error>>;

pub fn run(command: Commands, format: &str, cli_command: &mut clap::Command) -> BoxResult {
    shared::run_startup_hooks()?;
    dispatch::dispatch(command, format, cli_command)
}
