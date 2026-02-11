use anyhow::{Context, Result};
use serde::Deserialize;
use tracing::{info, warn};

#[derive(Clone)]
pub struct DiscordClient {
    bot_token: String,
    channel_id: String,
    openclaw_user_id: String,
    http: reqwest::Client,
}

#[derive(Debug, Deserialize)]
struct DiscordMessage {
    id: String,
    content: String,
    author: DiscordAuthor,
}

#[derive(Debug, Deserialize)]
struct DiscordAuthor {
    id: String,
}

impl DiscordClient {
    pub fn new(bot_token: &str, channel_id: &str, openclaw_user_id: &str) -> Self {
        Self {
            bot_token: bot_token.to_string(),
            channel_id: channel_id.to_string(),
            openclaw_user_id: openclaw_user_id.to_string(),
            http: reqwest::Client::new(),
        }
    }

    /// Send a message to the Discord channel and return its ID
    pub async fn send_message(&self, content: &str) -> Result<String> {
        let url = format!(
            "https://discord.com/api/v10/channels/{}/messages",
            self.channel_id
        );

        let body = serde_json::json!({ "content": content });

        let resp = self
            .http
            .post(&url)
            .header("Authorization", format!("Bot {}", self.bot_token))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .context("Failed to send Discord message")?;

        let status = resp.status();
        if !status.is_success() {
            let err_body = resp.text().await?;
            anyhow::bail!("Discord send failed ({}): {}", status, err_body);
        }

        let msg: DiscordMessage = resp.json().await?;
        info!(message_id = %msg.id, "Prompt sent to Discord");
        Ok(msg.id)
    }

    /// Poll for a response from OpenClaw after the given message ID
    /// Timeout: 60 seconds, polling interval: 2 seconds
    pub async fn poll_response(&self, after_message_id: &str) -> Result<Option<String>> {
        let max_attempts = 30; // 30 × 2s = 60s
        let poll_interval = tokio::time::Duration::from_secs(2);

        for attempt in 1..=max_attempts {
            tokio::time::sleep(poll_interval).await;

            let url = format!(
                "https://discord.com/api/v10/channels/{}/messages?after={}&limit=10",
                self.channel_id, after_message_id
            );

            let resp = self
                .http
                .get(&url)
                .header("Authorization", format!("Bot {}", self.bot_token))
                .send()
                .await;

            let resp = match resp {
                Ok(r) => r,
                Err(e) => {
                    warn!(attempt, error = %e, "Discord poll request failed");
                    continue;
                }
            };

            if !resp.status().is_success() {
                warn!(
                    attempt,
                    status = %resp.status(),
                    "Discord poll returned non-200"
                );
                continue;
            }

            let messages: Vec<DiscordMessage> = match resp.json().await {
                Ok(m) => m,
                Err(e) => {
                    warn!(attempt, error = %e, "Failed to parse Discord messages");
                    continue;
                }
            };

            // Find a response from OpenClaw
            for msg in &messages {
                if msg.author.id == self.openclaw_user_id {
                    info!(
                        attempt,
                        message_id = %msg.id,
                        content_len = msg.content.len(),
                        "OpenClaw response received"
                    );
                    return Ok(Some(msg.content.clone()));
                }
            }

            if attempt % 10 == 0 {
                info!(attempt, "Still waiting for OpenClaw response...");
            }
        }

        warn!("OpenClaw response timeout after 60 seconds");
        Ok(None)
    }

    /// Send prompt and wait for response (convenience wrapper)
    pub async fn ask(&self, prompt: &str) -> Result<Option<String>> {
        let msg_id = self.send_message(prompt).await?;
        self.poll_response(&msg_id).await
    }
}
