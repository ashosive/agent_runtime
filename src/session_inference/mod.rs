mod prompt_builder;

use std::time::Duration;

use chrono::Utc;
use once_cell::sync::Lazy;
use tokio::sync::mpsc;
use tokio::time::sleep;
use uuid::Uuid;

use crate::ollama::client::{OllamaClient, OllamaError};
use crate::session_creation::types::SessionState;
use crate::session_manager::SESSION_STORE;
use prompt_builder::{build_prompt, apply_user_input};

pub trait InferenceEngine {
    fn stream(&self, model: &str, prompt: &str) -> mpsc::Receiver<String>;
}

static OLLAMA: Lazy<OllamaClient> = Lazy::new(|| OllamaClient::new("http://127.0.0.1:11434"));

#[derive(thiserror::Error, Debug)]
pub enum InferenceError {
    #[error("session not found")]
    NotFound,
    #[error("invalid state: {0:?}")]
    InvalidState(SessionState),
    #[error("no model configured for this session")]
    NoModel,
    #[error("ollama: {0}")]
    Ollama(#[from] OllamaError),
    #[error("poisoned lock")]
    Poisoned,
}

pub async fn infer_once(id: &Uuid, prompt: &str) -> Result<String, InferenceError> {
    let (model, _limits) = {
        let Some(entry) = SESSION_STORE.get(id) else {
            return Err(InferenceError::NotFound);
        };
        let s = entry.read().map_err(|_| InferenceError::Poisoned)?;
        if s.state != SessionState::Active {
            return Err(InferenceError::InvalidState(s.state.clone()));
        }
        let model = s.model.clone().ok_or(InferenceError::NoModel)?;
        (model, s.limits.clone())
    };
    let out = OLLAMA.generate_once(&model, prompt).await?;
    Ok(out)
}

pub async fn infer_stream(
    id: &Uuid,
    prompt: &str,
) -> Result<mpsc::Receiver<String>, InferenceError> {
    let (model, _limits) = {
        let Some(entry) = SESSION_STORE.get(id) else {
            return Err(InferenceError::NotFound);
        };
        let s = entry.read().map_err(|_| InferenceError::Poisoned)?;
        if s.state != SessionState::Active {
            return Err(InferenceError::InvalidState(s.state.clone()));
        }
        let model = s.model.clone().ok_or(InferenceError::NoModel)?;
        (model, s.limits.clone())
    };
    let rx = OLLAMA.generate_stream(&model, prompt).await?;
    Ok(rx)
}

pub async fn infer_once_with_input(id: &Uuid, input: &str) -> Result<String, InferenceError> {
    let prompt = apply_user_input(id, input).ok_or(InferenceError::NotFound)?;
    infer_once(id, &prompt).await
}


pub async fn infer_stream_with_input(
    id: &Uuid,
    input: &str,
) -> Result<mpsc::Receiver<String>, InferenceError> {
    let prompt = apply_user_input(id, input).ok_or(InferenceError::NotFound)?;
    infer_stream(id, &prompt).await
}

pub async fn run_session_inference(session_id: Uuid) {
    let prompt = match build_prompt(&session_id) {
        Some(p) => p,
        None => return,
    };

    let mut maybe_rx = infer_stream(&session_id, &prompt).await.ok();

    if let Some(ref mut rx) = maybe_rx {
        while let Some(tok) = rx.recv().await {
            println!("session {} token: {}", session_id, tok);
            if let Some(entry) = SESSION_STORE.get(&session_id) {
                if let Ok(mut s) = entry.write() {
                    s.updated_at = Utc::now();
                }
            }
        }
    } else {
        for i in 1..=5 {
            println!("session {} fake token {}", session_id, i);
            if let Some(entry) = SESSION_STORE.get(&session_id) {
                if let Ok(mut s) = entry.write() {
                    s.updated_at = Utc::now();
                }
            }
            sleep(Duration::from_millis(200)).await;
        }
    }

    if let Some(entry) = SESSION_STORE.get(&session_id) {
        if let Ok(mut s) = entry.write() {
            s.updated_at = Utc::now();
        }
    }

    const INACTIVITY_TIMEOUT_SECS: i64 = 120;
    const INACTIVITY_POLL_INTERVAL_SECS: u64 = 100;

    loop {
        sleep(Duration::from_secs(INACTIVITY_POLL_INTERVAL_SECS)).await;

        let Some(entry) = SESSION_STORE.get(&session_id) else {
            break;
        };

        let mut s = match entry.write() {
            Ok(guard) => guard,
            Err(_) => break,
        };

        if s.state != SessionState::Active {
            break;
        }

        let inactive_for = Utc::now() - s.updated_at;
        if inactive_for.num_seconds() >= INACTIVITY_TIMEOUT_SECS {
            s.state = SessionState::Ended;
            s.updated_at = Utc::now();
            break;
        }
    }
}
