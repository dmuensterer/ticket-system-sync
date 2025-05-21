use axum::{Json, Router, extract::Path, routing::post};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use tracing::{error, instrument};

use super::zammad::ZammadState;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JiraWebhook<T> {
    pub issue: T,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JiraIssue {
    pub project: JiraProject,
    /// Additional fields for the ticket
    pub fields: JiraFields,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JiraProject {
    pub id: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JiraFields {
    pub project: JiraProject,
    pub summary: String,
    pub description: String,
    pub issuetype: JiraIssueType,
    pub priority: JiraPriority,
    pub duedate: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JiraIssueType {
    pub name: String,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JiraPriority {
    pub name: JiraPriorityEnum,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum JiraPriorityEnum {
    Highest = 1,
    High = 2,
    Medium = 3,
    Low = 4,
    Lowest = 5,
}

#[derive(Debug, Serialize, Clone, Copy)]
pub enum JiraStatus {
    Open,
    Closed,
}

impl JiraStatus {
    pub fn from_zammad_state(state: ZammadState) -> JiraStatus {
        match state {
            ZammadState::Open => JiraStatus::Open,
            ZammadState::Closed => JiraStatus::Closed,
        }
    }
}

#[instrument(skip(webhook))]
async fn create_ticket(id: String, webhook: JiraWebhook<JiraIssue>) -> anyhow::Result<()> {
    // TODO: Implement Jira to Zammad ticket creation
    Ok(())
}

#[instrument(skip(payload))]
#[axum::debug_handler]
async fn create_ticket_handler(
    Path(id): Path<String>,
    Json(payload): Json<JiraWebhook<JiraIssue>>,
) -> StatusCode {
    match create_ticket(id, payload).await {
        Ok(_) => StatusCode::OK,
        Err(e) => {
            error!("Failed to create ticket: {}", e);
            StatusCode::BAD_REQUEST
        }
    }
}

#[instrument(skip(webhook))]
async fn update_ticket(webhook: JiraWebhook<JiraIssue>) -> anyhow::Result<()> {
    // TODO: Implement Jira to Zammad ticket update
    Ok(())
}

#[instrument(skip(payload))]
#[axum::debug_handler]
async fn update_ticket_handler(
    Path(id): Path<String>,
    Json(payload): Json<JiraWebhook<JiraIssue>>,
) -> StatusCode {
    match update_ticket(payload).await {
        Ok(_) => StatusCode::OK,
        Err(e) => {
            error!("Failed to update ticket: {}", e);
            StatusCode::BAD_REQUEST
        }
    }
}

pub fn router() -> Router {
    // Using specific Router<()> type to ensure we don't need state
    let router: Router<()> = Router::new()
        .route("/create-ticket/:id", post(create_ticket_handler))
        .route("/update-ticket/:id", post(update_ticket_handler));
    router
}
