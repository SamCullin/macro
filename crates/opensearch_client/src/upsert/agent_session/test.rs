use serde_json::json;

use super::*;

fn args() -> ReconcileAgentSessionArgs {
    ReconcileAgentSessionArgs {
        agent_session_id: "session-1".to_owned(),
        name: "Investigate indexing".to_owned(),
        owner_id: "user-1".to_owned(),
        bot_id: "bot-1".to_owned(),
        thread_id: Some("thread-1".to_owned()),
        originating_message_id: None,
        created_at_millis: EpochMillis::new(10).unwrap(),
        updated_at_millis: EpochMillis::new(20).unwrap(),
        projection_generation: "generation-1".to_owned(),
        messages: vec![AgentSessionMessageDocument {
            turn: 3,
            author: "agent".to_owned(),
            author_user_id: None,
            content: "Folded output".to_owned(),
        }],
    }
}

#[test]
fn parent_and_child_documents_share_session_and_generation() {
    let args = args();

    assert_eq!(
        serde_json::to_value(parent_document(&args)).unwrap(),
        json!({
            "agent_session_id": "session-1",
            "projection_generation": "generation-1",
            "name": "Investigate indexing",
            "owner_id": "user-1",
            "bot_id": "bot-1",
            "thread_id": "thread-1",
            "originating_message_id": null,
            "created_at_millis": 10,
            "updated_at_millis": 20,
            "agent_session_relation": "agent_session",
        })
    );
    assert_eq!(
        serde_json::to_value(child_document(&args, &args.messages[0])).unwrap(),
        json!({
            "agent_session_id": "session-1",
            "projection_generation": "generation-1",
            "message_turn": 3,
            "author": "agent",
            "author_user_id": null,
            "content": "Folded output",
            "agent_session_relation": {
                "name": "message",
                "parent": "session-1",
            },
        })
    );
    assert_eq!(
        child_id("session-1", &args.messages[0]),
        "session-1:3:agent"
    );
}

#[test]
fn index_override_selects_a_physical_backfill_index() {
    assert_eq!(resolve_destination(None), "agent_sessions");
    assert_eq!(
        resolve_destination(Some("agent_sessions_v2")),
        "agent_sessions_v2"
    );
}
