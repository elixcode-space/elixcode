//! HTTP API client for the Elixcode gateway.
//!
//! Wraps the OpenAI-compatible /v1/chat/completions endpoint plus
//! Elixcode-specific endpoints (/sessions, /share, /models, /providers).

use crate::auth::AuthState;
use crate::config::Config;
use anyhow::{Context, Result};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::pin::Pin;

pub struct ApiClient {
    pub cfg: Config,
    pub auth: AuthState,
    pub http: reqwest::Client,
}

impl ApiClient {
    pub fn new(cfg: Config) -> Result<Self> {
        let auth = AuthState::load().unwrap_or_else(|_| AuthState {
            version: "1".to_string(),
            api_key: None,
            jwt: None,
            refresh_token: None,
            user: None,
            expires_at: None,
        });

        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(300))
            .build()?;

        Ok(Self { cfg, auth, http })
    }

    /// Build auth headers
    pub fn auth_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        if let Some(key) = self.cfg.effective_api_key() {
            if let Ok(val) = HeaderValue::from_str(&format!("Bearer {}", key)) {
                headers.insert(HeaderName::from_static("authorization"), val);
            }
        }
        headers
    }

    /// POST JSON and parse response
    pub async fn post_json<T: for<'de> Deserialize<'de>>(
        &self,
        path: &str,
        body: Value,
    ) -> Result<T> {
        let url = format!("{}{}", self.cfg.api_base(), path);
        let resp = self
            .http
            .post(&url)
            .headers(self.auth_headers())
            .header(CONTENT_TYPE, "application/json")
            .json(&body)
            .send()
            .await
            .with_context(|| format!("POST {} failed", url))?;

        if resp.status().is_success() {
            let data = resp.json().await?;
            Ok(data)
        } else {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("API error {}: {}", status, body);
        }
    }

    /// GET JSON
    pub async fn get_json<T: for<'de> Deserialize<'de>>(
        &self,
        path: &str,
    ) -> Result<T> {
        let url = format!("{}{}", self.cfg.api_base(), path);
        let resp = self
            .http
            .get(&url)
            .headers(self.auth_headers())
            .send()
            .await
            .with_context(|| format!("GET {} failed", url))?;

        if resp.status().is_success() {
            let data = resp.json().await?;
            Ok(data)
        } else {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("API error {}: {}", status, body);
        }
    }

    /// DELETE
    pub async fn delete(&self, path: &str) -> Result<()> {
        let url = format!("{}{}", self.cfg.api_base(), path);
        let resp = self
            .http
            .delete(&url)
            .headers(self.auth_headers())
            .send()
            .await
            .with_context(|| format!("DELETE {} failed", url))?;

        if resp.status().is_success() || resp.status().as_u16() == 404 {
            Ok(())
        } else {
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("DELETE failed: {}", body);
        }
    }

    /// OpenAI-compatible /v1/chat/completions
    pub async fn chat_completions(
        &self,
        model: &str,
        messages: &[ChatMessage],
        stream: bool,
        temperature: Option<f64>,
        max_tokens: Option<u32>,
    ) -> Result<ChatCompletionsResponse> {
        #[derive(Serialize)]
        struct Body<'a> {
            model: &'a str,
            messages: &'a [ChatMessage],
            stream: bool,
            #[serde(skip_serializing_if = "Option::is_none")]
            temperature: Option<f64>,
            #[serde(skip_serializing_if = "Option::is_none")]
            max_tokens: Option<u32>,
        }

        let body = Body {
            model,
            messages,
            stream,
            temperature,
            max_tokens,
        };

        self.post_json("/v1/chat/completions", serde_json::to_value(body)?).await
    }

    /// SSE stream — yields parsed chunks
    pub async fn chat_stream(
        &self,
        model: &str,
        messages: &[ChatMessage],
        temperature: Option<f64>,
        max_tokens: Option<u32>,
    ) -> Result<impl futures_util::Stream<Item = Result<StreamChunk>>> {
        #[derive(Serialize)]
        struct Body<'a> {
            model: &'a str,
            messages: &'a [ChatMessage],
            stream: bool,
            #[serde(skip_serializing_if = "Option::is_none")]
            temperature: Option<f64>,
            #[serde(skip_serializing_if = "Option::is_none")]
            max_tokens: Option<u32>,
        }

        let body = Body {
            model,
            messages,
            stream: true,
            temperature,
            max_tokens,
        };

        let url = format!("{}/v1/chat/completions", self.cfg.api_base());
        let resp = self
            .http
            .post(&url)
            .headers(self.auth_headers())
            .header(CONTENT_TYPE, "application/json")
            .json(&body)
            .send()
            .await
            .with_context(|| format!("POST {} failed", url))?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("stream request failed: {}", body);
        }

        let stream = SseStream::new(resp.bytes_stream());
        Ok(stream)
    }

    /// List sessions
    pub async fn list_sessions(&self) -> Result<Vec<SessionSummary>> {
        self.get_json("/sessions").await
    }

    /// Get a session
    pub async fn get_session(&self, id: &str) -> Result<Session> {
        self.get_json(&format!("/sessions/{}", id)).await
    }

    /// Delete a session
    pub async fn delete_session(&self, id: &str) -> Result<()> {
        self.delete(&format!("/sessions/{}", id)).await
    }

    /// Share a session
    pub async fn share_session(&self, id: &str) -> Result<String> {
        #[derive(Serialize)]
        struct Body { session_id: String }
        #[derive(Deserialize)]
        struct Resp { url: String }

        let resp: Resp = self
            .post_json("/share", serde_json::to_value(Body { session_id: id.to_string() })?)
            .await?;
        Ok(resp.url)
    }

    /// List models
    pub async fn list_models(&self) -> Result<Vec<ModelInfo>> {
        self.get_json("/v1/models").await
    }

    /// List providers
    pub async fn list_providers(&self) -> Result<Vec<ProviderInfo>> {
        self.get_json("/providers").await
    }

    /// Health check
    pub async fn health(&self) -> Result<bool> {
        match self
            .http
            .get(format!("{}/health", self.cfg.api_base()))
            .headers(self.auth_headers())
            .send()
            .await
        {
            Ok(resp) => Ok(resp.status().is_success()),
            Err(_) => Ok(false),
        }
    }
}

// ─── Types ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

impl ChatMessage {
    pub fn user(content: impl Into<String>) -> Self {
        Self { role: "user".to_string(), content: content.into(), name: None, tool_calls: None, tool_call_id: None }
    }
    pub fn assistant(content: impl Into<String>) -> Self {
        Self { role: "assistant".to_string(), content: content.into(), name: None, tool_calls: None, tool_call_id: None }
    }
    pub fn system(content: impl Into<String>) -> Self {
        Self { role: "system".to_string(), content: content.into(), name: None, tool_calls: None, tool_call_id: None }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub function: ToolFunction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolFunction {
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionsResponse {
    pub id: Option<String>,
    pub object: Option<String>,
    pub created: Option<u64>,
    pub model: String,
    pub choices: Vec<Choice>,
    pub usage: Option<Usage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Choice {
    pub index: u32,
    pub message: Option<ChatMessage>,
    pub delta: Option<ChatMessage>,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: Option<u32>,
    pub completion_tokens: Option<u32>,
    pub total_tokens: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamChunk {
    pub id: Option<String>,
    pub delta: String,
    pub finish_reason: Option<String>,
    pub usage: Option<Usage>,
}

struct SseStream<S> {
    inner: Pin<Box<S>>,
    buffer: Vec<u8>,
}

impl<S: futures_util::Stream<Item = std::result::Result<bytes::Bytes, reqwest::Error>> + Unpin> SseStream<S> {
    fn new(stream: S) -> Self {
        Self { inner: Box::pin(stream), buffer: Vec::new() }
    }
}

impl<S: futures_util::Stream<Item = std::result::Result<bytes::Bytes, reqwest::Error>> + Unpin> futures_util::Stream for SseStream<S> {
    type Item = Result<StreamChunk>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Option<Self::Item>> {
        use std::task::Poll::*;
        loop {
            // Drain any complete SSE events from buffer
            if let Some(idx) = self.buffer.windows(2).position(|w| w[0] == b'\n' && w[1] == b'\n') {
                let raw = String::from_utf8_lossy(&self.buffer[..idx]).to_string();
                self.buffer.drain(..idx + 2);

                for line in raw.lines() {
                    if let Some(data) = line.strip_prefix("data: ") {
                        let data = data.trim();
                        if data == "[DONE]" {
                            return Ready(None);
                        }
                        if let Ok(chunk) = serde_json::from_str::<StreamChunk>(data) {
                            return Ready(Some(Ok(chunk)));
                        }
                    }
                }
                // Continue draining
                continue;
            }

            // Buffer empty — try to read more
            return match self.inner.as_mut().poll_next(cx) {
                Ready(Some(Ok(bytes))) => {
                    self.buffer.extend_from_slice(&bytes);
                    cx.waker().wake_by_ref();
                    Pending
                }
                Ready(Some(Err(e))) => Ready(Some(Err(anyhow::anyhow!("stream error: {}", e)))),
                Ready(None) => Ready(None),
                Pending => Pending,
            };
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub name: Option<String>,
    pub messages: Vec<ChatMessage>,
    pub model: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    pub id: String,
    pub name: Option<String>,
    pub model: String,
    pub message_count: u32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub object: Option<String>,
    pub created: Option<u64>,
    pub owned_by: Option<String>,
    pub permission: Option<Vec<Value>>,
    pub root: Option<String>,
    pub parent: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderInfo {
    pub name: String,
    pub enabled: bool,
    pub default_model: Option<String>,
    pub api_key_configured: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerInfo {
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub hostname: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub target: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub host: Option<String>,
    #[serde(default)]
    pub port: Option<u16>,
    #[serde(default)]
    pub gpu_count: Option<u32>,
    #[serde(default)]
    pub vram_total_mb: Option<u64>,
    #[serde(default)]
    pub last_heartbeat: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareInfo {
    #[serde(default)]
    pub gpus: Vec<serde_json::Value>,
    #[serde(default)]
    pub backends: Vec<serde_json::Value>,
    #[serde(default)]
    pub recommended: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentResponse {
    pub deployment_id: String,
    pub status: String,
    pub started_at: String,
}
