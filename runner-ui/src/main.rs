//! Runner UI server - read-only web interface for monitoring runner state.

mod routes;
mod sse;
mod state;

use std::net::SocketAddr;
use std::path::PathBuf;

use axum::Router;
use axum::routing::get;
use clap::Parser;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tracing::info;

use crate::state::AppState;

#[derive(Parser)]
#[command(name = "runner-ui")]
#[command(about = "Read-only web UI for monitoring runner state")]
struct Args {
    /// Address to bind the server to
    #[arg(long, default_value = "127.0.0.1")]
    bind: String,

    /// Port to listen on
    #[arg(long, default_value = "3001")]
    port: u16,

    /// Project directory (contains .runner/)
    #[arg(long, default_value = ".")]
    project_dir: PathBuf,

    /// Directory containing UI static files (defaults to ./ui/dist relative to binary)
    #[arg(long)]
    ui_dir: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("runner_ui=info".parse()?),
        )
        .init();

    let args = Args::parse();

    let project_dir = args.project_dir.canonicalize().unwrap_or(args.project_dir);
    info!(project_dir = %project_dir.display(), "starting runner-ui");

    let state = AppState::new(project_dir.clone());

    // Start file watcher
    sse::start_file_watcher(state.clone());

    // Build router
    let api_router = routes::api_router();

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let mut app = Router::new()
        .nest("/api", api_router)
        .route("/events", get(sse::events_handler))
        .layer(cors)
        .with_state(state);

    // Serve static UI files if available
    let ui_dir = args.ui_dir.unwrap_or_else(|| {
        // Default to ui/dist relative to project directory
        project_dir.join("ui").join("dist")
    });

    if ui_dir.exists() {
        info!(ui_dir = %ui_dir.display(), "serving static UI files");
        app = app.fallback_service(ServeDir::new(ui_dir).append_index_html_on_directories(true));
    } else {
        info!(ui_dir = %ui_dir.display(), "UI directory not found, API-only mode");
    }

    let addr: SocketAddr = format!("{}:{}", args.bind, args.port).parse()?;
    info!(addr = %addr, "listening");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
