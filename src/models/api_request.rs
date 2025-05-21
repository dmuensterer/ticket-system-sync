use super::{
    jira::{JiraFields, JiraIssueType, JiraPriority, JiraPriorityEnum, JiraProject, JiraStatus},
    zammad::{ZammadPriorityId, ZammadState, ZammadWebhook},
};
use crate::config;
use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Deserializer, Serialize};
use std::str::FromStr;
use tracing::{debug, info};

#[derive(Debug, Serialize)]
pub struct JiraCreateIssueRequest {
    pub fields: JiraFields,
}

impl JiraCreateIssueRequest {
    pub fn from_zammad_webhook(webhook: &ZammadWebhook) -> Self {
        debug!("Ticket: {:?}", &webhook);
        Self {
            fields: JiraFields {
                project: JiraProject {
                    id: get_jira_project(),
                },
                summary: webhook.ticket.title.clone(),
                description: webhook.article.body.clone().unwrap_or_default(),
                priority: JiraPriority {
                    name: convert_zammad_priority_to_jira_priority(webhook.ticket.priority.id),
                },
                //                status: convert_zammad_state_to_jira_status(ticket.state),
                issuetype: JiraIssueType {
                    name: "Task".to_string(),
                },
                duedate: webhook.ticket.due_date.format("%Y-%m-%d").to_string(),
                // Jira doesn't allow to create an issue with a status.
                //                status: JiraStatus::from_zammad_state(webhook.ticket.state),
            },
        }
    }
    pub async fn submit(&self) -> anyhow::Result<JiraCreateIssueResponse> {
        debug!("Trying to make request to Jira");

        let client = Client::new();
        let url = get_jira_url();

        info!("Jira Request URL: {}", url);

        info!("Jira Request: {:?}", serde_json::to_string(self)?);

        let resp = client
            .post(&url)
            .json(&self)
            .basic_auth(get_jira_credentials().0, Some(get_jira_credentials().1))
            .send()
            .await
            .context("failed to send request to Jira API")?
            .error_for_status() // 4xx/5xx → error
            .context("error status from Jira API")?
            .json::<JiraCreateIssueResponse>()
            .await?;

        info!("Jira Response: {:?}", resp);
        // ---- read the body once -------------------------------------------------

        Ok(resp)
    }
}

#[derive(Debug, Serialize)]
pub struct JiraUpdateIssueRequest {
    fields: JiraUpdateIssueProperties,
}

impl JiraUpdateIssueRequest {
    pub fn from_zammad_webhook(webhook: &ZammadWebhook) -> Self {
        Self {
            fields: JiraUpdateIssueProperties {
                priority: JiraPriority {
                    name: convert_zammad_priority_to_jira_priority(webhook.ticket.priority.id),
                },
            },
        }
    }

    pub async fn submit(&self, jira_issue_id: &i32) -> anyhow::Result<()> {
        let client = Client::new();
        let url = get_jira_url();

        let url = format!("{}/{}", &url, jira_issue_id);

        info!("Jira Request URL: {}", url);
        info!("Jira Request: {:?}", self);

        info!(
            "Updating Jira issue to the following properties: {:?}",
            &self
        );

        let resp = client
            .put(&url)
            .json(&self)
            .basic_auth(get_jira_credentials().0, Some(get_jira_credentials().1))
            .send()
            .await?
            .error_for_status()
            .map_err(|e| anyhow::anyhow!("Error status from Jira API: {}", e))?
            .text()
            .await
            .context("Failed to get response body")?;

        info!("Jira Response: {:?}", resp);

        Ok(())
    }
}

#[derive(Debug, Serialize)]
pub struct JiraUpdateIssueProperties {
    priority: JiraPriority,
}

#[derive(Debug, Serialize)]
pub struct JiraAddCommentRequest {
    body: String,
}

impl JiraAddCommentRequest {
    pub fn from_zammad_webhook(webhook: &ZammadWebhook) -> Self {
        debug!("Article: {:?}", &webhook.article);
        Self {
            body: webhook.article.body.clone().unwrap_or_default(),
        }
    }

    pub async fn submit(&self, jira_issue_id: &i32) -> anyhow::Result<JiraAddCommentResponse> {
        debug!("Trying to make request to Jira");

        let client = Client::new();
        let url = get_jira_url();

        let url = format!("{}/{}/{}", &url, jira_issue_id, "comment");
        info!("Jira Request URL: {}", url);
        info!("Jira Request: {:?}", self);

        let resp = client
            .post(&url)
            .json(&self)
            .basic_auth(get_jira_credentials().0, Some(get_jira_credentials().1))
            .send()
            .await?
            .error_for_status()
            .map_err(|e| anyhow::anyhow!("Error status from Jira API: {}", e))?
            .text()
            .await
            .context("Failed to get response body")?;

        let resp: JiraAddCommentResponse = {
            let mut deserializer = serde_json::Deserializer::from_str(&resp);
            serde_path_to_error::deserialize(&mut deserializer)
                .map_err(|e| anyhow::anyhow!("Failed to parse Jira response: {}", e))?
        };

        info!("Jira Response: {:?}", resp);

        Ok(resp)
    }
}

fn string_to_number<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: FromStr,
    T::Err: std::fmt::Display,
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    T::from_str(&s).map_err(serde::de::Error::custom)
}

#[derive(Debug, Deserialize, Serialize)]
pub struct JiraAddCommentResponse {
    #[serde(deserialize_with = "string_to_number")]
    pub id: i32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct JiraCreateIssueResponse {
    #[serde(deserialize_with = "string_to_number")]
    pub id: i32,
    pub key: String,
}

fn convert_zammad_priority_to_jira_priority(priority: ZammadPriorityId) -> JiraPriorityEnum {
    match priority {
        ZammadPriorityId::Low => JiraPriorityEnum::Lowest,
        ZammadPriorityId::Normal => JiraPriorityEnum::Medium,
        ZammadPriorityId::High => JiraPriorityEnum::High,
    }
}

fn convert_zammad_state_to_jira_status(state: ZammadState) -> JiraStatus {
    match state {
        ZammadState::Open => JiraStatus::Open,
        ZammadState::Closed => JiraStatus::Closed,
    }
}

fn get_jira_url() -> String {
    config::get_jira().endpoint.clone()
}

fn get_jira_credentials() -> (String, String) {
    let jira_config = config::get_jira();
    (jira_config.username.clone(), jira_config.token.clone())
}

fn get_jira_project() -> i32 {
    config::get_jira().project_id
}
