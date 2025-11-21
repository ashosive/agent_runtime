use uuid::Uuid;
use chrono::Utc;
use serde::{Serialize, Deserialize};

use crate::session_creation::types::{
    Session,
    SessionState,
    SessionLimits,
    SessionContextSeed,
    SessionAccounting,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateSessionRequest {
    pub system_prompt: Option<String>,
    pub user_prompt_snapshot: Option<String>,
    pub max_tokens: Option<u32>,
    pub max_duration_ms: Option<u64>,
    pub max_context_bytes: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SessionReceipt {
    pub session_id: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub fn create_session(request: CreateSessionRequest) -> (Session, SessionReceipt) {
    let limits = SessionLimits {
        max_tokens: request.max_tokens.unwrap_or(2048),
        max_duration_ms: request.max_duration_ms.unwrap_or(60_000),
        max_context_bytes: request.max_context_bytes.unwrap_or(256 * 1024),
    };

    let context_seed = SessionContextSeed {
        system_prompt: request.system_prompt.unwrap_or_else(|| "You are an AI assistant.".to_string()),
        user_prompt_snapshot: request.user_prompt_snapshot,
    };

    let now = Utc::now();

    let session = Session {
        id: Uuid::new_v4(),
        state: SessionState::Pending,
        limits,
        created_at: now,
        updated_at: now,
        started_at: None,
        context_seed,
        accounting: SessionAccounting {
            prompt_tokens: 0,
            output_tokens: 0,
            requests: 0,
        },
        model: None,
    };

    let receipt = SessionReceipt {
        session_id: session.id,
        created_at: now,
    };

    (session, receipt)
}
