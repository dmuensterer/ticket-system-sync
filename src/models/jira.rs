use axum::{async_trait, http::Error};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::info;

use super::ticketsystem::TicketSystem;

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

pub struct JiraSystem;

#[async_trait]
impl TicketSystem for JiraSystem {
    fn name(&self) -> &'static str {
        "Jira"
    }

    async fn process_webhook(&self, payload: Value) -> Result<(), String> {
        info!("Jira Webhook not yet implemented");
        // TODO: Implement Jira-specific processing
        Ok(())
    }
}
