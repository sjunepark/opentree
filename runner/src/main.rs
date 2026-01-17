use std::path::Path;

use anyhow::Result;
use clap::{Parser, Subcommand};

use runner::io::config::load_config_with_legacy_fallback;
use runner::io::executor::CodexExecutor;
use runner::io::guards::CommandGuardRunner;
use runner::io::init::{InitOptions, init_runner};
use runner::start::start_run;
use runner::step::{StepConfig, run_step};

#[derive(Parser)]
#[command(author, version, about = "Deterministic goal-driven agent loop runner")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Initialize `.runner/` scaffolding in the current repo.
    Init {
        /// Overwrite known runner-owned artifacts if they already exist.
        #[arg(long)]
        force: bool,
    },
    /// Start a run (creates branch + sets run-id + commits bootstrap).
    Start,
    /// Execute one deterministic iteration (`runner step`).
    Step {
        /// Prompt pack size budget in bytes.
        #[arg(long, default_value_t = StepConfig::default().prompt_budget_bytes)]
        prompt_budget: usize,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Init { force } => {
            init_runner(Path::new("."), &InitOptions { force })?;
            println!("initialized .runner/");
        }
        Command::Start => {
            let outcome = start_run(Path::new("."))?;
            println!("started run={} branch={}", outcome.run_id, outcome.branch);
        }
        Command::Step { prompt_budget } => {
            let executor = CodexExecutor;
            let state_dir = Path::new(".").join(".runner").join("state");
            let cfg = load_config_with_legacy_fallback(
                &state_dir.join("config.toml"),
                &state_dir.join("config.json"),
            )?;
            let guard_runner = CommandGuardRunner::new(cfg.guard.command);
            let outcome = run_step(
                Path::new("."),
                &executor,
                &guard_runner,
                &StepConfig {
                    prompt_budget_bytes: prompt_budget,
                },
            )?;
            println!(
                "step: run={} iter={} node={} status={:?} guard={:?}",
                outcome.run_id, outcome.iter, outcome.selected_id, outcome.status, outcome.guard
            );
        }
    }
    Ok(())
}
