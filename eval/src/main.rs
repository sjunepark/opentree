//! Evaluation harness for running runner loops against declarative test cases.
//!
//! This crate provides tooling for local experimentation with real runner loops.
//! Cases are defined in TOML format and specify goals, configuration overrides,
//! and verification checks.
//!
//! # Commands
//!
//! - `eval list` — List available cases
//! - `eval run <case-id> [--runs N]` — Run a case (optionally multiple times)
//! - `eval report <case-id>` — Show aggregated results
//! - `eval clean <case-id>` — Remove workspaces and results
//!
//! # Architecture
//!
//! - [`case`] — Case file parsing and validation
//! - [`workspace`] — Isolated workspace creation
//! - [`harness`] — Runner binary building and execution
//! - [`judge`] — Check execution and outcome recording
//! - [`outcome`] — Outcome classification (success/fail/stuck/error)
//! - [`results`] — Result capture and persistence
//! - [`report`] — Result aggregation and reporting

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
