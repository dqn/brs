pub mod payload;

use anyhow::Result;
use tracing::{error, info};

use self::payload::WebhookPayload;
use crate::screenshot::ScreenshotScoreInfo;

/// Webhook handler for sending score results to Discord webhooks.
pub struct WebhookHandler {
    client: reqwest::Client,
    urls: Vec<String>,
}

impl WebhookHandler {
    pub fn new(urls: Vec<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            urls,
        }
    }

    /// Send a webhook with score information and optional screenshot.
    pub async fn send_webhook(
        &self,
        score_info: &ScreenshotScoreInfo,
        song_title: &str,
        screenshot_data: Option<&[u8]>,
        webhook_name: &str,
        webhook_avatar: &str,
    ) -> Result<()> {
        let embed = payload::create_embed(score_info, song_title);
        let payload = WebhookPayload {
            username: if webhook_name.is_empty() {
                None
            } else {
                Some(webhook_name.to_string())
            },
            avatar_url: if webhook_avatar.is_empty() {
                None
            } else {
                Some(webhook_avatar.to_string())
            },
            embeds: vec![embed],
        };

        for url in &self.urls {
            if url.is_empty() {
                continue;
            }
            if let Err(e) = self.send_to_url(url, &payload, screenshot_data).await {
                error!("webhook send failed for {}: {}", url, e);
            } else {
                info!("webhook sent to {}", url);
            }
        }

        Ok(())
    }

    async fn send_to_url(
        &self,
        url: &str,
        payload: &WebhookPayload,
        screenshot_data: Option<&[u8]>,
    ) -> Result<()> {
        if let Some(image) = screenshot_data {
            // Multipart: JSON payload + image attachment
            let payload_json = serde_json::to_string(payload)?;
            let form = reqwest::multipart::Form::new()
                .text("payload_json", payload_json)
                .part(
                    "file",
                    reqwest::multipart::Part::bytes(image.to_vec())
                        .file_name("screenshot.png")
                        .mime_str("image/png")?,
                );
            self.client.post(url).multipart(form).send().await?;
        } else {
            self.client.post(url).json(payload).send().await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn webhook_handler_new() {
        let handler = WebhookHandler::new(vec![
            "https://example.com/webhook1".to_string(),
            "https://example.com/webhook2".to_string(),
        ]);
        assert_eq!(handler.urls.len(), 2);
    }

    #[test]
    fn webhook_handler_empty_urls() {
        let handler = WebhookHandler::new(vec![]);
        assert!(handler.urls.is_empty());
    }
}
