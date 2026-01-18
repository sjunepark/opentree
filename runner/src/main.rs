use std::path::Path;

use anyhow::Result;
use clap::{Parser, Subcommand};

use runner::exit_codes;
use runner::io::config::load_config;
use runner::io::executor::CodexExecutor;
use runner::io::guards::CommandGuardRunner;
use runner::io::init::{InitOptions, init_runner};
use runner::looping::{LoopStop, run_loop};
use runner::select::{SelectOutcome, select_from_root};
use runner::start::start_run;
use runner::step::{StepConfig, StuckLeafError, run_step};
use runner::validate::{RunValidation, validate_runner};

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
    /// Validate `.runner/` layout, config, tree, and run identity.
    Validate,
    /// Print the next selected leaf (or complete/stuck).
    Select,
    /// Execute one deterministic iteration (`runner step`).
    Step {
        /// Prompt pack size budget in bytes.
        #[arg(long, default_value_t = StepConfig::default().prompt_budget_bytes)]
        prompt_budget: usize,
    },
    /// Execute iterations until complete/stuck/limit (`runner loop`).
    Loop {
        /// Prompt pack size budget in bytes.
        #[arg(long, default_value_t = StepConfig::default().prompt_budget_bytes)]
        prompt_budget: usize,
    },
}

fn main() -> Result<()> {
    runner::logging::init();
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
        Command::Validate => match validate_runner(Path::new(".")) {
            Ok(outcome) => {
                println!("validate: layout=ok");
                println!("validate: config=ok");
                println!("validate: tree=ok");
                match outcome.run {
                    RunValidation::NotStarted => {
                        println!("validate: run=not-started");
                    }
                    RunValidation::Ok { run_id, branch } => {
                        println!("validate: run=ok id={run_id} branch={branch}");
                    }
                }
                std::process::exit(exit_codes::OK);
            }
            Err(err) => {
                eprintln!("{err}");
                std::process::exit(exit_codes::INVALID);
            }
        },
        Command::Select => match select_from_root(Path::new("."))? {
            SelectOutcome::Complete => {
                println!("select: status=complete");
                std::process::exit(exit_codes::COMPLETE);
            }
            SelectOutcome::Open(leaf) => {
                println!(
                    "select: status=open id={} path={} attempts={}/{}",
                    leaf.id, leaf.path, leaf.attempts, leaf.max_attempts
                );
                std::process::exit(exit_codes::OK);
            }
            SelectOutcome::Stuck(leaf) => {
                println!(
                    "select: status=stuck id={} path={} attempts={}/{}",
                    leaf.id, leaf.path, leaf.attempts, leaf.max_attempts
                );
                std::process::exit(exit_codes::STUCK);
            }
        },
        Command::Step { prompt_budget } => {
            let executor = CodexExecutor;
            let state_dir = Path::new(".").join(".runner").join("state");
            let cfg = load_config(&state_dir.join("config.toml"))?;
            let guard_runner = CommandGuardRunner::new(cfg.guard.command);
            let outcome = match run_step(
                Path::new("."),
                &executor,
                &guard_runner,
                &StepConfig {
                    prompt_budget_bytes: prompt_budget,
                },
            ) {
                Ok(outcome) => outcome,
                Err(err) => {
                    if let Some(stuck) = err.downcast_ref::<StuckLeafError>() {
                        eprintln!("{stuck}");
                        std::process::exit(exit_codes::STUCK);
                    }
                    return Err(err);
                }
            };
            println!(
                "step: run={} iter={} node={} status={:?} guard={:?}",
                outcome.run_id, outcome.iter, outcome.selected_id, outcome.status, outcome.guard
            );
        }
        Command::Loop { prompt_budget } => {
            let executor = CodexExecutor;
            let state_dir = Path::new(".").join(".runner").join("state");
            let cfg = load_config(&state_dir.join("config.toml"))?;
            let guard_runner = CommandGuardRunner::new(cfg.guard.command);

            let outcome = run_loop(
                Path::new("."),
                &executor,
                &guard_runner,
                &StepConfig {
                    prompt_budget_bytes: prompt_budget,
                },
                |step| {
                    println!(
                        "loop: step run={} iter={} node={} status={:?} guard={:?}",
                        step.run_id, step.iter, step.selected_id, step.status, step.guard
                    );
                },
            )?;

            match outcome.stop {
                LoopStop::Complete => {
                    println!(
                        "loop: status=complete run={} steps={} started_at_iter={}",
                        outcome.run_id, outcome.steps_executed, outcome.started_at_iter
                    );
                    std::process::exit(exit_codes::OK);
                }
                LoopStop::Stuck {
                    id,
                    path,
                    attempts,
                    max_attempts,
                } => {
                    println!(
                        "loop: status=stuck run={} id={} path={} attempts={}/{}",
                        outcome.run_id, id, path, attempts, max_attempts
                    );
                    std::process::exit(exit_codes::STUCK);
                }
                LoopStop::MaxIterationsExceeded {
                    next_iter,
                    max_iterations,
                } => {
                    println!(
                        "loop: status=limit run={} next_iter={} max_iterations={} steps={} started_at_iter={}",
                        outcome.run_id,
                        next_iter,
                        max_iterations,
                        outcome.steps_executed,
                        outcome.started_at_iter
                    );
                    std::process::exit(exit_codes::INVALID);
                }
            }
        }
    }
    Ok(())
}
