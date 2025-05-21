use crate::config;
use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use super::{
    jira_webhook::{JiraWebhook, JiraWebhookComment},
    zammad::{ZammadArticle, ZammadPriority, ZammadPriorityId, ZammadState, ZammadTicket},
};

#[derive(Debug, Serialize)]
pub struct ZammadUpdateTicketRequest {
    pub title: String,
    pub state: ZammadState,
    pub priority: ZammadPriority,
}

impl ZammadUpdateTicketRequest {
    pub fn from_jira_webhook(webhook: &JiraWebhook) -> Self {
        let mut state = ZammadState::Open;
        let mut priority = ZammadPriority {
            id: ZammadPriorityId::Normal,
        };

        // Update state and priority based on changelog
        if let Some(changelog) = &webhook.jira_webhook_changelog {
            for item in &changelog.items {
                match item.field.as_str() {
                    "status" => {
                        state = match item.toString.to_lowercase().as_str() {
                            "done" | "closed" | "resolved" => ZammadState::Closed,
                            _ => ZammadState::Open,
                        };
                    }
                    "priority" => {
                        priority.id = match item.toString.to_lowercase().as_str() {
                            "highest" | "blocker" => ZammadPriorityId::High,
                            "high" => ZammadPriorityId::High,
                            "medium" => ZammadPriorityId::Normal,
                            "low" | "lowest" => ZammadPriorityId::Low,
                            _ => ZammadPriorityId::Normal,
                        };
                    }
                    _ => {}
                }
            }
        }

        Self {
            title: webhook
                .jira_webhook_issue
                .as_ref()
                .map(|issue| issue.fields.summary.clone())
                .unwrap_or_default(),
            state,
            priority,
        }
    }

    pub async fn submit(&self, zammad_id: &i32) -> anyhow::Result<()> {
        let client = Client::new();
        let url = get_zammad_url();
        let url = format!("{}/tickets/{}", &url, zammad_id);

        info!("Zammad Request URL: {}", url);
        info!("Zammad Request: {:?}", self);

        let resp = client
            .put(&url)
            .json(&self)
            .header(
                "Authorization",
                format!("Token token={}", get_zammad_token()),
            )
            .send()
            .await?
            .error_for_status()
            .map_err(|e| anyhow::anyhow!("Error status from Zammad API: {}", e))?
            .text()
            .await
            .context("Failed to get response body")?;

        info!("Zammad Response: {:?}", resp);

        Ok(())
    }
}

#[derive(Debug, Serialize)]
pub struct ZammadAddCommentRequest {
    pub body: String,
    pub content_type: String,
    pub sender: String,
}

impl ZammadAddCommentRequest {
    pub fn from_jira_comment(comment: &JiraWebhookComment) -> Self {
        Self {
            body: comment.body.clone(),
            content_type: "text/plain".to_string(),
            sender: comment.author.displayName.clone(),
        }
    }

    pub async fn submit(&self, zammad_id: &i32) -> anyhow::Result<()> {
        let client = Client::new();
        let url = get_zammad_url();
        let url = format!("{}/tickets/{}/articles", &url, zammad_id);

        info!("Zammad Request URL: {}", url);
        info!("Zammad Request: {:?}", self);

        let resp = client
            .post(&url)
            .json(&self)
            .header(
                "Authorization",
                format!("Token token={}", get_zammad_token()),
            )
            .send()
            .await?
            .error_for_status()
            .map_err(|e| anyhow::anyhow!("Error status from Zammad API: {}", e))?
            .text()
            .await
            .context("Failed to get response body")?;

        info!("Zammad Response: {:?}", resp);

        Ok(())
    }
}

fn get_zammad_url() -> String {
    config::get_zammad().endpoint.clone()
}

fn get_zammad_token() -> String {
    config::get_zammad().token.clone()
}
