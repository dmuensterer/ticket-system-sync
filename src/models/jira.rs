use axum::{Json, Router, extract::Path, routing::post};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use tracing::{error, info, instrument};

use super::{
    db::DB,
    zammad::ZammadState,
    zammad_api::{ZammadAddCommentRequest, ZammadUpdateTicketRequest},
};
use crate::models::jira_webhook::JiraWebhook;

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
async fn create_ticket(id: String, webhook: JiraWebhook) -> anyhow::Result<()> {
    // TODO: Implement Jira to Zammad ticket creation
    Ok(())
}

#[instrument(skip(payload))]
#[axum::debug_handler]
async fn create_ticket_handler(
    Path(id): Path<String>,
    Json(payload): Json<JiraWebhook>,
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
async fn update_ticket(webhook: JiraWebhook) -> anyhow::Result<()> {
    let db = DB::new().await?;

    // Get the Jira issue ID from the webhook
    let jira_issue_id = webhook
        .jira_webhook_issue
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("No issue information in webhook"))?
        .id;

    // Get the corresponding Zammad ticket ID
    let zammad_id = db.get_zammad_id_by_jira_id(&jira_issue_id).await?;

    // If there's a comment in the webhook, we should add it to Zammad
    if let Some(comment) = webhook.jira_webhook_comment {
        ZammadAddCommentRequest::from_jira_comment(&comment)
            .submit(&zammad_id)
            .await?;
    }

    //    // If there's a changelog, we should update the Zammad ticket
    //    if webhook.jira_webhook_changelog.is_some() || webhook.jira_webhook_issue.is_some() {
    //        ZammadUpdateTicketRequest::from_jira_webhook(&webhook)
    //            .submit(&zammad_id)
    //            .await?;
    //    }

    Ok(())
}

#[instrument(skip(payload))]
#[axum::debug_handler]
async fn update_ticket_handler(
    Path(id): Path<String>,
    Json(payload): Json<JiraWebhook>,
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
