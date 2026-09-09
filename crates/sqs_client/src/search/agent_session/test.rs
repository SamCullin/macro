use super::*;
use crate::search::SearchQueueMessage;

#[test]
fn agent_session_work_item_round_trips_with_an_index_override() {
    let message = SearchQueueMessage::AgentSession(AgentSession {
        agent_session_id: "00000000-0000-0000-0000-000000000007".to_owned(),
        index_override: Some("agent_sessions_v2".to_owned()),
    });

    let encoded = serde_json::to_string(&message).expect("serializable work item");
    let decoded: SearchQueueMessage = serde_json::from_str(&encoded).expect("decodable work item");

    assert_eq!(decoded, message);
}
