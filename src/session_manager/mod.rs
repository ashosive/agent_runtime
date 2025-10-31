use std::sync::{Arc, RwLock};
use dashmap::DashMap;
use once_cell::sync::Lazy;
use uuid::Uuid;

use crate::session_creation::session_create::{create_session, CreateSessionRequest, SessionReceipt};
use crate::session_creation::types::{Session};
use crate::session_start::start::{start_session_inplace, StartSessionReceipt, StartError};

pub struct SessionManager;

static SESSION_STORE: Lazy<DashMap<Uuid, Arc<RwLock<Session>>>> = Lazy::new(DashMap::new);

impl SessionManager {
    pub fn create_session(req: CreateSessionRequest) -> (Arc<RwLock<Session>>, SessionReceipt) {
        let (session, receipt) = create_session(req);
        let shared = Arc::new(RwLock::new(session));
        SESSION_STORE.insert(receipt.session_id, Arc::clone(&shared));
        (shared, receipt)
    }

    pub fn start_session(id: &Uuid) -> Result<StartSessionReceipt, StartError> {
        let Some(entry) = SESSION_STORE.get(id) else {
            return Err(StartError::NotFound);
        };
        let mut s = entry.write().map_err(|_| StartError::Poisoned)?;
        start_session_inplace(&mut *s)
    }

    pub fn get_session(id: &Uuid) -> Option<Arc<RwLock<Session>>> {
        SESSION_STORE.get(id).map(|e| Arc::clone(e.value()))
    }

    pub fn list_session_ids() -> Vec<Uuid> {
        SESSION_STORE.iter().map(|e| *e.key()).collect()
    }

    pub fn count_sessions() -> usize {
        SESSION_STORE.len()
    }

    pub fn exists_session(id: &Uuid) -> bool {
        SESSION_STORE.contains_key(id)
    }

    pub fn remove_session(id: &Uuid) -> Option<Arc<RwLock<Session>>> {
        SESSION_STORE.remove(id).map(|(_, v)| v)
    }
}
