//! Tag lifecycle browsing — `product tags` command group (ADR-036)

use clap::Subcommand;
use product_lib::tags;

use super::{load_graph, BoxResult};

#[derive(Subcommand)]
pub enum TagsCommands {
    /// List all product/* tags
    List {
        /// Filter to a specific feature lifecycle (e.g., --feature FT-001)
        #[arg(long)]
        feature: Option<String>,
        /// Filter by event type (e.g., --type complete)
        #[arg(long, rename_all = "kebab-case", value_name = "TYPE")]
        r#type: Option<String>,
    },
    /// Show detailed info for a feature's tags
    Show {
        /// Artifact ID (e.g., FT-001)
        artifact_id: String,
    },
}

pub(crate) fn handle_tags(cmd: TagsCommands, fmt: &str) -> BoxResult {
    let (_config, root, _graph) = load_graph()?;

    if !tags::is_git_repo(&root) {
        eprintln!("warning[W018]: not a git repository \u{2014} no tags available");
        return Ok(());
    }

    match cmd {
        TagsCommands::List { feature, r#type } => {
            tags_list(&root, feature, r#type, fmt)
        }
        TagsCommands::Show { artifact_id } => {
            tags_show(&root, &artifact_id, fmt)
        }
    }
}

fn tags_list(
    root: &std::path::Path,
    feature: Option<String>,
    event_type: Option<String>,
    fmt: &str,
) -> BoxResult {
    let filter = tags::TagFilter {
        feature,
        event_type,
    };
    let tag_list = tags::list_tags(root, &filter);

    if fmt == "json" {
        println!("{}", serde_json::to_string_pretty(&tag_list).unwrap_or_default());
    } else if tag_list.is_empty() {
        println!("No product tags found.");
    } else {
        println!(
            "{:<40} {:<12} {:<12} DATE",
            "TAG", "ARTIFACT", "EVENT"
        );
        for tag in &tag_list {
            println!(
                "{:<40} {:<12} {:<12} {}",
                tag.name, tag.artifact_id, tag.event, tag.timestamp
            );
        }
    }
    Ok(())
}

fn tags_show(
    root: &std::path::Path,
    artifact_id: &str,
    fmt: &str,
) -> BoxResult {
    let details = tags::show_artifact_tags(root, artifact_id);

    if details.is_empty() {
        // Try showing a single tag by exact name
        let full_name = if artifact_id.starts_with("product/") {
            artifact_id.to_string()
        } else {
            // Try as artifact ID
            format!("product/{}", artifact_id)
        };
        // Check if it exists as exact tag name (unlikely for show)
        if let Some(detail) = tags::show_tag(root, &full_name) {
            if fmt == "json" {
                println!("{}", serde_json::to_string_pretty(&detail).unwrap_or_default());
            } else {
                print_tag_detail(&detail);
            }
            return Ok(());
        }
        eprintln!("No tags found for {}", artifact_id);
        std::process::exit(1);
    }

    if fmt == "json" {
        println!("{}", serde_json::to_string_pretty(&details).unwrap_or_default());
    } else {
        for detail in &details {
            print_tag_detail(detail);
            println!();
        }
    }
    Ok(())
}

fn print_tag_detail(detail: &tags::TagDetail) {
    println!("Tag:       {}", detail.name);
    println!("Artifact:  {}", detail.artifact_id);
    println!("Event:     {}", detail.event);
    println!("Date:      {}", detail.timestamp);
    if !detail.tagger.is_empty() {
        println!("Tagger:    {}", detail.tagger);
    }
    if !detail.message.is_empty() {
        println!("Message:   {}", detail.message);
    }
}
