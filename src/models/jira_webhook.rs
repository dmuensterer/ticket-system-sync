use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JiraWebhook {
    pub jira_webhook_comment: Option<JiraWebhookComment>,
    pub jira_webhook_issue: Option<JiraWebhookIssue>,
    pub jira_webhook_changelog: Option<JiraWebhookChangelog>,
    pub jira_webhook_user: Option<JiraWebhookUser>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JiraWebhookComment {
    pub id: i32,
    pub body: String,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub author: JiraWebhookUser,
    pub issue: JiraWebhookIssue,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JiraWebhookIssue {
    pub id: i32,
    pub key: String,
    pub fields: JiraWebhookIssueFields,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JiraWebhookIssueFields {
    pub summary: String,
    pub project: JiraWebhookProject,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JiraWebhookProject {
    pub id: i32,
    pub key: String,
    pub name: String,
    pub projectTypeKey: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JiraWebhookChangelog {
    pub id: i32,
    pub items: Vec<JiraWebhookChangelogItem>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JiraWebhookChangelogItem {
    pub field: String,
    pub fieldtype: String,
    pub fieldId: String,
    pub from: String,
    pub fromString: String,
    pub to: String,
    pub toString: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JiraWebhookUser {
    pub accountId: String,
    pub displayName: String,
    pub active: bool,
    pub timeZone: String,
    pub accountType: String,
}
