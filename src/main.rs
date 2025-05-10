mod models;

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::post,
};
use models::zammad::ZammadSystem;
use models::{jira::JiraSystem, ticketsystem::TicketSystem};
use serde_json::Value;

use async_trait::async_trait;
use clap::Parser;
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tracing::{error, info};

/// ----------------------------------------------------------------------
/// 1  Kommandozeilen-Argumente
/// ----------------------------------------------------------------------
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

/// ----------------------------------------------------------------------
/// 2  Gemeinsamer App-State
/// ----------------------------------------------------------------------
struct AppState {
    id_map: HashMap<String, Box<dyn TicketSystem>>,
}

/// Jira ticket system implementation

/// ----------------------------------------------------------------------
/// 3  API Endpoints
/// ----------------------------------------------------------------------
#[async_trait]
pub trait APIEndpoint: Send + Sync {
    fn endpoint_type(&self) -> &'static str;
    async fn process_webhook(
        &self,
        system: &dyn TicketSystem,
        payload: Value,
    ) -> Result<(), String>;
}

/// ----------------------------------------------------------------------
/// 4  Programmstart
/// ----------------------------------------------------------------------
#[tokio::main]
async fn main() {
    // a) Logging
    tracing_subscriber::fmt().with_env_filter("info").init();

    // b) CLI
    let cli = Cli::parse();

    // c) State vorbereiten (Map ID → TicketSystem)
    let mut id_map: HashMap<String, Box<dyn TicketSystem>> = HashMap::new();
    id_map.insert(cli.zammad_id.clone(), Box::new(ZammadSystem));
    id_map.insert(cli.jira_id.clone(), Box::new(JiraSystem));
    let state = Arc::new(AppState { id_map });

    // d) Router
    let app = Router::new()
        .route("/ticket-sync/create-ticket/:id", post(create_ticket))
        .route("/ticket-sync/add-comment/:id", post(add_comment))
        .with_state(state);

    // e) Server
    let addr = SocketAddr::from(([0, 0, 0, 0], cli.port));
    info!("Listening on http://{addr}/ticket-sync/{{create-ticket, add-comment}}/<id>");
    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app)
        .await
        .expect("server error");
}

/// ----------------------------------------------------------------------
/// 5  Handler
/// ----------------------------------------------------------------------
#[tracing::instrument(skip(state, payload))]
async fn create_ticket(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<Value>,
) -> StatusCode {
    info!("creating ticket for ID: {}", id);

    let Some(ticket_system) = state.id_map.get(&id) else {
        error!("unknown system ID: {}", id);
        return StatusCode::NOT_FOUND;
    };

    match ticket_system.create_ticket(payload).await {
        Ok(_) => StatusCode::OK,
        Err(error) => {
            error!(%error, "error creating ticket for ID: {}", id);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

#[tracing::instrument(skip(state, payload))]
async fn add_comment(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<Value>,
) -> StatusCode {
    info!("adding comment to ticket for ID: {}", id);

    let Some(ticket_system) = state.id_map.get(&id) else {
        error!("unknown system ID: {}", id);
        return StatusCode::NOT_FOUND;
    };

    match ticket_system.add_comment(payload).await {
        Ok(_) => StatusCode::OK,
        Err(error) => {
            error!(%error, "error adding comment to ticket for ID: {} error: {}", id, error);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
