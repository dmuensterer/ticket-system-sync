use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{error, info};

use super::ticketsystem::TicketSystem;

/// Represents a Zammad webhook payload containing both ticket and article information.
/// This is the top-level structure that Zammad sends when a ticket is created or updated.
#[derive(Debug, Serialize, Deserialize)]
pub struct ZammadWebhook {
    /// The ticket information including metadata, state, and relationships
    pub ticket: ZammadTicket,
    /// The article (comment/message) associated with this ticket update
    pub article: ZammadArticle,
}

/// Represents a Zammad ticket with all its metadata and relationships.
/// This structure contains all the essential information needed to create/update
/// a corresponding ticket in another system (like Jira).
#[derive(Debug, Serialize, Deserialize)]
pub struct ZammadTicket {
    /// Unique identifier for the ticket in Zammad
    pub id: u64,
    /// Human-readable ticket number (e.g., "45003")
    pub number: String,
    /// The ticket's title/subject (e.g., "Test Ticket")
    pub title: String,
    /// Current state of the ticket (e.g., "open", "closed")
    pub state: String,
    /// Priority information including name and ID
    pub priority: ZammadPriority,
    /// When the ticket was created
    pub created_at: DateTime<Utc>,
    /// When the ticket was last updated
    pub updated_at: DateTime<Utc>,
    /// Optional due date for the ticket
    pub due_date: Option<DateTime<Utc>>,
    /// User who created the ticket
    pub created_by: ZammadUser,
    /// User who is currently assigned to the ticket
    pub owner: ZammadUser,
}

/// Represents a Zammad priority level.
/// Example: "2 normal" with ID 2
#[derive(Debug, Serialize, Deserialize)]
pub struct ZammadPriority {
    /// Unique identifier for the priority
    pub id: u64,
    /// Human-readable priority name (e.g., "2 normal", "3 high")
    pub name: String,
}

/// Represents a Zammad user with essential contact information.
/// This is a simplified version of the full user object from Zammad,
/// containing only the fields we need for ticket synchronization.
#[derive(Debug, Serialize, Deserialize)]
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
#[derive(Debug, Serialize, Deserialize)]
pub struct ZammadArticle {
    /// Unique identifier for the article
    pub id: u64,
    /// ID of the ticket this article belongs to
    pub ticket_id: u64,
    /// The actual content/message of the article
    pub body: String,
    /// MIME type of the content (e.g., "text/html", "text/plain")
    pub content_type: String,
    /// When the article was created
    pub created_at: DateTime<Utc>,
    /// When the article was last updated
    pub updated_at: DateTime<Utc>,
    /// Who sent the article (e.g., "Customer", "Agent")
    pub sender: String,
    /// Optional "From" field (e.g., "Dominik Münsterer")
    pub from: Option<String>,
    /// Optional "To" field (e.g., "Users")
    pub to: Option<String>,
    /// Whether this is an internal note (not visible to customers)
    pub internal: bool,
}

pub struct ZammadSystem;

#[async_trait]
impl TicketSystem for ZammadSystem {
    fn name(&self) -> &'static str {
        "Zammad"
    }

    #[tracing::instrument(skip_all, fields(ticket_id))]
    async fn add_comment(&self, payload: Value) -> Result<(), String> {
        let payload = parse_zammad_webhook(payload)?;

        // Add ticket_id to the tracing span
        tracing::Span::current().record("ticket_id", &payload.ticket.id);

        info!("Adding comment to Zammad ticket");
        Ok(())
    }

    #[tracing::instrument(skip_all, fields(ticket_id))]
    async fn create_ticket(&self, payload: Value) -> Result<(), String> {
        let payload = parse_zammad_webhook(payload)?;

        // Add ticket_id to the tracing span
        tracing::Span::current().record("ticket_id", &payload.ticket.id);

        info!("Creating Zammad ticket");
        Ok(())
    }
}

fn parse_zammad_webhook(payload: Value) -> Result<ZammadWebhook, String> {
    serde_json::from_value::<ZammadWebhook>(payload).map_err(|e| {
        error!("Failed to parse Zammad webhook: {}", e);
        e.to_string()
    })
}
