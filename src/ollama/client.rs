use std::time::Duration;

use reqwest::Client;
use serde_json::json;
use thiserror::Error;
use tokio::sync::mpsc;

use crate::ollama::types::*;

#[derive(Debug, Error)]
pub enum OllamaError {
    #[error("http: {0}")]
    Http(#[from] reqwest::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("server: {0}")]
    Server(String),
}

#[derive(Clone)]
pub struct OllamaClient {
    base: String,
    http: Client,
}

impl OllamaClient {
    pub fn new(base: impl Into<String>) -> Self {
        let http = Client::builder()
            .timeout(Duration::from_secs(600))
            .build()
            .unwrap();
        Self { base: base.into(), http }
    }

    pub async fn health(&self) -> Result<bool, OllamaError> {
        let resp = self.http.get(format!("{}/api/tags", self.base)).send().await?;
        Ok(resp.status().is_success())
    }

    pub async fn list_models(&self) -> Result<TagsResponse, OllamaError> {
        let resp = self
            .http
            .get(format!("{}/api/tags", self.base))
            .send()
            .await?;
        let tags = resp.error_for_status()?.json::<TagsResponse>().await?;
        Ok(tags)
    }

    pub async fn pull_model(&self, name: &str) -> Result<(), OllamaError> {
        let body = PullRequest {
            name: name.to_string(),
            stream: true,
        };
        let resp = self
            .http
            .post(format!("{}/api/pull", self.base))
            .json(&body)
            .send()
            .await?;
        if !resp.status().is_success() {
            return Err(OllamaError::Server(format!(
                "pull failed: {}",
                resp.status()
            )));
        }
        Ok(())
    }

    pub async fn generate_once(&self, model: &str, prompt: &str) -> Result<String, OllamaError> {
        let body = json!({
            "model": model,
            "prompt": prompt,
            "stream": false
        });

        let resp = self
            .http
            .post(format!("{}/api/generate", self.base))
            .json(&body)
            .send()
            .await?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(OllamaError::Server(format!(
                "status {} body {}",
                status, text
            )));
        }

        let v: serde_json::Value = resp.json().await?;
        let s = v
            .get("response")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .to_string();
        Ok(s)
    }

    pub async fn generate_stream(
        &self,
        model: &str,
        prompt: &str,
    ) -> Result<mpsc::Receiver<String>, OllamaError> {
        let text = self.generate_once(model, prompt).await?;
        let (tx, rx) = mpsc::channel::<String>(4);

        tokio::spawn(async move {
            let _ = tx.send(text).await;
        });

        Ok(rx)
    }
}
