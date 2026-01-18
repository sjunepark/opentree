mod case;
mod cli;
mod config;
mod harness;
mod judge;
mod outcome;
mod report;
mod results;
mod run;
mod workspace;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "eval", version, about = "Evaluation harness for runner")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    List,
    Run {
        case_id: String,
        #[arg(long, default_value_t = 1)]
        runs: u32,
    },
    Report {
        case_id: String,
    },
    Clean {
        case_id: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let repo_root = std::env::current_dir()?;
    match cli.command {
        Command::List => cli::list_cases(&repo_root),
        Command::Run { case_id, runs } => cli::run_case_by_id(&repo_root, &case_id, runs),
        Command::Report { case_id } => cli::report_case(&repo_root, &case_id),
        Command::Clean { case_id } => cli::clean_case(&repo_root, &case_id),
    }
}
