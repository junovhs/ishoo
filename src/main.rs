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
        id: String,
    },
    Set {
        id: String,
        status: String,
    },
    Delete {
        id: String,
        #[arg(short, long)]
        force: bool,
    },
    Lint {
        #[arg(long)]
        strict: bool,
    },
    Heatmap,
    Dash,
    New {
        title: String,
        #[arg(long, default_value = "iss")]
        category: String,
        #[arg(short, long, default_value = "open")]
        status: String,
    },
}

fn main() {
    let cli = Cli::parse();

    let exit_code = run(cli);
    if exit_code != 0 {
        std::process::exit(exit_code);
    }
}

fn run(cli: Cli) -> i32 {
    match &cli.command {
        Some(Commands::Init) => match model::init_workspace(&cli.path) {
            Ok(created_path) => {
                println!("✓ Initialized ishoo in {}", created_path.display());
                println!("  Created: issues-active.md, issues-backlog.md, issues-done.md");
                0
            }
            Err(e) => {
                eprintln!("Error: {e}");
                1
            }
        },
        _ => {
            let path = model::discover_root(&cli.path);
            match &cli.command {
                Some(Commands::Dash) | None => {
                    ui::launch_dashboard(path);
                    0
                }
                Some(Commands::List { filter }) => {
                    let ws = model::Workspace::load(&path).expect("Failed to load workspace");
                    model::cli_list(&ws, filter.as_deref());
                    0
                }
                Some(Commands::Show { id }) => {
                    let ws = model::Workspace::load(&path).expect("Failed to load workspace");
                    model::cli_show(&ws, id);
                    0
                }
                Some(Commands::Set { id, status }) => {
                    let mut ws = model::Workspace::load(&path).expect("Failed to load workspace");
                    model::cli_set_status(&mut ws, id, status).expect("Failed to set status");
                    0
                }
                Some(Commands::Delete { id, force }) => {
                    let mut ws = model::Workspace::load(&path).expect("Failed to load workspace");
                    model::cli_delete(&mut ws, id, *force).expect("Failed to delete issue");
                    0
                }
                Some(Commands::Lint { strict }) => match model::cli_lint(&path, *strict) {
                    Ok(()) => 0,
                    Err(err) => {
                        eprintln!("Error: {err}");
                        1
                    }
                },
                Some(Commands::Heatmap) => {
                    let ws = model::Workspace::load(&path).expect("Failed to load workspace");
                    model::cli_heatmap(&ws);
                    0
                }
                Some(Commands::New {
                    title,
                    category,
                    status,
                }) => {
                    let mut ws = model::Workspace::load(&path).expect("Failed to load workspace");
                    let issue = model::Issue {
                        id: ws
                            .allocate_issue_id(category)
                            .expect("Failed to allocate issue ID"),
                        title: title.clone(),
                        status: model::Status::from_str(status),
                        files: vec![],
                        labels: vec![],
                        links: vec![],
                        description: String::new(),
                        resolution: String::new(),
                        section: "ACTIVE Issues".to_string(),
                        depends_on: vec![],
                    };
                    println!("Created [{}] {}", issue.id, issue.title);
                    ws.issues.push(issue);
                    ws.save().expect("Failed to save");
                    0
                }
                Some(Commands::Init) => unreachable!(),
            }
        }
    }
}
