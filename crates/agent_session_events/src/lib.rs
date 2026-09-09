#![deny(missing_docs)]
//! Identifier-only invalidations for the agent-session search projection.
//!
//! Payloads never contain ACP frames or folded content. Consumers use the
//! session id to reread the authoritative log and run `agent_fold` themselves.

use macro_event_broker::{Event, MacroEvent, TopicEvent};
use macro_event_topics::MacroAgentSessionSearchTopic;
use macro_uuid::Uuid;
use serde::{Deserialize, Serialize};

#[cfg(test)]
mod test;

/// Session identifier carried by a search invalidation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentSessionSearchMetadata {
    /// Session whose search projection changed.
    pub agent_session_id: Uuid,
}

/// Changes that require rebuilding or removing an agent-session projection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "event_type", content = "metadata")]
pub enum AgentSessionSearchTopicEvent {
    /// Session metadata or its folded transcript changed.
    #[serde(rename = "agent_session_search.reconcile")]
    Reconcile(AgentSessionSearchMetadata),
    /// The authoritative session was permanently deleted.
    #[serde(rename = "agent_session_search.deleted")]
    Deleted(AgentSessionSearchMetadata),
}

impl TopicEvent for AgentSessionSearchTopicEvent {
    type Topic = MacroAgentSessionSearchTopic;

    const SCHEMA_VERSION: u8 = 1;
}

/// Publishable search invalidation keyed by the session UUID.
#[derive(Debug, Clone)]
pub struct AgentSessionSearchMacroEvent {
    key: String,
    event: Event<AgentSessionSearchTopicEvent>,
}

impl AgentSessionSearchMacroEvent {
    /// Request an authoritative reconciliation.
    #[must_use]
    pub fn reconcile(agent_session_id: Uuid) -> Self {
        Self::new(
            agent_session_id,
            AgentSessionSearchTopicEvent::Reconcile(AgentSessionSearchMetadata {
                agent_session_id,
            }),
        )
    }

    /// Remove the projection of a deleted session.
    #[must_use]
    pub fn deleted(agent_session_id: Uuid) -> Self {
        Self::new(
            agent_session_id,
            AgentSessionSearchTopicEvent::Deleted(AgentSessionSearchMetadata { agent_session_id }),
        )
    }

    fn new(agent_session_id: Uuid, event: AgentSessionSearchTopicEvent) -> Self {
        Self::with_event(agent_session_id.to_string(), Event::new(event))
    }

    fn with_event(key: String, event: Event<AgentSessionSearchTopicEvent>) -> Self {
        Self { key, event }
    }
}

impl MacroEvent for AgentSessionSearchMacroEvent {
    type EventPayload = AgentSessionSearchTopicEvent;

    fn key(&self) -> &str {
        &self.key
    }

    fn event(&self) -> &Event<Self::EventPayload> {
        &self.event
    }

    fn from_event(key: String, event: Event<Self::EventPayload>) -> Self {
        Self::with_event(key, event)
    }
}
