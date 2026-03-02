mod model;
mod ui;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "ishoo",
    about = "Portable markdown issue tracker with desktop UI"
)]
struct Cli {
    #[arg(short, long, default_value = ".")]
    path: PathBuf,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new issue tracker (creates docs/issues/)
    Init,
    List {
        #[arg(short, long)]
        filter: Option<String>,
    },
    Show {
        id: u32,
    },
    Set {
        id: u32,
        status: String,
    },
    Heatmap,
    Dash,
    New {
        title: String,
        #[arg(short, long, default_value = "open")]
        status: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Init) => match model::init_workspace(&cli.path) {
            Ok(created_path) => {
                println!("✓ Initialized ishoo in {}", created_path.display());
                println!("  Created: issues-active.md, issues-backlog.md, issues-done.md");
            }
            Err(e) => {
                eprintln!("Error: {e}");
                std::process::exit(1);
            }
        },
        _ => {
            let path = model::discover_root(&cli.path);
            match &cli.command {
                Some(Commands::Dash) | None => {
                    ui::launch_dashboard(path);
                }
                Some(Commands::List { filter }) => {
                    let ws = model::Workspace::load(&path).expect("Failed to load workspace");
                    model::cli_list(&ws, filter.as_deref());
                }
                Some(Commands::Show { id }) => {
                    let ws = model::Workspace::load(&path).expect("Failed to load workspace");
                    model::cli_show(&ws, *id);
                }
                Some(Commands::Set { id, status }) => {
                    let mut ws = model::Workspace::load(&path).expect("Failed to load workspace");
                    model::cli_set_status(&mut ws, *id, status).expect("Failed to set status");
                }
                Some(Commands::Heatmap) => {
                    let ws = model::Workspace::load(&path).expect("Failed to load workspace");
                    model::cli_heatmap(&ws);
                }
                Some(Commands::New { title, status }) => {
                    let mut ws = model::Workspace::load(&path).expect("Failed to load workspace");
                    let max_id = ws.issues.iter().map(|i| i.id).max().unwrap_or(0);
                    let issue = model::Issue {
                        id: max_id + 1,
                        title: title.clone(),
                        status: model::Status::from_str(status),
                        files: vec![],
                        description: String::new(),
                        resolution: String::new(),
                        section: "ACTIVE Issues".to_string(),
                        depends_on: vec![],
                    };
                    println!("Created [{}] {}", issue.id, issue.title);
                    ws.issues.push(issue);
                    ws.save().expect("Failed to save");
                }
                Some(Commands::Init) => unreachable!(),
            }
        }
    }
}
