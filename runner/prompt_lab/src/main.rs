//! Prompt Laboratory CLI for testing prompt variants.
//!
//! Provides commands to run prompt × input combinations and serve results via dashboard.

mod cache;
mod render;
mod runner;

use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing::info;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

#[derive(Parser)]
#[command(name = "prompt-lab")]
#[command(about = "Prompt Laboratory - test prompt variants against fixed inputs")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run prompt × input combinations for an agent
    Run {
        /// Agent name (e.g., tree_agent)
        agent: String,

        /// Force re-run even if cached
        #[arg(long)]
        force: bool,
    },

    /// Serve the dashboard (static file server)
    Serve {
        /// Port to serve on
        #[arg(short, long, default_value = "3030")]
        port: u16,
    },

    /// List available prompts and inputs
    List {
        /// Agent name (e.g., tree_agent)
        agent: Option<String>,
    },
}

fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();
    let lab_root = find_lab_root()?;

    match cli.command {
        Commands::Run { agent, force } => {
            info!(agent = %agent, force = force, "running combinations");
            runner::run_agent(&lab_root, &agent, force)?;
        }
        Commands::Serve { port } => {
            info!(port = port, "serving dashboard");
            serve_dashboard(&lab_root, port)?;
        }
        Commands::List { agent } => {
            list_resources(&lab_root, agent.as_deref())?;
        }
    }

    Ok(())
}

/// Find the prompt_lab root directory (where prompts/ and inputs/ live).
fn find_lab_root() -> Result<PathBuf> {
    // First check compile-time manifest directory (most reliable)
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    if manifest_dir.join("prompts").exists() && manifest_dir.join("inputs").exists() {
        return Ok(manifest_dir);
    }

    // Check cwd and relative paths
    let cwd = std::env::current_dir().ok();
    let candidates = [
        cwd.clone(),
        cwd.as_ref().map(|p| p.join("runner/prompt_lab")),
        cwd.as_ref().map(|p| p.join("prompt_lab")),
    ];

    for candidate in candidates.into_iter().flatten() {
        if candidate.join("prompts").exists() && candidate.join("inputs").exists() {
            return Ok(candidate);
        }
    }

    // Default to manifest dir even if prompts/inputs don't exist yet
    Ok(manifest_dir)
}

/// Serve static files for dashboard.
fn serve_dashboard(lab_root: &PathBuf, port: u16) -> Result<()> {
    let dashboard_dir = lab_root.join("dashboard").join("build");
    let results_dir = lab_root.join("results");

    if !dashboard_dir.exists() {
        anyhow::bail!(
            "Dashboard not built. Run `cd dashboard && bun run build` first.\n\
             Expected: {}",
            dashboard_dir.display()
        );
    }

    println!("Serving dashboard at http://localhost:{}", port);
    println!("  Dashboard: {}", dashboard_dir.display());
    println!("  Results: {}", results_dir.display());
    println!("\nFor development, use `cd dashboard && bun dev` instead.");

    // Simple static file serving - for production use, recommend nginx or similar
    // For now, just print instructions
    println!(
        "\nTo serve manually:\n  cd {} && python3 -m http.server {}",
        dashboard_dir.display(),
        port
    );

    Ok(())
}

/// List available prompts and inputs.
fn list_resources(lab_root: &PathBuf, agent: Option<&str>) -> Result<()> {
    let prompts_dir = lab_root.join("prompts");
    let inputs_dir = lab_root.join("inputs");

    if let Some(agent_name) = agent {
        // List for specific agent
        let agent_prompts = prompts_dir.join(agent_name);
        let agent_inputs = inputs_dir.join(agent_name);

        println!("Agent: {}", agent_name);
        println!("\nPrompts ({}):", agent_prompts.display());
        if agent_prompts.exists() {
            for entry in std::fs::read_dir(&agent_prompts)? {
                let entry = entry?;
                if entry.path().extension().map_or(false, |e| e == "md") {
                    println!("  - {}", entry.file_name().to_string_lossy());
                }
            }
        } else {
            println!("  (none)");
        }

        println!("\nInputs ({}):", agent_inputs.display());
        if agent_inputs.exists() {
            for entry in std::fs::read_dir(&agent_inputs)? {
                let entry = entry?;
                if entry.path().extension().map_or(false, |e| e == "json") {
                    println!("  - {}", entry.file_name().to_string_lossy());
                }
            }
        } else {
            println!("  (none)");
        }
    } else {
        // List all agents
        println!("Available agents:\n");

        if prompts_dir.exists() {
            for entry in std::fs::read_dir(&prompts_dir)? {
                let entry = entry?;
                if entry.file_type()?.is_dir() {
                    println!("  {}/", entry.file_name().to_string_lossy());
                }
            }
        }

        println!("\nUse `prompt-lab list <agent>` for details.");
    }

    Ok(())
}
