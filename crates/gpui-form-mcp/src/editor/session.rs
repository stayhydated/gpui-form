use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use serde_json::{Map, Value};

use crate::*;

#[derive(Debug)]
pub(crate) struct EditSession<Holder> {
    pub(crate) holder: Holder,
    pub(crate) present_fields: BTreeSet<String>,
    pub(crate) values: Map<String, Value>,
    pub(crate) revision: u64,
    pub(crate) opened_order: u64,
    pub(crate) last_accessed_at: Instant,
}

#[derive(Debug)]
pub(crate) struct EditSessions<Holder> {
    pub(crate) next_id: u64,
    pub(crate) sessions: BTreeMap<String, EditSession<Holder>>,
    pub(crate) options: McpFormEditorOptions,
}

#[derive(Debug, Default)]
pub(crate) struct EditSessionCleanup {
    pub(crate) expired_session_ids: Vec<String>,
    pub(crate) evicted_session_ids: Vec<String>,
}

#[derive(Debug)]
pub(crate) struct OpenedEditSession {
    pub(crate) session_id: String,
    pub(crate) cleanup: EditSessionCleanup,
}

impl<Holder> Default for EditSessions<Holder> {
    fn default() -> Self {
        Self::new(McpFormEditorOptions::default())
    }
}

impl<Holder> EditSessions<Holder> {
    pub(crate) fn new(options: McpFormEditorOptions) -> Self {
        Self {
            next_id: 1,
            sessions: BTreeMap::new(),
            options,
        }
    }

    pub(crate) fn open(
        &mut self,
        holder: Holder,
        present_fields: BTreeSet<String>,
        values: Map<String, Value>,
    ) -> OpenedEditSession {
        let now = Instant::now();
        let expired_session_ids = self.expire_idle_sessions(now);
        let session_id = self.next_id.to_string();
        self.next_id += 1;
        self.sessions.insert(
            session_id.clone(),
            EditSession {
                holder,
                present_fields,
                values,
                revision: 0,
                opened_order: self.next_id - 1,
                last_accessed_at: now,
            },
        );
        let evicted_session_ids = self.enforce_session_limit();
        OpenedEditSession {
            session_id,
            cleanup: EditSessionCleanup {
                expired_session_ids,
                evicted_session_ids,
            },
        }
    }

    pub(crate) fn get(&mut self, session_id: &str) -> Result<&EditSession<Holder>, McpToolError> {
        let now = Instant::now();
        self.expire_idle_sessions(now);
        let session = self
            .sessions
            .get_mut(session_id)
            .ok_or_else(|| McpToolError::invalid_field_value("session_id", session_id))?;
        session.last_accessed_at = now;
        Ok(session)
    }

    pub(crate) fn get_mut(
        &mut self,
        session_id: &str,
    ) -> Result<&mut EditSession<Holder>, McpToolError> {
        let now = Instant::now();
        self.expire_idle_sessions(now);
        let session = self
            .sessions
            .get_mut(session_id)
            .ok_or_else(|| McpToolError::invalid_field_value("session_id", session_id))?;
        session.last_accessed_at = now;
        Ok(session)
    }

    pub(crate) fn close(&mut self, session_id: &str) -> Result<bool, McpToolError> {
        if self.sessions.remove(session_id).is_some() {
            Ok(true)
        } else {
            Err(McpToolError::invalid_field_value("session_id", session_id))
        }
    }

    pub(crate) fn close_all(&mut self) -> Vec<String> {
        let session_ids = self.sessions.keys().cloned().collect();
        self.sessions.clear();
        session_ids
    }

    pub(crate) fn session_limit(&self) -> Option<usize> {
        self.options.session_limit()
    }

    pub(crate) fn session_idle_timeout(&self) -> Option<Duration> {
        self.options.session_idle_timeout()
    }

    pub(crate) fn expire_idle_sessions(&mut self, now: Instant) -> Vec<String> {
        let Some(timeout) = self.options.session_idle_timeout() else {
            return Vec::new();
        };
        let expired = self
            .sessions
            .iter()
            .filter(|&(_, session)| now.duration_since(session.last_accessed_at) >= timeout)
            .map(|(session_id, _)| session_id.clone())
            .collect::<Vec<_>>();
        for session_id in &expired {
            self.sessions.remove(session_id);
        }
        expired
    }

    pub(crate) fn enforce_session_limit(&mut self) -> Vec<String> {
        let Some(limit) = self.options.session_limit() else {
            return Vec::new();
        };
        let excess = self.sessions.len().saturating_sub(limit);
        if excess == 0 {
            return Vec::new();
        }

        let mut session_ids = self
            .sessions
            .iter()
            .map(|(session_id, session)| (session.opened_order, session_id.clone()))
            .collect::<Vec<_>>();
        session_ids.sort_by_key(|(opened_order, _)| *opened_order);
        let evicted = session_ids
            .into_iter()
            .take(excess)
            .map(|(_, session_id)| session_id)
            .collect::<Vec<_>>();
        for session_id in &evicted {
            self.sessions.remove(session_id.as_str());
        }
        evicted
    }
}

pub(crate) type SharedEditSessions<Holder> = Arc<Mutex<EditSessions<Holder>>>;
