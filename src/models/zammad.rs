use super::{
    api_request::{JiraAddCommentRequest, JiraUpdateIssueRequest},
    db::DB,
};
use std::sync::Arc;

use anyhow;
use async_trait::async_trait;
use axum::{Json, Router, extract::Path, routing::post};
use chrono::{DateTime, Utc};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_repr::Deserialize_repr;
use tracing::{debug, error, info};

use crate::models::{api_request::JiraCreateIssueRequest, assignment::Assignment, jira::JiraIssue};

/// Represents a Zammad webhook payload containing both ticket and article information.
/// This is the top-level structure that Zammad sends when a ticket is created or updated.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ZammadWebhook {
    /// The ticket information including metadata, state, and relationships
    pub ticket: ZammadTicket,
    /// The article (comment/message) associated with this ticket update
    pub article: ZammadArticle,
}

/// Represents a Zammad ticket with all its metadata and relationships.
/// This structure contains all the essential information needed to create/update
/// a corresponding ticket in another system (like Jira).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ZammadTicket {
    /// Unique identifier for the ticket in Zammad
    pub id: i32,
    /// Human-readable ticket number (e.g., "45003")
    pub number: String,
    /// The ticket's title/subject (e.g., "Test Ticket")
    pub title: String,

    pub state: ZammadState,
    /// Priority information including name and ID
    pub priority: ZammadPriority,
    /// When the ticket was created
    pub created_at: DateTime<Utc>,
    /// When the ticket was last updated
    pub updated_at: DateTime<Utc>,
    /// Optional due date for the ticket
    pub due_date: DateTime<Utc>,
    /// User who created the ticket
    pub created_by: ZammadUser,
    /// User who is currently assigned to the ticket
    pub owner: ZammadUser,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ZammadPriority {
    pub id: ZammadPriorityId,
}

/// Represents a Zammad priority level.
/// Example: "2 normal" with ID 2

#[repr(i32)] // store the enum as an 32-bit integer
#[derive(Debug, Serialize, Deserialize_repr, Clone, Copy)]
pub enum ZammadPriorityId {
    /// Unique identifier for the priority
    Low = 1,
    Normal = 2,
    High = 3,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
// We're expecting either "open" or "closed" as a string. Need to deserialize it to the enum.
#[serde(rename_all = "lowercase")]
pub enum ZammadState {
    Open,
    Closed,
}

impl ZammadState {
    pub fn from_str(state: String) -> Result<ZammadState, String> {
        match state.as_str() {
            "open" => Ok(ZammadState::Open),
            "closed" => Ok(ZammadState::Closed),
            _ => Err(format!("Invalid state: {}", state)),
        }
    }
}
/// Represents a Zammad user with essential contact information.
/// This is a simplified version of the full user object from Zammad,
/// containing only the fields we need for ticket synchronization.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ZammadUser {
    /// Unique identifier for the user
    pub id: u64,
    /// User's email address (also used as login)
    pub email: String,
    /// User's first name
    pub firstname: String,
    /// User's last name
    pub lastname: String,
}

/// Represents a Zammad article (comment/message) on a ticket.
/// Each article represents a communication in the ticket's history.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ZammadArticle {
    /// Unique identifier for the article
    pub id: Option<u64>,
    /// ID of the ticket this article belongs to
    pub ticket_id: Option<u64>,
    /// The actual content/message of the article
    pub body: Option<String>,
    /// MIME type of the content (e.g., "text/html", "text/plain")
    pub content_type: Option<String>,
    /// When the article was created
    pub created_at: Option<DateTime<Utc>>,
    /// When the article was last updated
    pub updated_at: Option<DateTime<Utc>>,
    /// Who sent the article (e.g., "Customer", "Agent")
    pub sender: Option<String>,
    /// Optional "From" field (e.g., "Dominik Münsterer")
    pub from: Option<String>,
    /// Optional "To" field (e.g., "Users")
    pub to: Option<String>,
}

async fn create_ticket(id: String, webhook: ZammadWebhook) -> anyhow::Result<()> {
    let db = DB::new().await?;

    db.create_assignment_from_zammad(&webhook.ticket.id).await?;

    let jira_issue_id = JiraCreateIssueRequest::from_zammad_webhook(&webhook)
        .submit()
        .await?
        .id;
    db.add_jira_id_to_assignment(&jira_issue_id, &webhook.ticket.id)
        .await?;
    Ok(())
}

#[tracing::instrument(skip(payload))]
async fn create_ticket_handler(
    Path(id): Path<String>,
    Json(payload): Json<ZammadWebhook>,
) -> StatusCode {
    match create_ticket(id, payload).await {
        Ok(_) => StatusCode::OK,
        Err(e) => {
            error!("Failed to create ticket: {}", e);
            StatusCode::BAD_REQUEST
        }
    }
}

#[tracing::instrument(skip(payload))]
async fn update_ticket_handler(Json(payload): Json<ZammadWebhook>) -> StatusCode {
    match update_ticket(payload).await {
        Ok(_) => StatusCode::OK,
        Err(e) => {
            error!("Failed to create ticket: {}", e);
            StatusCode::BAD_REQUEST
        }
    }
}

async fn update_ticket(payload: ZammadWebhook) -> anyhow::Result<()> {
    let db = DB::new().await?;
    let jira_issue_id = db.get_jira_id_by_zammad_id(&payload.ticket.id).await?;

    // We want to add a comment to the Jira issue if the article body is not empty
    if payload.article.body.is_some() {
        JiraAddCommentRequest::from_zammad_webhook(&payload)
            .submit(&jira_issue_id)
            .await?;
    }

    // We want to update the Jira issue with the new values from the Zammad ticket
    JiraUpdateIssueRequest::from_zammad_webhook(&payload)
        .submit(&jira_issue_id)
        .await?;

    Ok(())
}

pub fn router() -> Router {
    // Using specific Router<()> type to ensure we don't need state
    let router: Router<()> = Router::new()
        .route("/create-ticket/:id", post(create_ticket_handler))
        .route("/update-ticket/:id", post(update_ticket_handler));
    router
}
