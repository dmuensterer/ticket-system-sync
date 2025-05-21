mod config;
mod models;

use std::net::SocketAddr;

use axum::Router;
use models::{
    jira,
    zammad::{self},
};

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Cli {
    /// ID, die Zammad in der Webhook-URL benutzt
    #[arg(long, env = "ZAMMAD_ID")]
    zammad_id: String,

    /// ID, die Jira (CUN) in der Webhook-URL benutzt
    #[arg(long, env = "JIRA_ID")]
    jira_id: String,

    /// Port (Default 8080)
    #[arg(short, long, default_value_t = 8000)]
    port: u16,
}

#[tokio::main]
async fn main() {
    config::init();

    // a) Logging
    tracing_subscriber::fmt().with_env_filter("info").init();

    // b) CLI
    let cli = Cli::parse();

    // d) Router
    let app = Router::new()
        .nest("/ticket-sync/zammad", zammad::router())
        .nest("/ticket-sync/jira", jira::router());

    // e) Server
    let addr = SocketAddr::from(([0, 0, 0, 0], cli.port));
    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app)
        .await
        .expect("server error");
}
