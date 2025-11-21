use uuid::Uuid;
use chrono::Utc;
use crate::session_manager::SESSION_STORE;
use crate::session_creation::types::SessionState;

pub fn build_prompt(session_id: &Uuid) -> Option<String> {
    let entry = SESSION_STORE.get(session_id)?;
    let s = entry.read().ok()?;

    if s.state != SessionState::Active {
        return None;
    }

    let mut p = s.context_seed.system_prompt.clone();

    if let Some(user) = &s.context_seed.user_prompt_snapshot {
        p.push_str("\n\nUser: ");
        p.push_str(user);
    }

    Some(p)
}

pub fn apply_user_input(session_id: &Uuid, input: &str) -> Option<String> {
    {
        let entry = SESSION_STORE.get(session_id)?;
        let mut s = entry.write().ok()?;

        if s.state != SessionState::Active {
            return None;
        }

        s.context_seed.user_prompt_snapshot = Some(input.to_string());
        s.updated_at = Utc::now();
    }

    build_prompt(session_id)
}
