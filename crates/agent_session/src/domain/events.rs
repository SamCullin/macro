//! Kafka event models for the `macro.agent_session_lifecycle` topic.
//!
//! These are the session's *lifecycle* as other systems see it - it exists,
//! it is called this, it is in this state, it is gone - published so that
//! anything projecting sessions (Soup realtime, search, activity) learns of a
//! change without polling. Frames of the session's log are deliberately not
//! here: those stream to the session's viewers through
//! [`AgentSessionRealtime`](super::ports::AgentSessionRealtime), and a
//! consumer that wants them reads the log.
//!
//! The topic is distinct from `macro.agent_sessions`, whose records are the
//! harness's trigger signals with their own schema and consumer.

#[cfg(test)]
mod test;

use macro_event_broker::{Event, MacroEvent, TopicEvent};
use macro_event_topics::MacroAgentSessionLifecycleTopic;
use macro_user_id::user_id::MacroUserIdStr;
use serde::{Deserialize, Serialize};

use super::model::{AgentSession, AgentSessionId, SessionStatus};

/// A session's status as published, decoupled from the domain enum's serde.
///
/// `status` is the coarse state (`no_messages`, `event`, `disconnected`) and
/// `event_name` the wire name of the system event when `status` is `event` -
/// the same two columns the session row stores.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionStatusMetadata {
    /// Coarse state: `no_messages`, `event`, or `disconnected`.
    pub status: String,
    /// Wire name of the last system event, when `status` is `event`.
    pub event_name: Option<String>,
}

impl From<&SessionStatus> for SessionStatusMetadata {
    fn from(status: &SessionStatus) -> Self {
        Self {
            status: status.as_ref().to_owned(),
            event_name: match status {
                SessionStatus::Event(event) => Some(event.as_str().to_owned()),
                SessionStatus::NoMessages | SessionStatus::Disconnected => None,
            },
        }
    }
}

/// Metadata for [`AgentSessionLifecycleTopicEvent::Created`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentSessionCreatedMetadata {
    /// Identifier of the created session.
    pub agent_session_id: String,
    /// User who owns the session.
    pub owner: MacroUserIdStr<'static>,
    /// The bot running the session.
    pub bot_id: String,
    /// Display name the session was created with.
    pub name: String,
    /// The channel thread the session was opened from, when any.
    pub thread_id: Option<String>,
}

impl From<&AgentSession> for AgentSessionCreatedMetadata {
    fn from(session: &AgentSession) -> Self {
        Self {
            agent_session_id: session.id.to_string(),
            owner: session.owner_id.clone(),
            bot_id: session.bot_id.to_string(),
            name: session.name.clone(),
            thread_id: session.thread_id.map(|id| id.to_string()),
        }
    }
}

/// Metadata for [`AgentSessionLifecycleTopicEvent::Renamed`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentSessionRenamedMetadata {
    /// Identifier of the renamed session.
    pub agent_session_id: String,
    /// The new display name.
    pub name: String,
}

/// Metadata for [`AgentSessionLifecycleTopicEvent::StatusChanged`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentSessionStatusChangedMetadata {
    /// Identifier of the session whose status changed.
    pub agent_session_id: String,
    /// The status the session is now in.
    pub status: SessionStatusMetadata,
}

/// Metadata for [`AgentSessionLifecycleTopicEvent::Deleted`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentSessionDeletedMetadata {
    /// Identifier of the deleted session.
    pub agent_session_id: String,
}

/// Lifecycle events published to [`MacroAgentSessionLifecycleTopic`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "event_type", content = "metadata")]
pub enum AgentSessionLifecycleTopicEvent {
    /// A session was created.
    #[serde(rename = "agent_session.created")]
    Created(AgentSessionCreatedMetadata),
    /// A session's display name changed, by its owner or by auto-naming.
    #[serde(rename = "agent_session.renamed")]
    Renamed(AgentSessionRenamedMetadata),
    /// A system event moved the session to a new status.
    #[serde(rename = "agent_session.status_changed")]
    StatusChanged(AgentSessionStatusChangedMetadata),
    /// A session was deleted.
    #[serde(rename = "agent_session.deleted")]
    Deleted(AgentSessionDeletedMetadata),
}

impl TopicEvent for AgentSessionLifecycleTopicEvent {
    type Topic = MacroAgentSessionLifecycleTopic;

    const SCHEMA_VERSION: u8 = 1;
}

/// Publishable event for [`MacroAgentSessionLifecycleTopic`], keyed by the
/// session's bare id so one session's events stay ordered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentSessionLifecycleMacroEvent {
    key: String,
    event: Event<AgentSessionLifecycleTopicEvent>,
}

impl AgentSessionLifecycleMacroEvent {
    /// Build a created event from the session as it was persisted.
    #[must_use]
    pub fn created(session: &AgentSession) -> Self {
        Self::new(
            session.id,
            AgentSessionLifecycleTopicEvent::Created(session.into()),
        )
    }

    /// Build a renamed event.
    #[must_use]
    pub fn renamed(id: AgentSessionId, name: &str) -> Self {
        Self::new(
            id,
            AgentSessionLifecycleTopicEvent::Renamed(AgentSessionRenamedMetadata {
                agent_session_id: id.to_string(),
                name: name.to_owned(),
            }),
        )
    }

    /// Build a status-changed event.
    #[must_use]
    pub fn status_changed(id: AgentSessionId, status: &SessionStatus) -> Self {
        Self::new(
            id,
            AgentSessionLifecycleTopicEvent::StatusChanged(AgentSessionStatusChangedMetadata {
                agent_session_id: id.to_string(),
                status: status.into(),
            }),
        )
    }

    /// Build a deleted event.
    #[must_use]
    pub fn deleted(id: AgentSessionId) -> Self {
        Self::new(
            id,
            AgentSessionLifecycleTopicEvent::Deleted(AgentSessionDeletedMetadata {
                agent_session_id: id.to_string(),
            }),
        )
    }

    fn new(id: AgentSessionId, event: AgentSessionLifecycleTopicEvent) -> Self {
        Self::with_event(id.to_string(), Event::new(event))
    }

    fn with_event(key: String, event: Event<AgentSessionLifecycleTopicEvent>) -> Self {
        Self { key, event }
    }
}

impl MacroEvent for AgentSessionLifecycleMacroEvent {
    type EventPayload = AgentSessionLifecycleTopicEvent;

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
