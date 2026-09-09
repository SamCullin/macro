use super::*;

fn session() -> Uuid {
    Uuid::from_u128(7)
}

#[test]
fn reconcile_is_identifier_only_and_keyed_by_session() {
    let event = AgentSessionSearchMacroEvent::reconcile(session());

    assert_eq!(event.key(), session().to_string());
    assert_eq!(
        serde_json::to_value(event.event()).unwrap(),
        serde_json::json!({
            "event_id": event.event().event_id,
            "schema_version": 1,
            "event_type": "agent_session_search.reconcile",
            "metadata": { "agent_session_id": session() }
        })
    );
}
