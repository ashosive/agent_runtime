use std::time::Duration;
use futures_util::StreamExt;
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
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
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
            .timeout(Duration::from_secs(120))
            .build()
            .unwrap();
        Self { base: base.into(), http }
    }

    pub async fn health(&self) -> Result<bool, OllamaError> {
        let resp = self.http.get(format!("{}/api/tags", self.base)).send().await?;
        Ok(resp.status().is_success())
    }

    pub async fn list_models(&self) -> Result<TagsResponse, OllamaError> {
        let resp = self.http.get(format!("{}/api/tags", self.base)).send().await?;
        let tags = resp.error_for_status()?.json::<TagsResponse>().await?;
        Ok(tags)
    }

    pub async fn pull_model(&self, name: &str) -> Result<(), OllamaError> {
        let body = PullRequest { name: name.to_string(), stream: true };
        let mut stream = self.http
            .post(format!("{}/api/pull", self.base))
            .json(&body)
            .send()
            .await?
            .bytes_stream();

        while let Some(chunk) = stream.next().await {
            let bytes = chunk?;
            let s = String::from_utf8_lossy(&bytes);
            if s.contains("\"status\":\"success\"") { break; }
        }
        Ok(())
    }

    pub async fn generate_once(&self, model: &str, prompt: &str) -> Result<String, OllamaError> {
        let req = GenerateRequest { model: model.to_string(), prompt: prompt.to_string(), stream: false };
        let resp = self.http
            .post(format!("{}/api/generate", self.base))
            .json(&req)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;
        let v: serde_json::Value = serde_json::from_str(&resp)?;
        let s = v.get("response").and_then(|x| x.as_str()).unwrap_or("").to_string();
        Ok(s)
    }

    pub async fn generate_stream(&self, model: &str, prompt: &str) -> Result<mpsc::Receiver<String>, OllamaError> {
        let req = json!({"model": model, "prompt": prompt, "stream": true});
        let resp = self.http
            .post(format!("{}/api/generate", self.base))
            .json(&req)
            .send()
            .await?
            .error_for_status()?;

        let mut stream = resp.bytes_stream();
        let (tx, rx) = mpsc::channel::<String>(64);

        tokio::spawn(async move {
            let tx = tx;
            let mut done = false;
            while let Some(chunk) = stream.next().await {
                match chunk {
                    Ok(bytes) => {
                        let text = String::from_utf8_lossy(&bytes);
                        for line in text.split('\n').filter(|l| !l.trim().is_empty()) {
                            if let Ok(evt) = serde_json::from_str::<GenerateStreamChunk>(line) {
                                if let Some(token) = evt.response {
                                    let _ = tx.send(token).await;
                                }
                                if evt.done.unwrap_or(false) {
                                    done = true;
                                    break;
                                }
                            }
                        }
                    }
                    Err(_) => break,
                }
            }
            if !done { let _ = tx.send("\n".to_string()).await; }
        });

        Ok(rx)
    }
}
