use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SessionState {
    Pending,
    Active,
    Paused,
    Suspended,
    Ended,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SessionLimits {
    pub max_tokens: u32,
    pub max_duration_ms: u64,
    pub max_context_bytes: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SessionContextSeed {
    pub system_prompt: String,
    pub user_prompt_snapshot: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SessionAccounting {
    pub prompt_tokens: u64,
    pub output_tokens: u64,
    pub requests: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Session {
    pub id: Uuid,
    pub state: SessionState,
    pub limits: SessionLimits,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub context_seed: SessionContextSeed,
    pub accounting: SessionAccounting,
}
