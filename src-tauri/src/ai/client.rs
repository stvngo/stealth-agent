use super::context::Message;
use futures::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

#[derive(Debug, Clone, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    stream: bool,
    max_tokens: u32,
    temperature: f32,
}

#[derive(Debug, Deserialize)]
struct StreamChunk {
    choices: Vec<StreamChoice>,
}

#[derive(Debug, Deserialize)]
struct StreamChoice {
    delta: Delta,
}

#[derive(Debug, Deserialize)]
struct Delta {
    content: Option<String>,
}

pub struct AiClient {
    client: Client,
    api_key: String,
    model: String,
    base_url: String,
}

impl AiClient {
    pub fn new(api_key: String, model: Option<String>, base_url: Option<String>) -> Self {
        Self {
            client: Client::new(),
            api_key,
            model: model.unwrap_or_else(|| "gpt-4o".to_string()),
            base_url: base_url.unwrap_or_else(|| "https://api.openai.com/v1".to_string()),
        }
    }

    /// Send a chat completion request and stream tokens back via the channel.
    pub async fn chat_stream(
        &self,
        messages: Vec<Message>,
        tx: mpsc::UnboundedSender<String>,
    ) -> anyhow::Result<()> {
        let request = ChatRequest {
            model: self.model.clone(),
            messages,
            stream: true,
            max_tokens: 4096,
            temperature: 0.3,
        };

        let response = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("API error {}: {}", status, body);
        }

        let mut stream = response.bytes_stream();
        let mut buffer = String::new();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            buffer.push_str(&String::from_utf8_lossy(&chunk));

            while let Some(line_end) = buffer.find('\n') {
                let line = buffer[..line_end].trim().to_string();
                buffer = buffer[line_end + 1..].to_string();

                if line.is_empty() || line == "data: [DONE]" {
                    continue;
                }

                if let Some(json_str) = line.strip_prefix("data: ") {
                    if let Ok(chunk) = serde_json::from_str::<StreamChunk>(json_str) {
                        for choice in &chunk.choices {
                            if let Some(ref content) = choice.delta.content {
                                let _ = tx.send(content.clone());
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Non-streaming chat completion for simpler use cases.
    pub async fn chat(&self, messages: Vec<Message>) -> anyhow::Result<String> {
        let (tx, mut rx) = mpsc::unbounded_channel();
        self.chat_stream(messages, tx).await?;

        let mut result = String::new();
        while let Some(token) = rx.recv().await {
            result.push_str(&token);
        }
        Ok(result)
    }
}
