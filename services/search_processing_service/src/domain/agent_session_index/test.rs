use std::sync::{Arc, Mutex};

use super::*;

#[derive(Clone)]
struct FakeSource {
    snapshot: Option<AgentSessionSearchSnapshot>,
}

impl AgentSessionSearchSource for FakeSource {
    async fn load(
        &self,
        _id: AgentSessionId,
    ) -> Result<Option<AgentSessionSearchSnapshot>, Report> {
        Ok(self.snapshot.clone())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum IndexCall {
    Reconcile {
        id: AgentSessionId,
        name: String,
        messages: usize,
        index_override: Option<String>,
    },
    Delete {
        id: AgentSessionId,
        index_override: Option<String>,
    },
}

#[derive(Clone, Default)]
struct FakeIndex {
    calls: Arc<Mutex<Vec<IndexCall>>>,
}

impl AgentSessionSearchIndex for FakeIndex {
    async fn reconcile(
        &self,
        projection: AgentSessionSearchProjection,
        index_override: Option<&str>,
    ) -> Result<(), Report> {
        self.calls.lock().unwrap().push(IndexCall::Reconcile {
            id: projection.id,
            name: projection.name,
            messages: projection.messages.len(),
            index_override: index_override.map(str::to_owned),
        });
        Ok(())
    }

    async fn delete(&self, id: AgentSessionId, index_override: Option<&str>) -> Result<(), Report> {
        self.calls.lock().unwrap().push(IndexCall::Delete {
            id,
            index_override: index_override.map(str::to_owned),
        });
        Ok(())
    }
}

fn session_id(value: u128) -> AgentSessionId {
    AgentSessionId::new_from_uuid(Uuid::from_u128(value))
}

fn snapshot(id: AgentSessionId) -> AgentSessionSearchSnapshot {
    let timestamp = DateTime::from_timestamp_millis(1_700_000_000_000).unwrap();
    AgentSessionSearchSnapshot {
        id,
        name: "Index agent chats".to_owned(),
        owner_id: MacroUserIdStr::try_from("macro|owner@example.com".to_owned()).unwrap(),
        bot_id: Uuid::from_u128(2),
        thread_id: None,
        originating_message_id: None,
        created_at: timestamp,
        modified_at: timestamp,
        log: Vec::new(),
        last_log_id: None,
    }
}

#[tokio::test]
async fn a_present_session_is_folded_and_reconciled() {
    let id = session_id(1);
    let index = FakeIndex::default();
    let service = AgentSessionIndexService::new(
        FakeSource {
            snapshot: Some(snapshot(id)),
        },
        index.clone(),
    );

    service
        .reconcile(id, Some("agent_sessions_v2"))
        .await
        .unwrap();

    assert_eq!(
        *index.calls.lock().unwrap(),
        [IndexCall::Reconcile {
            id,
            name: "Index agent chats".to_owned(),
            messages: 0,
            index_override: Some("agent_sessions_v2".to_owned()),
        }]
    );
}

#[tokio::test]
async fn a_missing_session_removes_any_stale_projection() {
    let id = session_id(1);
    let index = FakeIndex::default();
    let service = AgentSessionIndexService::new(FakeSource { snapshot: None }, index.clone());

    service.reconcile(id, None).await.unwrap();

    assert_eq!(
        *index.calls.lock().unwrap(),
        [IndexCall::Delete {
            id,
            index_override: None,
        }]
    );
}
