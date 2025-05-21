use serde::Serialize;
use tracing::info;
use tracing_subscriber::field::debug;
use uuid::Uuid;

use super::db::DB;
use super::jira::JiraIssue;
use super::zammad::ZammadTicket;

#[derive(Serialize, Debug, Clone)]
pub struct Assignment {
    pub id: String,
    pub zammad_ticket: Option<ZammadTicket>,
    pub jira_issue: Option<JiraIssue>,
}

impl Assignment {
    pub async fn new(
        zammad_ticket: Option<ZammadTicket>,
        jira_issue: Option<JiraIssue>,
    ) -> Assignment {
        let assignment = Assignment {
            id: Uuid::new_v4().to_string(),
            zammad_ticket,
            jira_issue,
        };
        assignment
    }

    pub fn add_to_zammad(
        &self,
        zammad_ticket: &ZammadTicket,
        jira_issue: &JiraIssue,
    ) -> anyhow::Result<Assignment> {
        let assignment = Assignment {
            id: self.id.clone(),
            zammad_ticket: Some(zammad_ticket.clone()),
            jira_issue: Some(jira_issue.clone()),
        };

        info!(
            "Updated ticket assignment: Added jira_issue_id: {:?} to zammad_ticket_id: {:?}",
            jira_issue.project.id, zammad_ticket.id
        );
        Ok(assignment)
    }
}
