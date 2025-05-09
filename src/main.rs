mod models;

use axum::{
    extract::{Path, State}, http::StatusCode, routing::post, Json, Router
};
use models::{jira::JiraSystem, ticketsystem::TicketSystem};
use models::zammad::ZammadSystem;
use serde_json::Value;

use clap::Parser;
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tracing::{error, info};
use async_trait::async_trait;

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
    async fn process_webhook(&self, system: &dyn TicketSystem, payload: Value) -> Result<(), String>;
}

pub struct CreateTicketEndpoint;

#[async_trait]
impl APIEndpoint for CreateTicketEndpoint {
    fn endpoint_type(&self) -> &'static str {
        "create-ticket"
    }

    async fn process_webhook(&self, system: &dyn TicketSystem, payload: Value) -> Result<(), String> {
        info!("Processing create ticket request for {}", system.name());
        system.process_webhook(payload).await
    }
}

pub struct AddCommentEndpoint;

#[async_trait]
impl APIEndpoint for AddCommentEndpoint {
    fn endpoint_type(&self) -> &'static str {
        "add-comment"
    }

    async fn process_webhook(&self, system: &dyn TicketSystem, payload: Value) -> Result<(), String> {
        info!("Processing add comment request for {}", system.name());
        system.process_webhook(payload).await
    }
}

fn create_endpoint(request_type: &str) -> Option<Box<dyn APIEndpoint>> {
    match request_type {
        "create-ticket" => Some(Box::new(CreateTicketEndpoint)),
        "add-comment" => Some(Box::new(AddCommentEndpoint)),
        _ => None,
    }
}

/// ----------------------------------------------------------------------
/// 4  Programmstart
/// ----------------------------------------------------------------------
#[tokio::main]
async fn main() {
    // a) Logging
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    // b) CLI
    let cli = Cli::parse();

    // c) State vorbereiten (Map ID → TicketSystem)
    let mut id_map: HashMap<String, Box<dyn TicketSystem>> = HashMap::new();
    id_map.insert(cli.zammad_id.clone(), Box::new(ZammadSystem));
    id_map.insert(cli.jira_id.clone(), Box::new(JiraSystem));
    let state = Arc::new(AppState { id_map });

    // d) Router
    let app = Router::new()
        .route("/ticket-sync/:request_type/:id", post(webhook))
        .with_state(state);

    // e) Server
    let addr = SocketAddr::from(([0, 0, 0, 0], cli.port));
    info!("Listening on http://{addr}/ticket-sync/<request_type>/<id>");
    axum::serve(
        tokio::net::TcpListener::bind(addr).await.unwrap(),
        app
    )
    .await
    .expect("server error");
}

/// ----------------------------------------------------------------------
/// 5  Handler
/// ----------------------------------------------------------------------
async fn webhook(
    Path((request_type, id)): Path<(String, String)>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<Value>,
) -> StatusCode {
    // First, validate the request type and create the appropriate endpoint
    let endpoint = match create_endpoint(&request_type) {
        Some(endpoint) => endpoint,
        None => {
            error!("Invalid request type: {}", request_type);
            return StatusCode::BAD_REQUEST;
        }
    };

    // Then, get the appropriate ticket system
    match state.id_map.get(&id) {
        Some(system) => {
            match endpoint.process_webhook(system.as_ref(), payload).await {
                Ok(_) => StatusCode::OK,
                Err(e) => {
                    error!("Error processing webhook: {}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                }
            }
        }
        None => {
            error!("Unknown system ID: {}", id);
            StatusCode::NOT_FOUND
        }
    }
}