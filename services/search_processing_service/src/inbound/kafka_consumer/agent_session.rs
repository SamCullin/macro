//! Maps agent-session invalidations to authoritative search reconciliation.

use agent_fold::domain::log::AgentSessionId;
use agent_session_events::{AgentSessionSearchMacroEvent, AgentSessionSearchTopicEvent};
use macro_event_broker::MacroEvent as _;

use super::{EventOutcome, MAX_PROCESSING_ATTEMPTS, PROCESSING_RETRY_BASE_DELAY, retry_processing};
use crate::inbound::kafka_consumer::KafkaProcessingContext;

pub(super) async fn process_agent_session_event(
    context: &KafkaProcessingContext,
    event: &AgentSessionSearchMacroEvent,
    partition: i32,
    offset: i64,
) -> EventOutcome {
    let (raw_id, event_type, deleted) = match &event.event().event {
        AgentSessionSearchTopicEvent::Reconcile(metadata) => (
            metadata.agent_session_id,
            "agent_session_search.reconcile",
            false,
        ),
        AgentSessionSearchTopicEvent::Deleted(metadata) => (
            metadata.agent_session_id,
            "agent_session_search.deleted",
            true,
        ),
    };
    let id = AgentSessionId::new_from_uuid(raw_id);

    let result = retry_processing(|attempt| async move {
        tracing::trace!(
            agent_session_id = %id,
            event_type,
            partition,
            offset,
            attempt,
            "processing agent-session search event"
        );
        let result = if deleted {
            context.agent_session_indexer.delete(id, None).await
        } else {
            context.agent_session_indexer.reconcile(id, None).await
        };
        result.inspect_err(|error| {
            if attempt < MAX_PROCESSING_ATTEMPTS {
                let retry_delay = PROCESSING_RETRY_BASE_DELAY * 2u32.pow(attempt.saturating_sub(1));
                tracing::warn!(
                    error = ?error,
                    agent_session_id = %id,
                    event_type,
                    partition,
                    offset,
                    attempt,
                    delay_secs = retry_delay.as_secs(),
                    "agent-session search processing failed, retrying"
                );
            }
        })
    })
    .await;

    match result {
        Ok(()) => EventOutcome::Indexed,
        Err(error) => {
            tracing::error!(
                error = ?error,
                agent_session_id = %id,
                event_type,
                partition,
                offset,
                attempts = MAX_PROCESSING_ATTEMPTS,
                "dropping agent-session event after processing retries were exhausted"
            );
            EventOutcome::Dropped
        }
    }
}
