use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::post,
    Json, Router,
};
use clap::Parser;
use serde::{Deserialize, Serialize};
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
    #[arg(short, long, default_value_t = 8080)]
    port: u16,
}

/// ----------------------------------------------------------------------
/// 2  Gemeinsamer App-State
/// ----------------------------------------------------------------------
#[derive(Clone)]
struct AppState {
    id_map: HashMap<String, TicketSystem>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum TicketSystem {
    Zammad,
    Jira,
}

/// ----------------------------------------------------------------------
/// 3  Datentypen – eingehend (Zammad) + ausgehend (Jira)
/// ----------------------------------------------------------------------
#[derive(Debug, Deserialize)]
struct ZammadEvent {
    event: String,
    #[serde(flatten)]
    extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize)]
struct JiraCreateRequest {
    fields: JiraFields,
}
#[derive(Debug, Serialize)]
struct JiraFields {
    project: JiraProject,
    summary: String,
    description: String,
    #[serde(rename = "issuetype")]
    issue_type: JiraIssueType,
}
#[derive(Debug, Serialize)]
struct JiraProject {
    key: String,
}
#[derive(Debug, Serialize)]
struct JiraIssueType {
    name: String,
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
    let mut id_map = HashMap::new();
    id_map.insert(cli.zammad_id.clone(), TicketSystem::Zammad);
    id_map.insert(cli.jira_id.clone(), TicketSystem::Jira);
    let state = Arc::new(AppState { id_map });

    // d) Router
    let app = Router::new()
        // → POST /ticket-sync/<id>
        .route("/ticket-sync/:id", post(webhook))
        .with_state(state);

    // e) Server
    let addr = SocketAddr::from(([0, 0, 0, 0], cli.port));
    info!("Listening on http://{addr}/ticket-sync/<id>");
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
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<serde_json::Value>,
) -> StatusCode {
    match state.id_map.get(&id) {
        // --------------------------------------------------------------
        // a) Zammad → Jira  (One-Way)
        // --------------------------------------------------------------
        Some(TicketSystem::Zammad) => {
            match serde_json::from_value::<ZammadEvent>(payload) {
                Ok(evt) => {
                    info!("Zammad-Event {:?}", evt.event);

                    // Nur ein minimaler Mapper:
                    let jira_req = JiraCreateRequest {
                        fields: JiraFields {
                            project: JiraProject { key: "CUN".into() },
                            summary: format!("Zammad {}", evt.event),
                            description: format!(
                                "Importiert von Zammad-Hook\n\n{:?}",
                                evt.extra
                            ),
                            issue_type: JiraIssueType { name: "Task".into() },
                        },
                    };

                    // Dry-run: nur ausgeben
                    info!("→ Jira-Payload\n{}", serde_json::to_string_pretty(&jira_req).unwrap());

                    // TODO: reqwest-POST   (nächster Schritt)
                    StatusCode::OK
                }
                Err(e) => {
                    error!("JSON Fehler {}", e);
                    StatusCode::BAD_REQUEST
                }
            }
        }

        // --------------------------------------------------------------
        // b) Jira schickt einen Hook (künftig für Bidirektionalität)
        // --------------------------------------------------------------
        Some(TicketSystem::Jira) => {
            info!("(noch) ignorierter Jira-Webhook");
            StatusCode::NO_CONTENT
        }

        // --------------------------------------------------------------
        // c) Unbekannte ID
        // --------------------------------------------------------------
        None => {
            error!("Unbekannte ID {}", id);
            StatusCode::NOT_FOUND
        }
    }
}
