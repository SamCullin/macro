use agent_runtime_protocol::domain::schema::v0::SystemEvent;
use macro_event_broker::{Event, MacroEvent};
use macro_event_topics::{MacroAgentSessionLifecycleTopic, Topic};
use macro_user_id::user_id::MacroUserIdStr;
use macro_uuid::Uuid;
use serde_json::{Value, json};

use super::*;

const SESSION_ID: &str = "0197f776-6e7b-7c69-a251-780ae754d3e4";
const BOT_ID: &str = "3f6f8b0a-6f9f-4a3f-9c3a-2b1e5d4c7a90";
const THREAD_ID: &str = "c1a2b3d4-e5f6-4a7b-8c9d-0e1f2a3b4c5d";
const EVENT_ID: &str = "01998a30-1a2b-7c3d-9e4f-5a6b7c8d9e0f";

fn user_id(value: &str) -> MacroUserIdStr<'static> {
    MacroUserIdStr::try_from(value.to_string()).expect("valid user id")
}

fn session_id() -> AgentSessionId {
    AgentSessionId::new_from_uuid(Uuid::parse_str(SESSION_ID).expect("valid session id"))
}

fn topic_events() -> Vec<(AgentSessionLifecycleTopicEvent, Value)> {
    vec![
        (
            AgentSessionLifecycleTopicEvent::Created(AgentSessionCreatedMetadata {
                agent_session_id: SESSION_ID.to_string(),
                owner: user_id("macro|owner@acme.com"),
                bot_id: BOT_ID.to_string(),
                name: "New Agent Session".to_string(),
                thread_id: Some(THREAD_ID.to_string()),
            }),
            json!({
                "event_type": "agent_session.created",
                "metadata": {
                    "agent_session_id": SESSION_ID,
                    "owner": "macro|owner@acme.com",
                    "bot_id": BOT_ID,
                    "name": "New Agent Session",
                    "thread_id": THREAD_ID
                }
            }),
        ),
        (
            AgentSessionLifecycleTopicEvent::Renamed(AgentSessionRenamedMetadata {
                agent_session_id: SESSION_ID.to_string(),
                name: "Fix the flaky test".to_string(),
            }),
            json!({
                "event_type": "agent_session.renamed",
                "metadata": {
                    "agent_session_id": SESSION_ID,
                    "name": "Fix the flaky test"
                }
            }),
        ),
        (
            AgentSessionLifecycleTopicEvent::StatusChanged(AgentSessionStatusChangedMetadata {
                agent_session_id: SESSION_ID.to_string(),
                status: SessionStatusMetadata {
                    status: "event".to_string(),
                    event_name: Some("acp_ready".to_string()),
                },
            }),
            json!({
                "event_type": "agent_session.status_changed",
                "metadata": {
                    "agent_session_id": SESSION_ID,
                    "status": { "status": "event", "event_name": "acp_ready" }
                }
            }),
        ),
        (
            AgentSessionLifecycleTopicEvent::StatusChanged(AgentSessionStatusChangedMetadata {
                agent_session_id: SESSION_ID.to_string(),
                status: SessionStatusMetadata {
                    status: "disconnected".to_string(),
                    event_name: None,
                },
            }),
            json!({
                "event_type": "agent_session.status_changed",
                "metadata": {
                    "agent_session_id": SESSION_ID,
                    "status": { "status": "disconnected", "event_name": null }
                }
            }),
        ),
        (
            AgentSessionLifecycleTopicEvent::Deleted(AgentSessionDeletedMetadata {
                agent_session_id: SESSION_ID.to_string(),
            }),
            json!({
                "event_type": "agent_session.deleted",
                "metadata": {
                    "agent_session_id": SESSION_ID
                }
            }),
        ),
    ]
}

fn macro_events() -> Vec<AgentSessionLifecycleMacroEvent> {
    vec![
        AgentSessionLifecycleMacroEvent::renamed(session_id(), "Fix the flaky test"),
        AgentSessionLifecycleMacroEvent::status_changed(
            session_id(),
            &SessionStatus::Event(SystemEvent::AcpReady),
        ),
        AgentSessionLifecycleMacroEvent::status_changed(session_id(), &SessionStatus::Disconnected),
        AgentSessionLifecycleMacroEvent::status_changed(session_id(), &SessionStatus::NoMessages),
        AgentSessionLifecycleMacroEvent::deleted(session_id()),
    ]
}

#[test]
fn every_variant_has_exact_json_envelope() {
    let event_id = Uuid::parse_str(EVENT_ID).expect("valid event id");

    for (event, expected_payload) in topic_events() {
        let mut expected = expected_payload;
        let object = expected.as_object_mut().expect("expected object");
        object.insert("event_id".to_string(), json!(EVENT_ID));
        object.insert("schema_version".to_string(), json!(1));

        assert_eq!(
            serde_json::to_value(Event::with_event_id(event_id, event))
                .expect("serializable event"),
            expected
        );
    }
}

#[test]
fn every_variant_round_trips() {
    for original in macro_events() {
        let payload = serde_json::to_vec(original.event()).expect("serializable event");
        let decoded = AgentSessionLifecycleMacroEvent::decode(original.key(), &payload)
            .expect("decodable event");

        assert_eq!(decoded.key(), SESSION_ID);
        assert_eq!(decoded.event(), original.event());
        assert_eq!(decoded.topic(), MacroAgentSessionLifecycleTopic::TOPIC_STR);
        assert_eq!(decoded.topic(), "macro.agent_session_lifecycle");
    }
}

#[test]
fn constructors_use_lifecycle_topic_bare_session_id_key_and_schema_version_one() {
    for event in macro_events() {
        assert_eq!(event.key(), SESSION_ID);
        assert_eq!(event.topic(), "macro.agent_session_lifecycle");
        assert_eq!(event.event().schema_version, 1);
    }
}

#[test]
fn status_metadata_mirrors_the_session_row_columns() {
    assert_eq!(
        SessionStatusMetadata::from(&SessionStatus::NoMessages),
        SessionStatusMetadata {
            status: "no_messages".to_string(),
            event_name: None,
        }
    );
    assert_eq!(
        SessionStatusMetadata::from(&SessionStatus::Event(SystemEvent::Unknown(
            "custom_event".to_string()
        ))),
        SessionStatusMetadata {
            status: "event".to_string(),
            event_name: Some("custom_event".to_string()),
        }
    );
}
