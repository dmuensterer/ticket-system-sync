use async_trait::async_trait;
use serde_json::Value;

use super::{jira::JiraSystem, zammad::ZammadSystem};

/// Trait defining the behavior of a ticket system
#[async_trait]
pub trait TicketSystem: Send + Sync {
    /// Get the name of the ticket system
    fn name(&self) -> &'static str;

    /// Process a webhook payload
    async fn add_comment(&self, payload: Value) -> Result<(), String>;
    async fn create_ticket(&self, payload: Value) -> Result<(), String>;
}

/// Factory function to create the appropriate ticket system
fn create_ticket_system(system_type: &str) -> Option<Box<dyn TicketSystem>> {
    match system_type {
        "zammad" => Some(Box::new(ZammadSystem)),
        "jira" => Some(Box::new(JiraSystem)),
        _ => None,
    }
}
