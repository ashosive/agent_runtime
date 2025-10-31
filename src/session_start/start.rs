use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::session_creation::types::{Session, SessionState};

#[derive(Debug, Serialize, Deserialize)]
pub struct StartSessionReceipt {
    pub session_id: Uuid,
    pub prev_state: SessionState,
    pub new_state: SessionState,
    pub updated_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub was_noop: bool,
}

#[derive(Debug)]
pub enum StartError {
    NotFound,
    InvalidState(SessionState),
    Poisoned,
}

pub fn start_session_inplace(s: &mut Session) -> Result<StartSessionReceipt, StartError> {
    let now = Utc::now();
    let prev = s.state.clone();

    match s.state {
        SessionState::Pending | SessionState::Paused | SessionState::Suspended => {
            s.state = SessionState::Active;
            s.updated_at = now;
            if s.started_at.is_none() {
                s.started_at = Some(now);
            }
            Ok(StartSessionReceipt {
                session_id: s.id,
                prev_state: prev,
                new_state: s.state.clone(),
                updated_at: s.updated_at,
                started_at: s.started_at,
                was_noop: false,
            })
        }
        SessionState::Active => {
            s.updated_at = now;
            Ok(StartSessionReceipt {
                session_id: s.id,
                prev_state: prev,
                new_state: s.state.clone(),
                updated_at: s.updated_at,
                started_at: s.started_at,
                was_noop: true,
            })
        }
        SessionState::Ended => Err(StartError::InvalidState(SessionState::Ended)),
    }
}
