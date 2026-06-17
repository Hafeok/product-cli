//! Command dispatch module — subcommand enums, run(), shared helpers.

mod adr;
mod adr_conflicts;
mod archetype;
mod adr_seal;
mod adr_write;
mod agent_init;
mod author;
mod build;
mod cell;
mod checklist;
mod completions;
mod conformance;
mod context;
mod cycle_times;
mod decider;
mod deliverable;
mod dep;
mod dispatch;
mod domain;
mod domain_fields;
mod drift;
mod drift_diff;
mod feature;
mod feature_fields;
mod feature_write;
mod gap;
mod graph_autolink;
mod graph_cmd;
mod hash;
mod hooks;
mod how;
mod how_fields;
mod implement;
mod init;
mod init_helpers;
mod mcp_cmd;
mod metrics_cmd;
mod migrate;
mod onboard;
mod output;
mod pattern;
mod preflight;
mod prompts_cmd;
mod release;
mod request_builder_add;
mod request_builder_cmd;
mod request_cmd;
mod request_cmd_helpers;
mod request_log_cmd;
mod schema;
mod shared;
mod slice;
mod status;
mod tags;
mod test_cmd;
mod work_unit;
mod worker;

pub(crate) use self::output::{render_result as render, CmdResult, Output};
pub(crate) use self::shared::{acquire_write_lock, acquire_write_lock_typed, load_graph, load_graph_typed};

pub use self::adr::AdrCommands;
pub use self::archetype::ArchetypeCommands;
pub use self::author::AuthorCommands;
pub use self::cell::CellCommands;
pub use self::checklist::ChecklistCommands;
pub use self::conformance::ConformanceCommands;
pub use self::decider::DeciderCommands;
pub use self::deliverable::DeliverableCommands;
pub use self::dep::DepCommands;
pub use self::domain::DomainCommands;
pub use self::drift::DriftCommands;
pub use self::feature::FeatureCommands;
pub use self::gap::GapCommands;
pub use self::graph_cmd::GraphCommands;
pub use self::hash::HashCommands;
pub use self::how::HowCommands;
pub use self::metrics_cmd::MetricsCommands;
pub use self::migrate::MigrateCommands;
pub use self::onboard::OnboardCommands;
pub use self::pattern::PatternCommands;
pub use self::prompts_cmd::PromptsCommands;
pub use self::release::ReleaseCommands;
pub use self::slice::SliceCommands;

mod root_enum;
pub use root_enum::Commands;


pub use self::test_cmd::TestCommands;
pub use self::work_unit::WorkUnitCommands;
pub use self::worker::WorkerCommands;

type BoxResult = Result<(), Box<dyn std::error::Error>>;

pub fn run(command: Commands, format: &str, cli_command: &mut clap::Command) -> BoxResult {
    shared::run_startup_hooks()?;
    dispatch::dispatch(command, format, cli_command)
}
