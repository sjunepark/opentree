use std::path::Path;

use anyhow::Result;
use clap::{Parser, Subcommand};

use runner::io::executor::CodexExecutor;
use runner::io::guards::JustGuardRunner;
use runner::io::init::{InitOptions, init_runner};
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
        Command::Step { prompt_budget } => {
            let executor = CodexExecutor;
            let guard_runner = JustGuardRunner;
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
